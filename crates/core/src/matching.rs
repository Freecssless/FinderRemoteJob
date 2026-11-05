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
//! Es *compatibilidad*, no probabilidad: los pesos se recalibrarán con datos reales en la fase 2.

use serde::Serialize;

use crate::ats::keywords::{SkillDict, contains_word, normalize};
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

    let wanted_langs: Vec<&str> = req
        .terms
        .iter()
        .filter(|t| t.kind == TermKind::Language && t.required)
        .map(|t| t.canonical.as_str())
        .chain(req.languages.iter().map(String::as_str))
        .collect();
    let missing_langs: Vec<&str> = wanted_langs
        .iter()
        .copied()
        .filter(|l| !ev.has_name(dict, l))
        .collect();
    let languages = if wanted_langs.is_empty() {
        MAX_LANGUAGES
    } else {
        MAX_LANGUAGES * (wanted_langs.len() - missing_langs.len()) as f32
            / wanted_langs.len() as f32
    };
    for l in &missing_langs {
        blockers.push(format!("Piden {l}, que no aparece en tu CV."));
    }
    let logistics = remote + region + languages;

    // --- Bloqueantes por texto del anuncio + los que haya detectado el LLM.
    for m in BLOCKER_MARKERS
        .iter()
        .filter(|m| contains_word(&job_text, m))
    {
        blockers.push(format!("La vacante dice «{m}»."));
    }
    blockers.extend(req.blockers.iter().cloned());

    // --- Total.
    let mut sum = skills + title + seniority + years + logistics;
    match semantic {
        Some(s) => sum += s,
        // Sin embeddings: los 85 puntos restantes se escalan a 100.
        None => sum *= 100.0 / (100.0 - MAX_SEMANTIC),
    }
    if low_relevance {
        sum = sum.min(LOW_RELEVANCE_CAP);
    }
    if !blockers.is_empty() {
        sum = sum.min(BLOCKED_CAP);
    }

    MatchReport {
        total: sum.round().clamp(0.0, 100.0) as u8,
        breakdown: MatchBreakdown {
            skills,
            semantic,
            title,
            seniority,
            years,
            logistics,
        },
        years_experience,
        missing_required,
        blockers,
        low_relevance,
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ats::keywords::extract_terms;
    use crate::model::Experience;

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
    }

    #[test]
    fn coseno_se_reescala() {
        assert_eq!(rescale_cosine(0.1), 0.0);
        assert_eq!(rescale_cosine(0.55), 0.5);
        assert_eq!(rescale_cosine(0.95), 1.0);
    }
}
