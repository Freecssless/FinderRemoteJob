//! Puntaje ATS de un CV concreto contra una vacante (0–100), según §4.4 B del plan:
//!
//! | Componente           | Peso |
//! |----------------------|------|
//! | Palabras clave       | 60   |
//! | Título               | 10   |
//! | Secciones y contacto | 10   |
//! | Legibilidad          | 20   |
//!
//! Recibe el **texto plano** del CV. En el módulo 8 la legibilidad se completa con la prueba de
//! ida y vuelta del PDF real; aquí se usan heurísticas sobre el texto.

use std::collections::HashMap;

use serde::Serialize;

use super::keywords::{SkillDict, contains_word, normalize};
use crate::model::{CvMaster, Term};

const MAX_KEYWORDS: f32 = 60.0;
const MAX_TITLE: f32 = 10.0;
const MAX_SECTIONS: f32 = 10.0;
const MAX_LEGIBILITY: f32 = 20.0;

/// Crédito cuando el CV tiene la habilidad pero con otro nombre ("Premiere" vs "Adobe Premiere Pro").
/// Algunos ATS comparan literal, así que la forma exacta del anuncio vale más.
const SYNONYM_CREDIT: f32 = 0.7;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AtsReport {
    /// 0–100.
    pub total: u8,
    pub breakdown: Breakdown,
    /// Términos encontrados con la forma exacta del anuncio.
    pub matched: Vec<String>,
    /// Términos encontrados solo como sinónimo: conviene usar la forma exacta.
    pub synonym_only: Vec<SynonymHit>,
    /// Términos que no aparecen en el CV.
    pub missing: Vec<Term>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Breakdown {
    pub keywords: f32,
    pub title: f32,
    pub sections: f32,
    pub legibility: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SynonymHit {
    /// Forma del anuncio.
    pub job_surface: String,
    /// Forma que aparece en el CV.
    pub cv_surface: String,
}

/// Calcula el puntaje ATS de `cv_text` para una vacante con `job_title` y sus `terms`.
pub fn ats_score(dict: &SkillDict, cv_text: &str, job_title: &str, terms: &[Term]) -> AtsReport {
    let mut suggestions = Vec::new();
    let cv_norm = normalize(cv_text);

    // --- Palabras clave ---------------------------------------------------
    // Primera forma en que aparece cada habilidad del diccionario dentro del CV.
    let mut cv_skills: HashMap<usize, &str> = HashMap::new();
    for hit in dict.find(cv_text) {
        cv_skills.entry(hit.skill).or_insert(hit.surface);
    }

    let (mut got, mut total_w) = (0.0f32, 0.0f32);
    let (mut matched, mut synonym_only, mut missing) = (Vec::new(), Vec::new(), Vec::new());
    for t in terms {
        total_w += t.weight;
        if contains_word(&cv_norm, &normalize(&t.surface)) {
            got += t.weight;
            matched.push(t.surface.clone());
        } else if let Some(cv_surface) = dict.lookup(&t.canonical).and_then(|i| cv_skills.get(&i)) {
            got += t.weight * SYNONYM_CREDIT;
            suggestions.push(format!(
                "Escribe «{}» tal como aparece en la vacante (tu CV dice «{}»).",
                t.surface, cv_surface
            ));
            synonym_only.push(SynonymHit {
                job_surface: t.surface.clone(),
                cv_surface: cv_surface.to_string(),
            });
        } else {
            missing.push(t.clone());
        }
    }
    let keywords = if total_w > 0.0 {
        MAX_KEYWORDS * got / total_w
    } else {
        MAX_KEYWORDS
    };
    for t in missing.iter().filter(|t| t.required) {
        suggestions.push(format!(
            "Falta «{}», un término importante en la vacante.",
            t.surface
        ));
    }

    // --- Resto de componentes ---------------------------------------------
    let title = title_score(cv_text, job_title);
    if title < MAX_TITLE {
        suggestions.push(format!(
            "Alinea tu titular con el puesto: «{job_title}» (si es fiel a tu experiencia)."
        ));
    }
    let (sections, mut sec_tips) = sections_score(cv_text);
    suggestions.append(&mut sec_tips);
    let (legibility, mut leg_tips) = legibility_score(cv_text);
    suggestions.append(&mut leg_tips);

    let sum = keywords + title + sections + legibility;
    AtsReport {
        total: sum.round().clamp(0.0, 100.0) as u8,
        breakdown: Breakdown {
            keywords,
            title,
            sections,
            legibility,
        },
        matched,
        synonym_only,
        missing,
        suggestions,
    }
}

// ---------------------------------------------------------------------------
// Título
// ---------------------------------------------------------------------------

/// Palabras del título de la vacante que no aportan (modalidad, nivel, conectores).
const TITLE_STOPWORDS: &[&str] = &[
    "de", "del", "en", "la", "el", "y", "e", "o", "para", "con", "the", "of", "and", "for", "a",
    "remoto", "remota", "remote", "hibrido", "hybrid", "latam", "mexico", "jr", "sr", "junior",
    "senior", "ssr", "semi", "mid", "home", "office",
];

fn tokens(norm: &str) -> impl Iterator<Item = &str> {
    norm.split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 2 && !TITLE_STOPWORDS.contains(w))
}

/// Fracción de las palabras del puesto que aparecen en la cabecera del CV (primeras 5 líneas).
fn title_score(cv_text: &str, job_title: &str) -> f32 {
    let header: String = cv_text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .take(5)
        .collect::<Vec<_>>()
        .join("\n");
    let header = normalize(&header);
    let header_words: Vec<&str> = tokens(&header).collect();

    let job = normalize(job_title);
    let wanted: Vec<&str> = tokens(&job).collect();
    if wanted.is_empty() {
        return MAX_TITLE;
    }
    let found = wanted.iter().filter(|w| header_words.contains(w)).count();
    MAX_TITLE * found as f32 / wanted.len() as f32
}

// ---------------------------------------------------------------------------
// Secciones y contacto
// ---------------------------------------------------------------------------

/// (nombre a sugerir, variantes aceptadas ya normalizadas)
const SECTIONS: &[(&str, &[&str])] = &[
    (
        "Experiencia profesional",
        &[
            "experiencia",
            "experiencia profesional",
            "experiencia laboral",
            "experience",
            "professional experience",
            "work experience",
        ],
    ),
    (
        "Educación",
        &["educacion", "formacion", "formacion academica", "education"],
    ),
    (
        "Habilidades",
        &[
            "habilidades",
            "habilidades tecnicas",
            "competencias",
            "conocimientos",
            "aptitudes",
            "skills",
            "technical skills",
        ],
    ),
];

fn is_heading(line_norm: &str, variants: &[&str]) -> bool {
    let l = line_norm.trim().trim_end_matches(':').trim();
    variants.contains(&l)
}

fn has_email(text: &str) -> bool {
    text.split_whitespace().any(|w| {
        w.split_once('@')
            .is_some_and(|(user, domain)| !user.is_empty() && domain.contains('.'))
    })
}

/// Un teléfono: tramo de dígitos y separadores (`+ ( ) - .` y espacios) con al menos 10 dígitos.
/// Las fechas tipo "2020-06 – 2021-12" no cuentan porque el guion largo corta el tramo.
fn has_phone(text: &str) -> bool {
    text.split(|c: char| !(c.is_ascii_digit() || " +()-.".contains(c)))
        .any(|run| run.chars().filter(char::is_ascii_digit).count() >= 10)
}

/// 5 elementos (3 encabezados + correo + teléfono), cada uno vale 1/5 de `MAX_SECTIONS`.
fn sections_score(cv_text: &str) -> (f32, Vec<String>) {
    const ITEM: f32 = MAX_SECTIONS / 5.0;
    let mut score = 0.0;
    let mut tips = Vec::new();
    let lines: Vec<String> = cv_text.lines().map(normalize).collect();
    for (name, variants) in SECTIONS {
        if lines.iter().any(|l| is_heading(l, variants)) {
            score += ITEM;
        } else {
            tips.push(format!(
                "Agrega una sección con el encabezado estándar «{name}»."
            ));
        }
    }
    if has_email(cv_text) {
        score += ITEM;
    } else {
        tips.push("Agrega tu correo electrónico en el cuerpo del CV.".into());
    }
    if has_phone(cv_text) {
        score += ITEM;
    } else {
        tips.push("Agrega tu teléfono con lada (ej. +52 55 1234 5678).".into());
    }
    (score, tips)
}

// ---------------------------------------------------------------------------
// Legibilidad (heurísticas de texto; la prueba con el PDF real llega en el módulo 8)
// ---------------------------------------------------------------------------

const MAX_WORDS: usize = 1000; // ≈ 2 páginas
const MONTHS: &[&str] = &[
    "ene",
    "enero",
    "feb",
    "febrero",
    "mar",
    "marzo",
    "abr",
    "abril",
    "may",
    "mayo",
    "jun",
    "junio",
    "jul",
    "julio",
    "ago",
    "agosto",
    "sep",
    "sept",
    "septiembre",
    "oct",
    "octubre",
    "nov",
    "noviembre",
    "dic",
    "diciembre",
    "jan",
    "january",
    "february",
    "march",
    "apr",
    "april",
    "june",
    "july",
    "aug",
    "august",
    "september",
    "october",
    "november",
    "dec",
    "december",
];

#[derive(PartialEq, Eq, Clone, Copy)]
enum DateFmt {
    Slash,     // 03/2024
    Iso,       // 2024-03
    MonthName, // mar 2024
}

fn is_digits(s: &str, n: usize) -> bool {
    s.len() == n && s.chars().all(|c| c.is_ascii_digit())
}

fn date_formats(cv_text: &str) -> Vec<DateFmt> {
    let norm = normalize(cv_text);
    let toks: Vec<&str> = norm
        .split(|c: char| c.is_whitespace() || ",;()|".contains(c))
        .filter(|t| !t.is_empty())
        .collect();
    let mut found = Vec::new();
    for (i, t) in toks.iter().enumerate() {
        let fmt = match (t.split_once('/'), t.split_once('-')) {
            (Some((m, y)), _) if (is_digits(m, 1) || is_digits(m, 2)) && is_digits(y, 4) => {
                Some(DateFmt::Slash)
            }
            (_, Some((y, m))) if is_digits(y, 4) && is_digits(m, 2) => Some(DateFmt::Iso),
            _ if MONTHS.contains(&t.trim_end_matches('.'))
                && toks.get(i + 1).is_some_and(|y| is_digits(y, 4)) =>
            {
                Some(DateFmt::MonthName)
            }
            _ => None,
        };
        if let Some(f) = fmt.filter(|f| !found.contains(f)) {
            found.push(f);
        }
    }
    found
}

/// Muchas líneas con huecos de 3+ espacios o tabuladores suelen venir de tablas o columnas.
fn looks_like_columns(cv_text: &str) -> bool {
    cv_text
        .lines()
        .filter(|l| {
            let t = l.trim();
            t.contains('\t') || t.contains("   ")
        })
        .count()
        >= 3
}

fn legibility_score(cv_text: &str) -> (f32, Vec<String>) {
    let mut score = MAX_LEGIBILITY;
    let mut tips = Vec::new();

    // U+FFFD = carácter de reemplazo; U+E000–U+F8FF = uso privado (íconos de fuentes).
    if cv_text
        .chars()
        .any(|c| c == '\u{FFFD}' || ('\u{E000}'..='\u{F8FF}').contains(&c))
    {
        score -= 8.0;
        tips.push("Hay caracteres ilegibles (suele pasar con íconos o fuentes especiales). Usa texto simple.".into());
    }
    if cv_text.split_whitespace().count() > MAX_WORDS {
        score -= 4.0;
        tips.push("El CV pasa de ~2 páginas; resume o quita lo menos relevante.".into());
    }
    if looks_like_columns(cv_text) {
        score -= 4.0;
        tips.push("Parece haber tablas o columnas; usa una sola columna.".into());
    }
    if date_formats(cv_text).len() > 1 {
        score -= 4.0;
        tips.push("Usa un solo formato de fecha en todo el CV (ej. «ene 2022 – mar 2024»).".into());
    }
    (score, tips)
}

// ---------------------------------------------------------------------------
// Huecos
// ---------------------------------------------------------------------------

/// Faltantes divididos según el CV maestro (§4.4):
/// - `with_evidence`: el maestro lo respalda pero el CV no lo muestra → la IA puede arreglarlo.
/// - `without_evidence`: no hay respaldo → hueco que decide el usuario, nunca se inventa.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Gaps {
    pub with_evidence: Vec<Term>,
    pub without_evidence: Vec<Term>,
}

pub fn classify_gaps(dict: &SkillDict, missing: &[Term], master: &CvMaster) -> Gaps {
    let mut parts = vec![master.basics.headline.as_str()];
    for e in &master.experience {
        parts.push(&e.title);
        parts.extend(e.bullets.iter().map(|b| b.text.as_str()));
    }
    parts.extend(master.skills.iter().map(|s| s.name.as_str()));
    parts.extend(master.certifications.iter().map(|c| c.name.as_str()));
    parts.extend(master.languages.iter().map(|l| l.name.as_str()));
    let text = parts.join("\n");

    let text_norm = normalize(&text);
    let skills: Vec<usize> = dict.find(&text).into_iter().map(|h| h.skill).collect();

    let (with_evidence, without_evidence) = missing.iter().cloned().partition(|t| {
        contains_word(&text_norm, &normalize(&t.surface))
            || dict
                .lookup(&t.canonical)
                .is_some_and(|i| skills.contains(&i))
    });
    Gaps {
        with_evidence,
        without_evidence,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ats::keywords::{DictError, extract_terms};

    const JD: &str = include_str!("../../tests/fixtures/jd_editor_video_es.txt");
    const JOB_TITLE: &str = "Editor de Video (Remoto - LATAM)";
    const CV: &str = include_str!("../../tests/fixtures/cv_editor_video.txt");

    type R = Result<(), DictError>;

    fn setup() -> Result<(SkillDict, Vec<Term>), DictError> {
        let dict = SkillDict::builtin()?;
        let terms = extract_terms(&dict, JOB_TITLE, JD);
        Ok((dict, terms))
    }

    #[test]
    fn cv_de_ejemplo_tiene_puntaje_coherente() -> R {
        let (dict, terms) = setup()?;
        let r = ats_score(&dict, CV, JOB_TITLE, &terms);

        assert!(r.matched.contains(&"Adobe Premiere Pro".to_string()));
        assert!(r.matched.contains(&"After Effects".to_string()));
        assert!(r.missing.iter().any(|t| t.canonical == "TikTok"));
        assert_eq!(r.breakdown.sections, 10.0);
        assert_eq!(
            r.breakdown.title, 10.0,
            "«Editor de video» está en la cabecera"
        );
        assert_eq!(r.breakdown.legibility, 20.0);
        assert!((40..90).contains(&r.total), "total = {}", r.total);
        Ok(())
    }

    #[test]
    fn forma_exacta_vale_mas_que_sinonimo() -> R {
        let (dict, terms) = setup()?;
        let premiere: Vec<Term> = terms
            .into_iter()
            .filter(|t| t.canonical == "Adobe Premiere Pro")
            .collect();

        let exacto = ats_score(&dict, "Uso Adobe Premiere Pro", "x", &premiere);
        let sinonimo = ats_score(&dict, "Uso Premiere", "x", &premiere);
        let nada = ats_score(&dict, "Uso Word", "x", &premiere);

        assert_eq!(exacto.breakdown.keywords, 60.0);
        assert!((sinonimo.breakdown.keywords - 60.0 * SYNONYM_CREDIT).abs() < 1e-3);
        assert_eq!(nada.breakdown.keywords, 0.0);
        assert_eq!(sinonimo.synonym_only[0].cv_surface, "Premiere");
        assert!(sinonimo.suggestions[0].contains("Adobe Premiere Pro"));
        Ok(())
    }

    #[test]
    fn agregar_palabras_clave_sube_el_puntaje() -> R {
        let (dict, terms) = setup()?;
        let antes = ats_score(&dict, CV, JOB_TITLE, &terms);
        let optimizado = CV.replace(
            "Adobe Premiere Pro, After Effects, DaVinci Resolve",
            "Adobe Premiere Pro, After Effects, DaVinci Resolve, edición de video, motion graphics, YouTube, TikTok",
        );
        let despues = ats_score(&dict, &optimizado, JOB_TITLE, &terms);
        assert!(
            despues.total > antes.total,
            "{} → {}",
            antes.total,
            despues.total
        );
        assert!(despues.missing.len() < antes.missing.len());
        Ok(())
    }

    #[test]
    fn sin_terminos_no_penaliza_palabras_clave() -> R {
        let dict = SkillDict::builtin()?;
        assert_eq!(
            ats_score(&dict, CV, "Editor de video", &[])
                .breakdown
                .keywords,
            60.0
        );
        Ok(())
    }

    #[test]
    fn detecta_secciones_y_contacto_faltantes() {
        let (s, tips) = sections_score("Juan\nHABILIDADES:\nExcel");
        assert_eq!(s, 2.0);
        assert_eq!(tips.len(), 4);
        assert!(has_phone("Tel. (55) 1234-5678 ext 12, +52"));
        assert!(!has_phone("2020-06 – 2021-12"));
        assert!(has_email("correo: ana.p@mail.mx"));
        assert!(!has_email("@usuario"));
    }

    #[test]
    fn legibilidad_penaliza_caracteres_rotos_columnas_y_fechas_mezcladas() {
        assert_eq!(legibility_score(CV).0, 20.0);
        assert_eq!(legibility_score("Editor \u{FFFD} video").0, 12.0);
        assert_eq!(legibility_score("ene 2022 – 03/2024").0, 16.0);
        assert_eq!(legibility_score("a   b\nc   d\ne\tf").0, 16.0);
    }

    #[test]
    fn titulo_ignora_modalidad_y_conectores() {
        assert_eq!(
            title_score("Ana\nVideo Editor", "Editor de Video (Remoto)"),
            10.0
        );
        assert_eq!(title_score("Ana\nEditor", "Editor de Video"), 5.0);
    }

    #[test]
    fn huecos_separa_lo_que_tiene_respaldo_en_el_maestro() -> Result<(), Box<dyn std::error::Error>>
    {
        let (dict, terms) = setup()?;
        let master: CvMaster =
            serde_json::from_str(include_str!("../../tests/fixtures/cv_editor_video.json"))?;
        let missing: Vec<Term> = terms
            .into_iter()
            .filter(|t| ["Edición de video", "TikTok"].contains(&t.canonical.as_str()))
            .collect();

        let g = classify_gaps(&dict, &missing, &master);
        // El titular "Editor de video" respalda la habilidad; TikTok no aparece en ningún lado.
        assert_eq!(g.with_evidence.len(), 1);
        assert_eq!(g.with_evidence[0].canonical, "Edición de video");
        assert_eq!(g.without_evidence[0].canonical, "TikTok");
        Ok(())
    }
}
