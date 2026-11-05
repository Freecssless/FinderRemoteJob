//! Combina los términos del diccionario con lo que detecta la IA en una vacante.
//!
//! El diccionario es determinista; la IA aporta lo que el diccionario no sabe: si cada término es
//! requerido o deseable, habilidades fuera del diccionario, seniority, años mínimos, idiomas y
//! bloqueantes. Todo lo que propone la IA se **verifica contra el texto de la vacante**: lo que no
//! esté respaldado por el anuncio se descarta.

use serde::{Deserialize, Serialize};

use super::keywords::{SkillDict, contains_word, normalize};
use crate::guard::numbers;
use crate::model::{JobRequirements, Term, TermKind};

/// Respuesta de la IA (esquema en `finder_infra::llm::tasks`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LlmRequirements {
    /// Clasificación de los términos que ya encontró el diccionario (por nombre canónico).
    #[serde(default)]
    pub terms: Vec<LlmTerm>,
    /// Habilidades que el diccionario no conoce, con la forma exacta del anuncio.
    #[serde(default)]
    pub extra_skills: Vec<LlmExtraSkill>,
    #[serde(default)]
    pub seniority: Option<String>,
    #[serde(default)]
    pub years_min: Option<f32>,
    #[serde(default)]
    pub languages: Vec<String>,
    #[serde(default)]
    pub blockers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LlmTerm {
    pub canonical: String,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LlmExtraSkill {
    pub surface: String,
    pub required: bool,
    pub kind: TermKind,
}

const SENIORITY: &[&str] = &["jr", "mid", "sr", "lead"];

/// `terms` son los del diccionario (`extract_terms`); `llm` lo que respondió la IA.
pub fn merge(
    dict: &SkillDict,
    title: &str,
    description: &str,
    mut terms: Vec<Term>,
    llm: LlmRequirements,
) -> JobRequirements {
    let text = format!("{title}\n{description}");
    let text_norm = normalize(&text);
    let title_norm = normalize(title);
    let job_skills: Vec<usize> = dict.find(&text).into_iter().map(|h| h.skill).collect();
    let title_skills: Vec<usize> = dict.find(title).into_iter().map(|h| h.skill).collect();

    // 1. Requerido/deseable según la IA: se cambia la base del peso (3 ↔ 1) y se conserva el resto.
    for lt in &llm.terms {
        if let Some(t) = terms
            .iter_mut()
            .find(|t| normalize(&t.canonical) == normalize(&lt.canonical))
        {
            // Lo que aparece en el título siempre es requerido.
            let in_title = contains_word(&title_norm, &normalize(&t.surface))
                || dict
                    .lookup(&t.canonical)
                    .is_some_and(|i| title_skills.contains(&i));
            let required = lt.required || in_title;
            let base = |r: bool| if r { 3.0 } else { 1.0 };
            t.weight += base(required) - base(t.required);
            t.required = required;
        }
    }

    // 2. Habilidades extra: solo si están en el anuncio y no las cubre ya el diccionario.
    for s in llm.extra_skills {
        let surface = s.surface.trim().to_string();
        let n = normalize(&surface);
        if surface.is_empty()
            || !contains_word(&text_norm, &n)
            || terms
                .iter()
                .any(|t| normalize(&t.surface) == n || normalize(&t.canonical) == n)
            || dict
                .lookup(&surface)
                .is_some_and(|i| job_skills.contains(&i))
        {
            continue;
        }
        let in_title = contains_word(&title_norm, &n);
        terms.push(Term {
            canonical: surface.clone(),
            surface,
            kind: s.kind,
            required: s.required || in_title,
            weight: if s.required || in_title { 3.0 } else { 1.0 }
                + if in_title { 2.0 } else { 0.0 },
        });
    }

    // 3. Datos sueltos, cada uno verificado contra el anuncio.
    let seniority = llm
        .seniority
        .map(|s| normalize(&s))
        .filter(|s| SENIORITY.contains(&s.as_str()));
    let job_numbers = numbers(&text);
    let years_min = llm.years_min.filter(|y| {
        *y > 0.0
            && job_numbers
                .iter()
                .any(|n| n.parse::<f32>().is_ok_and(|v| v == *y))
    });
    let languages = llm
        .languages
        .into_iter()
        .filter(|l| {
            contains_word(&text_norm, &normalize(l))
                || dict.lookup(l).is_some_and(|i| job_skills.contains(&i))
        })
        .collect();
    // Un bloqueante limita la compatibilidad a 20: se exige que la mayoría de sus palabras estén en el anuncio.
    let blockers = llm
        .blockers
        .into_iter()
        .filter(|b| {
            let n = normalize(b);
            let words: Vec<&str> = n
                .split(|c: char| !c.is_alphanumeric())
                .filter(|w| w.len() >= 2)
                .collect();
            !words.is_empty()
                && words
                    .iter()
                    .filter(|w| contains_word(&text_norm, w))
                    .count()
                    * 2
                    > words.len()
        })
        .collect();

    JobRequirements {
        terms,
        seniority,
        years_min,
        languages,
        blockers,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ats::keywords::{DictError, extract_terms};

    const JD: &str = include_str!("../../tests/fixtures/jd_editor_video_es.txt");
    const TITLE: &str = "Editor de Video (Remoto - LATAM)";

    fn setup() -> Result<(SkillDict, Vec<Term>), DictError> {
        let dict = SkillDict::builtin()?;
        let terms = extract_terms(&dict, TITLE, JD);
        Ok((dict, terms))
    }

    fn term<'a>(r: &'a JobRequirements, canonical: &str) -> Option<&'a Term> {
        r.terms.iter().find(|t| t.canonical == canonical)
    }

    #[test]
    fn la_ia_reclasifica_requerido_y_deseable() -> Result<(), DictError> {
        let (dict, terms) = setup()?;
        let llm = LlmRequirements {
            terms: vec![
                LlmTerm {
                    canonical: "TikTok".into(),
                    required: false,
                },
                LlmTerm {
                    canonical: "Edición de video".into(),
                    required: false,
                },
            ],
            ..Default::default()
        };
        let r = merge(&dict, TITLE, JD, terms, llm);
        let tiktok = term(&r, "TikTok").ok_or_else(|| DictError::Parse {
            file: "t".into(),
            line: 0,
            msg: "falta".into(),
        })?;
        assert!(!tiktok.required);
        assert_eq!(tiktok.weight, 1.0);
        // Está en el título: sigue siendo requerido aunque la IA diga lo contrario.
        assert!(term(&r, "Edición de video").is_some_and(|t| t.required));
        Ok(())
    }

    #[test]
    fn habilidades_extra_solo_si_estan_en_el_anuncio() -> Result<(), DictError> {
        let (dict, terms) = setup()?;
        let llm = LlmRequirements {
            extra_skills: vec![
                LlmExtraSkill {
                    surface: "marketing digital".into(),
                    required: false,
                    kind: TermKind::Hard,
                },
                LlmExtraSkill {
                    surface: "Nuke".into(),
                    required: true,
                    kind: TermKind::Tool,
                },
                LlmExtraSkill {
                    surface: "Premiere".into(),
                    required: true,
                    kind: TermKind::Tool,
                },
            ],
            ..Default::default()
        };
        let before = terms.len();
        let r = merge(&dict, TITLE, JD, terms, llm);
        assert!(term(&r, "marketing digital").is_some());
        assert!(term(&r, "Nuke").is_none(), "no está en el anuncio");
        assert_eq!(
            r.terms.len(),
            before + 1,
            "Premiere ya lo cubre el diccionario"
        );
        Ok(())
    }

    #[test]
    fn datos_sueltos_se_verifican_contra_el_anuncio() -> Result<(), DictError> {
        let (dict, terms) = setup()?;
        let llm = LlmRequirements {
            seniority: Some("SR".into()),
            years_min: Some(3.0),
            languages: vec!["Inglés".into(), "Francés".into()],
            blockers: vec!["Solo residentes de Canadá".into(), "pago en USD".into()],
            ..Default::default()
        };
        let r = merge(&dict, TITLE, JD, terms.clone(), llm);
        assert_eq!(r.seniority.as_deref(), Some("sr"));
        assert_eq!(r.years_min, Some(3.0), "«3+ años» está en el anuncio");
        assert_eq!(r.languages, ["Inglés"]);
        assert_eq!(
            r.blockers,
            ["pago en USD"],
            "el de Canadá no está en el anuncio"
        );

        let inventado = LlmRequirements {
            years_min: Some(7.0),
            seniority: Some("experto".into()),
            ..Default::default()
        };
        let r = merge(&dict, TITLE, JD, terms, inventado);
        assert_eq!((r.years_min, r.seniority), (None, None));
        Ok(())
    }
}
