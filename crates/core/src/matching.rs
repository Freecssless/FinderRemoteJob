//! Puntaje de compatibilidad perfil ↔ vacante (0–100), para ordenar vacantes. Plan §4.4 A:
//!
//! | Componente          | Peso |
//! |---------------------|------|
//! | Habilidades         | 45   |
//! | Similitud semántica | 15   | (opcional: sin embeddings se reparte en proporción)
//! | Puesto              | 10   |
//! | Seniority           | 5    |
//! | Años de experiencia | 10   |
//! | Logística           | 15   | (remoto 6 · región 5 · idiomas 4)
//!
//! Con un bloqueante ("US only", idioma requerido que no hablas…) el total se limita a 20.
//! Es *compatibilidad*, no probabilidad. Con suficientes postulaciones con resultado, los pesos se
//! pueden reajustar a tus datos ([`learn_weights`] y [`reweigh`]); el usuario decide si aplicarlos.

use serde::{Deserialize, Serialize};

use crate::ats::keywords::{SkillDict, at_word_boundary, contains_word, normalize};
use crate::ats::score::{Evidence, title_overlap};
use crate::model::{CvMaster, Job, JobRequirements, Profile, RemotePref, TermKind};

const MAX_SKILLS: f32 = 45.0;
const MAX_SEMANTIC: f32 = 15.0;
const MAX_TITLE: f32 = 10.0;
const MAX_SENIORITY: f32 = 5.0;
const MAX_YEARS: f32 = 10.0;
const MAX_REMOTE: f32 = 6.0;
const MAX_REGION: f32 = 5.0;
const MAX_LANGUAGES: f32 = 4.0;
const BLOCKED_CAP: f32 = 20.0;
/// Si la vacante pide habilidades y cubres menos del 20 %, el total no pasa de 30:
/// evita que remoto + LATAM hagan parecer compatible una vacante de otro oficio.
const LOW_RELEVANCE_COVERAGE: f32 = 0.2;
const LOW_RELEVANCE_CAP: f32 = 30.0;

pub struct MatchInput<'a> {
    pub profile: &'a Profile,
    pub master: &'a CvMaster,
    pub job: &'a Job,
    pub requirements: &'a JobRequirements,
    /// Coseno entre embeddings del maestro y la vacante, si ya se calcularon (módulo 7).
    pub semantic: Option<f32>,
    /// Mes actual "AAAA-MM", para experiencias sin fecha de fin. Se recibe de fuera: `core` no lee el reloj.
    pub today: &'a str,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MatchReport {
    /// 0–100.
    pub total: u8,
    pub breakdown: MatchBreakdown,
    pub years_experience: f32,
    /// Términos requeridos sin respaldo en el CV maestro.
    pub missing_required: Vec<String>,
    /// Motivos que impiden postular; si hay alguno, `total` ≤ 20.
    pub blockers: Vec<String>,
    /// Cubres menos del 20 % de las habilidades pedidas; la interfaz lo muestra como "Poco relevante".
    pub low_relevance: bool,
    /// Máximo de cada parte del desglose (los pesos en uso).
    pub max: Weights,
}

/// Puntos máximos de cada parte; suman 100.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Weights {
    pub skills: f32,
    pub semantic: f32,
    pub title: f32,
    pub seniority: f32,
    pub years: f32,
    pub logistics: f32,
}

impl Default for Weights {
    fn default() -> Self {
        Weights {
            skills: MAX_SKILLS,
            semantic: MAX_SEMANTIC,
            title: MAX_TITLE,
            seniority: MAX_SENIORITY,
            years: MAX_YEARS,
            logistics: MAX_REMOTE + MAX_REGION + MAX_LANGUAGES,
        }
    }
}

impl Weights {
    fn parts(&self) -> [f32; 6] {
        [
            self.skills,
            self.semantic,
            self.title,
            self.seniority,
            self.years,
            self.logistics,
        ]
    }

    fn from_parts(p: [f32; 6]) -> Self {
        Weights {
            skills: p[0],
            semantic: p[1],
            title: p[2],
            seniority: p[3],
            years: p[4],
            logistics: p[5],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MatchBreakdown {
    pub skills: f32,
    pub semantic: Option<f32>,
    pub title: f32,
    pub seniority: f32,
    pub years: f32,
    pub logistics: f32,
}

pub fn match_score(dict: &SkillDict, input: &MatchInput) -> MatchReport {
    let MatchInput {
        profile,
        master,
        job,
        requirements: req,
        semantic,
        today,
    } = *input;
    let ev = Evidence::new(dict, master);
    let mut blockers = Vec::new();

    // --- Habilidades: aquí un sinónimo cuenta completo (importa tenerla, no cómo se escribe).
    // Los idiomas se evalúan en logística para no contarlos dos veces.
    let (mut got, mut total_w) = (0.0f32, 0.0f32);
    let mut missing_required = Vec::new();
    for t in req.terms.iter().filter(|t| t.kind != TermKind::Language) {
        total_w += t.weight;
        if ev.has(dict, t) {
            got += t.weight;
        } else if t.required {
            missing_required.push(t.surface.clone());
        }
    }
    // Sin términos extraídos no hay con qué comparar: valor neutro.
    let low_relevance = total_w > 0.0 && got / total_w < LOW_RELEVANCE_COVERAGE;
    let skills = MAX_SKILLS * if total_w > 0.0 { got / total_w } else { 0.5 };

    // --- Similitud semántica.
    let semantic = semantic.map(|cos| MAX_SEMANTIC * rescale_cosine(cos));

    // --- Puesto: el mejor entre titular, puestos anteriores y puestos objetivo del perfil.
    let title = MAX_TITLE
        * std::iter::once(master.basics.headline.as_str())
            .chain(master.experience.iter().map(|e| e.title.as_str()))
            .chain(profile.target_titles.iter().map(String::as_str))
            .map(|t| title_overlap(t, &job.title))
            .fold(0.0, f32::max);

    // --- Seniority y años.
    let years_experience = years_of_experience(master, today);
    let seniority_text = format!("{} {}", job.title, req.seniority.as_deref().unwrap_or(""));
    let seniority = match job_level(&seniority_text) {
        None => MAX_SENIORITY,
        Some(wanted) => match candidate_level(years_experience).abs_diff(wanted) {
            0 => MAX_SENIORITY,
            1 => MAX_SENIORITY / 2.0,
            _ => 0.0,
        },
    };
    let years = match req.years_min {
        Some(min) if min > 0.0 => MAX_YEARS * (years_experience / min).min(1.0),
        _ => MAX_YEARS,
    };

    // --- Logística.
    let job_text = normalize(&format!(
        "{}\n{}\n{}\n{}",
        job.title,
        job.location.as_deref().unwrap_or(""),
        job.remote_type.as_deref().unwrap_or(""),
        job.description
    ));
    let remote = remote_score(profile.remote_pref, work_mode(&job_text));
    let region_ok = REGION_MARKERS.iter().any(|m| contains_word(&job_text, m))
        || profile
            .locations
            .iter()
            .any(|l| contains_word(&job_text, &normalize(l)));
    let region = if region_ok {
        MAX_REGION
    } else {
        MAX_REGION / 2.0
    };

    let mut wanted_langs: Vec<&str> = req
        .terms
        .iter()
        .filter(|t| t.kind == TermKind::Language && t.required)
        .map(|t| t.canonical.as_str())
        .chain(req.languages.iter().map(String::as_str))
        .collect();
    wanted_langs.sort_unstable();
    wanted_langs.dedup();
    // Cada idioma vale 1; con menos nivel del pedido, la proporción (B2 frente a C1 = 4/5).
    let mut lang_credit = 0.0f32;
    for l in &wanted_langs {
        if !ev.has_name(dict, l) {
            blockers.push(format!("Piden {l}, que no aparece en tu CV."));
            continue;
        }
        match (
            job_language_level(dict, &job.description, l),
            cv_language_level(dict, master, l),
        ) {
            (Some(want), Some(have)) if have < want => {
                lang_credit += f32::from(have) / f32::from(want);
                missing_required.push(format!(
                    "{l} {} (tu CV dice {})",
                    CEFR[usize::from(want)],
                    CEFR[usize::from(have)]
                ));
            }
            _ => lang_credit += 1.0,
        }
    }
    let languages = if wanted_langs.is_empty() {
        MAX_LANGUAGES
    } else {
        MAX_LANGUAGES * lang_credit / wanted_langs.len() as f32
    };
    let logistics = remote + region + languages;

    // --- Bloqueantes por texto del anuncio + los que haya detectado el LLM.
    for m in BLOCKER_MARKERS
        .iter()
        .filter(|m| contains_word(&job_text, m))
    {
        blockers.push(format!("La vacante dice «{m}»."));
    }
    blockers.extend(req.blockers.iter().cloned());

    // --- Total (sin embeddings, los 85 puntos restantes se escalan a 100).
    let breakdown = MatchBreakdown {
        skills,
        semantic,
        title,
        seniority,
        years,
        logistics,
    };
    MatchReport {
        total: capped_total(
            &breakdown,
            MAX_SEMANTIC,
            low_relevance,
            !blockers.is_empty(),
        ),
        breakdown,
        years_experience,
        missing_required,
        blockers,
        low_relevance,
        max: Weights::default(),
    }
}

/// Total con los límites de siempre: sin similitud semántica se escala a 100, y los topes de
/// «poco relevante» y bloqueos se respetan.
fn capped_total(
    b: &MatchBreakdown,
    semantic_weight: f32,
    low_relevance: bool,
    blocked: bool,
) -> u8 {
    let mut sum = b.skills + b.title + b.seniority + b.years + b.logistics;
    match b.semantic {
        Some(s) => sum += s,
        None => sum *= 100.0 / (100.0 - semantic_weight),
    }
    if low_relevance {
        sum = sum.min(LOW_RELEVANCE_CAP);
    }
    if blocked {
        sum = sum.min(BLOCKED_CAP);
    }
    sum.round().clamp(0.0, 100.0) as u8
}

/// El mismo puntaje con otros pesos: cada parte conserva su proporción (valor / máximo).
pub fn reweigh(r: &MatchReport, w: &Weights) -> MatchReport {
    let f = |v: f32, new: f32, old: f32| if old > 0.0 { v * new / old } else { 0.0 };
    let o = r.max;
    let breakdown = MatchBreakdown {
        skills: f(r.breakdown.skills, w.skills, o.skills),
        semantic: r.breakdown.semantic.map(|s| f(s, w.semantic, o.semantic)),
        title: f(r.breakdown.title, w.title, o.title),
        seniority: f(r.breakdown.seniority, w.seniority, o.seniority),
        years: f(r.breakdown.years, w.years, o.years),
        logistics: f(r.breakdown.logistics, w.logistics, o.logistics),
    };
    MatchReport {
        total: capped_total(
            &breakdown,
            w.semantic,
            r.low_relevance,
            !r.blockers.is_empty(),
        ),
        breakdown,
        max: *w,
        ..r.clone()
    }
}

/// Postulaciones con resultado necesarias para sugerir pesos, y mínimo de cada tipo.
pub const LEARN_MIN_SAMPLES: usize = 30;
pub const LEARN_MIN_EACH: usize = 5;

/// Pesos sugeridos por tus resultados. Cada muestra es el desglose con los pesos normales y si
/// la postulación avanzó (entrevista, en proceso u oferta). Sube el peso de las partes que eran
/// más altas en las que avanzaron y baja el de las que no distinguían; cada peso cambia como
/// mucho ±50 % y la suma sigue en 100. `None` si aún no hay datos suficientes.
pub fn learn_weights(samples: &[(MatchBreakdown, bool)]) -> Option<Weights> {
    let pos = samples.iter().filter(|(_, ok)| *ok).count();
    let neg = samples.len() - pos;
    if samples.len() < LEARN_MIN_SAMPLES || pos < LEARN_MIN_EACH || neg < LEARN_MIN_EACH {
        return None;
    }
    let d = Weights::default().parts();
    let values = |b: &MatchBreakdown| -> [Option<f32>; 6] {
        [
            Some(b.skills),
            b.semantic,
            Some(b.title),
            Some(b.seniority),
            Some(b.years),
            Some(b.logistics),
        ]
    };
    let mut parts = d;
    for (i, part) in parts.iter_mut().enumerate() {
        let mean = |want: bool| {
            let v: Vec<f32> = samples
                .iter()
                .filter(|(_, ok)| *ok == want)
                .filter_map(|(b, _)| values(b)[i])
                .map(|v| v / d[i])
                .collect();
            (v.len() >= LEARN_MIN_EACH).then(|| v.iter().sum::<f32>() / v.len() as f32)
        };
        if let (Some(p), Some(n)) = (mean(true), mean(false)) {
            *part = d[i] * (1.0 + (p - n)).clamp(0.5, 1.5);
        }
    }
    let total: f32 = parts.iter().sum();
    Some(Weights::from_parts(parts.map(|p| p * 100.0 / total)))
}

/// Lo que representa al CV maestro en los embeddings: titular, habilidades, puestos y logros
/// (sin nombre ni datos de contacto).
pub fn cv_semantic_text(m: &CvMaster) -> String {
    let mut parts = vec![m.basics.headline.clone()];
    parts.push(
        m.skills
            .iter()
            .map(|s| s.name.as_str())
            .collect::<Vec<_>>()
            .join(", "),
    );
    for e in &m.experience {
        parts.push(e.title.clone());
        parts.extend(e.bullets.iter().map(|b| b.text.clone()));
    }
    truncate_chars(&parts.join("\n"), SEMANTIC_MAX_CHARS)
}

/// Puesto y descripción de la vacante, recortados para que quepan en el modelo de embeddings.
pub fn job_semantic_text(job: &Job) -> String {
    truncate_chars(
        &format!("{}\n{}", job.title, job.description),
        SEMANTIC_MAX_CHARS,
    )
}

/// ~1500 tokens: lo esencial de una vacante, y rápido en CPU.
const SEMANTIC_MAX_CHARS: usize = 6000;

fn truncate_chars(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

/// Similitud coseno entre dos embeddings (−1 a 1). `None` si difieren en tamaño o alguno es cero.
pub fn cosine(a: &[f32], b: &[f32]) -> Option<f32> {
    if a.len() != b.len() || a.is_empty() {
        return None;
    }
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let na = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    (na > 0.0 && nb > 0.0).then(|| dot / (na * nb))
}

/// Los cosenos de embeddings rara vez bajan de ~0.3 entre textos del mismo idioma:
/// 0.3 → 0 y 0.8 → 1.
/// shortcut: umbrales a ojo; calibrarlos con bge-m3 y vacantes reales en el módulo 7.
fn rescale_cosine(cos: f32) -> f32 {
    ((cos - 0.3) / 0.5).clamp(0.0, 1.0)
}

// ---------------------------------------------------------------------------
// Experiencia y nivel
// ---------------------------------------------------------------------------

/// "AAAA-MM" o "AAAA" → número de mes absoluto.
fn month_index(s: &str) -> Option<i32> {
    let (y, m) = s.trim().split_once('-').unwrap_or((s.trim(), "1"));
    let (y, m): (i32, i32) = (y.parse().ok()?, m.parse().ok()?);
    (1..=12).contains(&m).then_some(y * 12 + m - 1)
}

/// Años totales de experiencia, fusionando periodos que se traslapan
/// (dos trabajos simultáneos no cuentan doble). Ignora fechas que no se puedan leer.
pub fn years_of_experience(master: &CvMaster, today: &str) -> f32 {
    let mut spans: Vec<(i32, i32)> = master
        .experience
        .iter()
        .filter_map(|e| {
            let start = month_index(&e.start)?;
            let end = month_index(e.end.as_deref().unwrap_or(today))?;
            (end >= start).then_some((start, end))
        })
        .collect();
    spans.sort_unstable();

    let mut months = 0;
    let mut current: Option<(i32, i32)> = None;
    for (s, e) in spans {
        current = match current {
            Some((cs, ce)) if s <= ce => Some((cs, ce.max(e))),
            Some((cs, ce)) => {
                months += ce - cs;
                Some((s, e))
            }
            None => Some((s, e)),
        };
    }
    if let Some((cs, ce)) = current {
        months += ce - cs;
    }
    months as f32 / 12.0
}

/// 0 = Jr, 1 = Mid, 2 = Sr, 3 = Lead.
fn candidate_level(years: f32) -> u8 {
    match years {
        y if y < 2.0 => 0,
        y if y < 5.0 => 1,
        y if y < 8.0 => 2,
        _ => 3,
    }
}

/// Nivel pedido según el título/seniority. Se revisa "semi senior" antes que "senior".
fn job_level(text: &str) -> Option<u8> {
    const LEVELS: &[(u8, &[&str])] = &[
        (
            1,
            &[
                "semi senior",
                "semisenior",
                "semi-senior",
                "ssr",
                "mid",
                "middle",
                "mid-level",
            ],
        ),
        (
            0,
            &[
                "jr",
                "junior",
                "trainee",
                "intern",
                "internship",
                "practicante",
                "becario",
                "entry level",
                "entry-level",
            ],
        ),
        (3, &["lead", "lider", "principal", "staff", "head"]),
        (2, &["sr", "senior"]),
    ];
    let n = normalize(text);
    LEVELS
        .iter()
        .find(|(_, words)| words.iter().any(|w| contains_word(&n, w)))
        .map(|(lvl, _)| *lvl)
}

// ---------------------------------------------------------------------------
// Modalidad, región y bloqueantes
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkMode {
    Remote,
    Hybrid,
    Onsite,
    Unknown,
}

const REMOTE_MARKERS: &[&str] = &[
    "remote",
    "remoto",
    "remota",
    "anywhere",
    "teletrabajo",
    "home office",
    "work from home",
    "trabajo desde casa",
];
const HYBRID_MARKERS: &[&str] = &["hibrido", "hibrida", "hybrid"];
const ONSITE_MARKERS: &[&str] = &[
    "presencial",
    "on-site",
    "onsite",
    "on site",
    "en oficina",
    "in office",
    "in-office",
];
const REGION_MARKERS: &[&str] = &[
    "latam",
    "latin america",
    "america latina",
    "latinoamerica",
    "mexico",
    "worldwide",
    "anywhere",
    "global",
];
const BLOCKER_MARKERS: &[&str] = &[
    "us only",
    "usa only",
    "us-only",
    "us based only",
    "must be based in the us",
    "must be located in the us",
    "us citizens",
    "us citizenship",
    "green card",
    "security clearance",
    "eu only",
    "europe only",
    "uk only",
    "north america only",
    "ciudadania estadounidense",
];

fn work_mode(text_norm: &str) -> WorkMode {
    let has = |list: &[&str]| list.iter().any(|m| contains_word(text_norm, m));
    // Si dice "remoto" y no exige oficina, es remoto aunque mencione "híbrido" de pasada.
    if has(REMOTE_MARKERS) && !has(ONSITE_MARKERS) {
        WorkMode::Remote
    } else if has(HYBRID_MARKERS) {
        WorkMode::Hybrid
    } else if has(ONSITE_MARKERS) {
        WorkMode::Onsite
    } else {
        WorkMode::Unknown
    }
}

fn remote_score(pref: RemotePref, mode: WorkMode) -> f32 {
    use WorkMode::*;
    let fraction = match (pref, mode) {
        (RemotePref::Cualquiera, _) | (_, Remote) => 1.0,
        (_, Unknown) => 0.5,
        (RemotePref::RemotoOHibrido, Hybrid) => 0.67,
        (RemotePref::RemotoOHibrido, Onsite) | (RemotePref::SoloRemoto, Hybrid | Onsite) => 0.0,
    };
    MAX_REMOTE * fraction
}

/// Palabras de nivel de idioma (normalizadas) → nivel MCER: 1 = A1 … 6 = C2 o nativo.
const LANG_LEVELS: &[(&str, u8)] = &[
    ("a1", 1),
    ("basico", 1),
    ("basica", 1),
    ("basic", 1),
    ("beginner", 1),
    ("principiante", 1),
    ("a2", 2),
    ("elemental", 2),
    ("elementary", 2),
    ("b1", 3),
    ("intermedio", 3),
    ("intermedia", 3),
    ("intermediate", 3),
    ("b2", 4),
    ("intermedio alto", 4),
    ("intermedia alta", 4),
    ("upper intermediate", 4),
    ("upper-intermediate", 4),
    ("conversacional", 4),
    ("conversational", 4),
    ("c1", 5),
    ("avanzado", 5),
    ("avanzada", 5),
    ("advanced", 5),
    ("fluent", 5),
    ("fluency", 5),
    ("fluido", 5),
    ("fluida", 5),
    ("fluidez", 5),
    ("c2", 6),
    ("nativo", 6),
    ("nativa", 6),
    ("native", 6),
    ("bilingue", 6),
    ("bilingual", 6),
    ("lengua materna", 6),
    ("mother tongue", 6),
];

/// Nombre de cada nivel de [`LANG_LEVELS`] para mostrarlo.
const CEFR: [&str; 7] = ["", "A1", "A2", "B1", "B2", "C1", "C2"];

/// Nivel que pide la vacante para `lang`: en cada mención, la palabra de nivel más cercana dentro
/// de la misma frase (corta en `,` `.` `;` y saltos de línea), así «inglés avanzado y portugués
/// básico» da C1 para el inglés y A1 para el portugués. `None` si el anuncio no dice nivel.
fn job_language_level(dict: &SkillDict, description: &str, lang: &str) -> Option<u8> {
    let idx = dict.lookup(lang)?;
    let text = normalize(description);
    let cuts = [',', '.', ';', '\n'];
    dict.find(&text)
        .iter()
        .filter(|h| h.skill == idx)
        .filter_map(|h| {
            let (s, e) = (h.start, h.start + h.surface.len());
            let from = text[..s].rfind(cuts).map_or(0, |i| i + 1);
            let to = text[e..].find(cuts).map_or(text.len(), |i| e + i);
            LANG_LEVELS
                .iter()
                .flat_map(|&(w, level)| {
                    text[from..to]
                        .match_indices(w)
                        .map(move |(i, _)| (from + i, from + i + w.len(), level))
                })
                .filter(|&(a, b, _)| at_word_boundary(&text, a, b) && (b <= s || a >= e))
                // La más cercana; si empatan («intermedio» e «intermedio alto»), la más alta.
                .min_by_key(|&(a, b, level)| (if b <= s { s - b } else { a - e }, u8::MAX - level))
                .map(|(_, _, level)| level)
        })
        .max()
}

/// Nivel de `lang` según la sección de idiomas del CV («B2», «Avanzado», «Nativo»…).
fn cv_language_level(dict: &SkillDict, master: &CvMaster, lang: &str) -> Option<u8> {
    let idx = dict.lookup(lang);
    let lang_norm = normalize(lang);
    let entry = master.languages.iter().find(|l| {
        contains_word(&normalize(&l.name), &lang_norm)
            || dict.find(&l.name).iter().any(|h| Some(h.skill) == idx)
    })?;
    let text = normalize(&format!("{} {}", entry.name, entry.level));
    LANG_LEVELS
        .iter()
        .filter(|(w, _)| contains_word(&text, w))
        .map(|&(_, level)| level)
        .max()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ats::keywords::extract_terms;
    use crate::model::Experience;

    fn bd(skills: f32, title: f32) -> MatchBreakdown {
        MatchBreakdown {
            skills,
            semantic: None,
            title,
            seniority: 3.0,
            years: 6.0,
            logistics: 10.0,
        }
    }

    #[test]
    fn reponderar_con_los_pesos_normales_no_cambia_nada() {
        let r = MatchReport {
            total: capped_total(&bd(30.0, 8.0), MAX_SEMANTIC, false, false),
            breakdown: bd(30.0, 8.0),
            years_experience: 3.0,
            missing_required: vec![],
            blockers: vec![],
            low_relevance: false,
            max: Weights::default(),
        };
        assert_eq!(reweigh(&r, &Weights::default()), r);
        // Con más peso a habilidades (que van al 67 %) y menos a puesto (al 80 %), el total baja.
        let w = Weights {
            skills: 55.0,
            title: 0.0,
            ..Weights::default()
        };
        let r2 = reweigh(&r, &w);
        assert_eq!(r2.max, w);
        assert!((r2.breakdown.skills - 30.0 * 55.0 / 45.0).abs() < 1e-4);
        assert_eq!(r2.breakdown.title, 0.0);
        // Los topes se respetan: con un bloqueo nunca pasa de 20.
        let blocked = MatchReport {
            blockers: vec!["US only".into()],
            ..r
        };
        assert!(reweigh(&blocked, &w).total <= 20);
    }

    #[test]
    fn sin_datos_suficientes_no_sugiere_pesos() {
        let few: Vec<_> = (0..10).map(|i| (bd(40.0, 9.0), i % 2 == 0)).collect();
        assert_eq!(learn_weights(&few), None);
        let all_pos: Vec<_> = (0..40).map(|_| (bd(40.0, 9.0), true)).collect();
        assert_eq!(learn_weights(&all_pos), None);
    }

    #[test]
    fn sube_el_peso_de_lo_que_distingue_a_las_que_avanzaron() -> Result<(), &'static str> {
        // Las que avanzaron tenían habilidades altas; el puesto no distinguía.
        let mut s: Vec<_> = (0..20).map(|_| (bd(42.0, 8.0), true)).collect();
        s.extend((0..20).map(|_| (bd(15.0, 8.0), false)));
        let w = learn_weights(&s).ok_or("debería sugerir")?;
        let d = Weights::default();
        assert!(w.skills > d.skills, "habilidades debería subir: {w:?}");
        assert!(
            w.title < d.title,
            "al renormalizar, lo que no distingue baja: {w:?}"
        );
        let sum: f32 = w.parts().iter().sum();
        assert!((sum - 100.0).abs() < 1e-3);
        assert!(w.skills <= d.skills * 1.5 * 100.0 / 85.0); // tope de +50 % (antes de renormalizar)
        Ok(())
    }

    type R = Result<(), Box<dyn std::error::Error>>;
    const TODAY: &str = "2026-11";

    fn master() -> Result<CvMaster, serde_json::Error> {
        serde_json::from_str(include_str!("../tests/fixtures/cv_editor_video.json"))
    }

    fn profile() -> Profile {
        Profile {
            id: "p1".into(),
            name: "Editor de video".into(),
            target_titles: vec!["Editor de video".into()],
            remote_pref: RemotePref::SoloRemoto,
            locations: vec![],
            created_at: "2026-11-05".into(),
        }
    }

    fn job(title: &str, description: &str) -> Job {
        Job {
            id: "j1".into(),
            source_id: "test".into(),
            external_id: None,
            url: "https://example.com".into(),
            company: None,
            title: title.into(),
            location: None,
            remote_type: None,
            salary_min: None,
            salary_max: None,
            currency: None,
            description: description.into(),
            language: None,
            requirements: None,
            posted_at: None,
            found_at: "2026-11-05".into(),
            dedupe_hash: "h".into(),
        }
    }

    /// Evalúa con los términos extraídos por el diccionario.
    fn evaluate(
        job: &Job,
        profile: &Profile,
        master: &CvMaster,
        semantic: Option<f32>,
    ) -> Result<MatchReport, Box<dyn std::error::Error>> {
        let dict = SkillDict::builtin()?;
        let req = JobRequirements {
            terms: extract_terms(&dict, &job.title, &job.description),
            ..Default::default()
        };
        Ok(match_score(
            &dict,
            &MatchInput {
                profile,
                master,
                job,
                requirements: &req,
                semantic,
                today: TODAY,
            },
        ))
    }

    /// Cuánto tarda puntuar 500 vacantes (lo que hacen la lista y la cola de Postular). A mano:
    /// `cargo test -p finder-core --release banco_puntaje -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn banco_puntaje() -> R {
        let dict = SkillDict::builtin()?;
        let (p, m) = (profile(), master()?);
        let j = job(
            "Editor de Video (Remoto - LATAM)",
            include_str!("../tests/fixtures/jd_editor_video_es.txt"),
        );
        let t = std::time::Instant::now();
        for _ in 0..500 {
            let req = JobRequirements {
                terms: extract_terms(&dict, &j.title, &j.description),
                ..Default::default()
            };
            let _ = match_score(
                &dict,
                &MatchInput {
                    profile: &p,
                    master: &m,
                    job: &j,
                    requirements: &req,
                    semantic: None,
                    today: TODAY,
                },
            );
        }
        println!("500 puntajes: {:.1?}", t.elapsed());
        Ok(())
    }

    #[test]
    fn editor_vs_vacante_de_editor_es_alta() -> R {
        let j = job(
            "Editor de Video (Remoto - LATAM)",
            include_str!("../tests/fixtures/jd_editor_video_es.txt"),
        );
        let r = evaluate(&j, &profile(), &master()?, None)?;
        assert!(r.blockers.is_empty(), "{:?}", r.blockers);
        assert!(!r.low_relevance);
        assert_eq!(r.breakdown.title, MAX_TITLE);
        assert_eq!(
            r.breakdown.logistics,
            MAX_REMOTE + MAX_REGION + MAX_LANGUAGES
        );
        assert!(r.missing_required.contains(&"TikTok".to_string()));
        assert!(r.total >= 60, "total = {}", r.total);
        Ok(())
    }

    #[test]
    fn editor_vs_vacante_de_frontend_es_baja() -> R {
        let j = job(
            "Junior Frontend Developer",
            include_str!("../tests/fixtures/jd_frontend_jr_en.txt"),
        );
        let r = evaluate(&j, &profile(), &master()?, None)?;
        assert!(r.low_relevance);
        assert!(r.total <= 30, "total = {}", r.total);
        Ok(())
    }

    #[test]
    fn bloqueante_limita_a_20() -> R {
        let j = job(
            "Video Editor (Remote)",
            "Remote, US only. Requirements: Adobe Premiere Pro, After Effects, video editing.",
        );
        let r = evaluate(&j, &profile(), &master()?, Some(0.9))?;
        assert!(r.total <= 20, "total = {}", r.total);
        assert!(r.blockers[0].contains("us only"));
        Ok(())
    }

    #[test]
    fn idioma_requerido_que_no_hablas_es_bloqueante() -> R {
        let j = job(
            "Editor de video remoto",
            "Requisitos: Premiere y portugués avanzado.",
        );
        let r = evaluate(&j, &profile(), &master()?, None)?;
        assert!(r.total <= 20);
        assert!(r.blockers[0].contains("Portugués"));
        Ok(())
    }

    #[test]
    fn sin_embeddings_se_reescala_a_100() -> R {
        let j = job(
            "Editor de Video (Remoto - LATAM)",
            include_str!("../tests/fixtures/jd_editor_video_es.txt"),
        );
        let (p, m) = (profile(), master()?);
        let sin = evaluate(&j, &p, &m, None)?;
        let perfecto = evaluate(&j, &p, &m, Some(1.0))?;
        let b = &sin.breakdown;
        let resto = b.skills + b.title + b.seniority + b.years + b.logistics;
        assert_eq!(sin.total, (resto * 100.0 / 85.0).round() as u8);
        assert_eq!(perfecto.total, (resto + MAX_SEMANTIC).round() as u8);
        Ok(())
    }

    #[test]
    fn anos_de_experiencia_fusiona_traslapes() -> R {
        let m = master()?;
        // 2020-06→2021-12 (18 meses) + 2022-01→2026-11 (58 meses) = 76 meses.
        assert!((years_of_experience(&m, TODAY) - 76.0 / 12.0).abs() < 1e-4);

        let exp = |start: &str, end: Option<&str>| Experience {
            id: "e".into(),
            company: "c".into(),
            title: "t".into(),
            start: start.into(),
            end: end.map(Into::into),
            bullets: vec![],
        };
        let traslape = CvMaster {
            experience: vec![
                exp("2020-01", Some("2022-01")),
                exp("2021-01", Some("2023-01")),
                exp("fecha rara", None),
            ],
            ..Default::default()
        };
        assert_eq!(years_of_experience(&traslape, TODAY), 3.0);
        Ok(())
    }

    #[test]
    fn seniority_y_anos_minimos() -> R {
        let dict = SkillDict::builtin()?;
        let (p, m) = (profile(), master()?);
        let j = job("Senior Video Editor (Remote)", "Remote LATAM");
        let req = JobRequirements {
            years_min: Some(10.0),
            ..Default::default()
        };
        let r = match_score(
            &dict,
            &MatchInput {
                profile: &p,
                master: &m,
                job: &j,
                requirements: &req,
                semantic: None,
                today: TODAY,
            },
        );
        // ~6.3 años → Sr (nivel 2): coincide con "Senior".
        assert_eq!(r.breakdown.seniority, MAX_SENIORITY);
        assert!((r.breakdown.years - MAX_YEARS * (76.0 / 12.0) / 10.0).abs() < 1e-3);

        assert_eq!(job_level("Semi Senior Developer"), Some(1));
        assert_eq!(job_level("Programador Jr."), Some(0));
        assert_eq!(job_level("Tech Lead"), Some(3));
        assert_eq!(job_level("Editor de video"), None);
        Ok(())
    }

    #[test]
    fn modalidad_segun_preferencia() {
        assert_eq!(
            work_mode(&normalize("Trabajo 100% remoto")),
            WorkMode::Remote
        );
        assert_eq!(
            work_mode(&normalize("Esquema híbrido en CDMX")),
            WorkMode::Hybrid
        );
        assert_eq!(
            work_mode(&normalize("Presencial en Monterrey")),
            WorkMode::Onsite
        );
        assert_eq!(
            work_mode(&normalize("Remoto, pero presencial 1 vez al mes")),
            WorkMode::Onsite
        );
        assert_eq!(remote_score(RemotePref::SoloRemoto, WorkMode::Onsite), 0.0);
        assert_eq!(remote_score(RemotePref::SoloRemoto, WorkMode::Unknown), 3.0);
        assert_eq!(
            remote_score(RemotePref::Cualquiera, WorkMode::Onsite),
            MAX_REMOTE
        );
    }

    #[test]
    fn coseno_basico() {
        assert_eq!(cosine(&[1.0, 0.0], &[1.0, 0.0]), Some(1.0));
        assert_eq!(cosine(&[1.0, 0.0], &[0.0, 1.0]), Some(0.0));
        assert_eq!(cosine(&[1.0], &[1.0, 2.0]), None);
        assert_eq!(cosine(&[0.0, 0.0], &[1.0, 1.0]), None);
        let long = "á".repeat(10_000);
        assert_eq!(
            truncate_chars(&long, SEMANTIC_MAX_CHARS).chars().count(),
            SEMANTIC_MAX_CHARS
        );
    }

    #[test]
    fn coseno_se_reescala() {
        assert_eq!(rescale_cosine(0.1), 0.0);
        assert_eq!(rescale_cosine(0.55), 0.5);
        assert_eq!(rescale_cosine(0.95), 1.0);
    }

    #[test]
    fn nivel_de_idioma_menor_al_pedido_cuenta_en_proporcion() -> R {
        let (p, m) = (profile(), master()?); // el CV dice «Inglés: B2»
        let con = |d: &str| evaluate(&job("Editor de video remoto", d), &p, &m, None);
        let pide_c1 = con("Requisitos: Premiere, inglés avanzado (C1).")?;
        assert!(pide_c1.blockers.is_empty());
        assert!(
            pide_c1
                .missing_required
                .contains(&"Inglés C1 (tu CV dice B2)".to_string())
        );
        let pide_b2 = con("Requisitos: Premiere, inglés intermedio alto.")?;
        let sin_nivel = con("Requisitos: Premiere, inglés.")?;
        assert!(
            pide_b2
                .missing_required
                .iter()
                .all(|s| !s.starts_with("Inglés"))
        );
        assert_eq!(pide_b2.breakdown.logistics, sin_nivel.breakdown.logistics);
        // 4/5 de los puntos de idioma.
        let diff = sin_nivel.breakdown.logistics - pide_c1.breakdown.logistics;
        assert!((diff - MAX_LANGUAGES / 5.0).abs() < 1e-4, "diff = {diff}");
        Ok(())
    }

    #[test]
    fn nivel_de_idioma_toma_la_palabra_mas_cercana() -> R {
        let dict = SkillDict::builtin()?;
        let nivel = |d: &str, l: &str| job_language_level(&dict, d, l);
        assert_eq!(
            nivel("Inglés avanzado y portugués básico.", "Inglés"),
            Some(5)
        );
        assert_eq!(
            nivel("Inglés avanzado y portugués básico.", "Portugués"),
            Some(1)
        );
        assert_eq!(nivel("Fluent English and basic Spanish", "Inglés"), Some(5));
        assert_eq!(nivel("Upper-intermediate English", "Inglés"), Some(4));
        assert_eq!(nivel("Inglés: intermedio alto", "Inglés"), Some(4));
        // «avanzado» es de After Effects, en otra parte de la lista.
        assert_eq!(nivel("After Effects avanzado, inglés", "Inglés"), None);
        assert_eq!(nivel("Se valora inglés", "Inglés"), None);
        Ok(())
    }
}
