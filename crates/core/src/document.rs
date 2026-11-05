//! CV listo para imprimir: lo que reciben las plantillas de PDF y DOCX.
//!
//! Aquí se deciden, de forma determinista, el orden de secciones, los encabezados estándar
//! (en el idioma de la vacante) y el formato único de fechas. Las plantillas solo dibujan.
//! El contenido viene del CV maestro ([`CvDocument::from_master`]); el optimizador (módulo 9)
//! sustituye resumen, bullets y orden de habilidades por los ya validados por el guardián.

use serde::Serialize;

use crate::ats::keywords::normalize;
use crate::guard::GuardReport;
use crate::model::CvMaster;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DocLang {
    Es,
    En,
}

impl DocLang {
    /// Idioma de un texto (la vacante) por palabras frecuentes. Ante la duda, español.
    pub fn detect(text: &str) -> Self {
        const ES: &[&str] = &[
            "de",
            "la",
            "el",
            "en",
            "y",
            "para",
            "con",
            "experiencia",
            "requisitos",
            "que",
            "los",
            "las",
        ];
        const EN: &[&str] = &[
            "the",
            "and",
            "of",
            "to",
            "with",
            "for",
            "experience",
            "requirements",
            "you",
            "we",
            "our",
            "is",
        ];
        let n = normalize(text);
        let words: Vec<&str> = n.split(|c: char| !c.is_alphanumeric()).collect();
        let count = |list: &[&str]| words.iter().filter(|w| list.contains(w)).count();
        if count(EN) > count(ES) {
            DocLang::En
        } else {
            DocLang::Es
        }
    }
}

/// Decisiones del usuario en el diff. Por defecto se usa todo lo que aprobó el guardián.
#[derive(Debug, Clone, PartialEq, serde::Deserialize, Serialize)]
pub struct Choices {
    pub summary: bool,
    /// Una entrada por cada bullet de `GuardReport::bullets` (`true` = usar la propuesta).
    pub bullets: Vec<bool>,
}

impl Choices {
    pub fn accept_all(report: &GuardReport) -> Self {
        Self {
            summary: true,
            bullets: report.bullets.iter().map(|b| b.accepted()).collect(),
        }
    }
}

/// Encabezados estándar que los ATS reconocen (plan §4.6).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Headings {
    pub summary: &'static str,
    pub experience: &'static str,
    pub education: &'static str,
    pub skills: &'static str,
    pub certifications: &'static str,
    pub languages: &'static str,
}

impl Headings {
    pub fn for_lang(lang: DocLang) -> Self {
        match lang {
            DocLang::Es => Self {
                summary: "Resumen",
                experience: "Experiencia profesional",
                education: "Educación",
                skills: "Habilidades",
                certifications: "Certificaciones",
                languages: "Idiomas",
            },
            DocLang::En => Self {
                summary: "Summary",
                experience: "Professional Experience",
                education: "Education",
                skills: "Skills",
                certifications: "Certifications",
                languages: "Languages",
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CvDocument {
    pub lang: DocLang,
    pub headings: Headings,
    pub name: String,
    pub headline: String,
    /// Correo, teléfono, ubicación y enlaces, en el cuerpo (no en el encabezado de página).
    pub contact: Vec<String>,
    pub summary: Option<String>,
    pub experience: Vec<DocExperience>,
    pub skills: Vec<String>,
    pub education: Vec<DocEducation>,
    pub certifications: Vec<String>,
    pub languages: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DocExperience {
    pub title: String,
    pub company: String,
    pub dates: String,
    pub bullets: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DocEducation {
    pub degree: String,
    pub institution: String,
    pub dates: String,
}

impl CvDocument {
    /// El CV maestro tal cual, con la plantilla base (sirve de "antes" y de punto de partida).
    pub fn from_master(m: &CvMaster, lang: DocLang) -> Self {
        let b = &m.basics;
        let contact = [&b.email, &b.phone, &b.location]
            .into_iter()
            .chain(&b.links)
            .filter(|s| !s.trim().is_empty())
            .cloned()
            .collect();
        Self {
            lang,
            headings: Headings::for_lang(lang),
            name: b.name.clone(),
            headline: b.headline.clone(),
            contact,
            summary: None,
            experience: m
                .experience
                .iter()
                .map(|e| DocExperience {
                    title: e.title.clone(),
                    company: e.company.clone(),
                    dates: date_range(Some(&e.start), e.end.as_deref(), true, lang),
                    bullets: e.bullets.iter().map(|b| b.text.clone()).collect(),
                })
                .collect(),
            skills: m.skills.iter().map(|s| s.name.clone()).collect(),
            education: m
                .education
                .iter()
                .map(|e| DocEducation {
                    degree: e.degree.clone(),
                    institution: e.institution.clone(),
                    dates: date_range(e.start.as_deref(), e.end.as_deref(), false, lang),
                })
                .collect(),
            certifications: m
                .certifications
                .iter()
                .map(|c| {
                    let extra: Vec<&str> = [c.issuer.as_deref(), c.date.as_deref()]
                        .into_iter()
                        .flatten()
                        .collect();
                    if extra.is_empty() {
                        c.name.clone()
                    } else {
                        format!("{} — {}", c.name, extra.join(", "))
                    }
                })
                .collect(),
            languages: m
                .languages
                .iter()
                .map(|l| format!("{} ({})", l.name, l.level))
                .collect(),
        }
    }

    /// CV optimizado: el maestro con lo que el guardián aceptó y el usuario confirmó.
    ///
    /// - Resumen: el propuesto si se acepta; si no, ninguno.
    /// - Logros: por experiencia, en el orden propuesto; los rechazados vuelven a su texto original.
    ///   Si la propuesta no tocó una experiencia, se conservan sus logros originales.
    /// - Habilidades: primero las del orden propuesto y después las demás (se reordena, nunca se quita).
    /// - Empresa, puesto, fechas, educación e idiomas salen siempre del maestro.
    pub fn from_report(
        m: &CvMaster,
        report: &GuardReport,
        choices: &Choices,
        lang: DocLang,
    ) -> Self {
        let mut doc = Self::from_master(m, lang);
        doc.summary = report
            .summary
            .as_ref()
            .filter(|s| choices.summary && s.accepted())
            .map(|s| s.proposed.clone());

        for (exp, out) in m.experience.iter().zip(doc.experience.iter_mut()) {
            let mut proposed = report
                .bullets
                .iter()
                .enumerate()
                .filter(|(_, c)| c.exp_id.as_deref() == Some(exp.id.as_str()))
                .peekable();
            if proposed.peek().is_none() {
                continue;
            }
            let mut bullets: Vec<String> = Vec::new();
            for (i, c) in proposed {
                let use_proposed = c.accepted() && choices.bullets.get(i).copied().unwrap_or(true);
                let texts: Vec<&str> = if use_proposed {
                    vec![c.proposed.as_str()]
                } else {
                    c.originals.iter().map(String::as_str).collect()
                };
                for t in texts {
                    // Un original puede repetirse si dos propuestas citaban el mismo logro.
                    if !bullets.iter().any(|b| b == t) {
                        bullets.push(t.to_string());
                    }
                }
            }
            out.bullets = bullets;
        }

        let mut skills: Vec<String> = report
            .skills_order
            .iter()
            .filter_map(|id| m.skill(id))
            .map(|s| s.name.clone())
            .collect();
        for s in &m.skills {
            if !skills.contains(&s.name) {
                skills.push(s.name.clone());
            }
        }
        doc.skills = skills;
        doc
    }

    /// Texto plano en el mismo orden que las plantillas: lo que un ATS debería poder leer.
    pub fn to_plain_text(&self) -> String {
        let h = &self.headings;
        let mut out = vec![self.name.clone()];
        if !self.headline.is_empty() {
            out.push(self.headline.clone());
        }
        out.push(self.contact.join(" | "));
        if let Some(s) = &self.summary {
            out.extend([String::new(), h.summary.to_uppercase(), s.clone()]);
        }
        if !self.experience.is_empty() {
            out.extend([String::new(), h.experience.to_uppercase()]);
            for e in &self.experience {
                out.push(format!("{} — {}", e.title, e.company));
                out.push(e.dates.clone());
                out.extend(e.bullets.iter().map(|b| format!("- {b}")));
            }
        }
        if !self.education.is_empty() {
            out.extend([String::new(), h.education.to_uppercase()]);
            out.extend(
                self.education
                    .iter()
                    .map(|e| format!("{} — {}, {}", e.degree, e.institution, e.dates)),
            );
        }
        for (heading, items) in [
            (h.skills, &self.skills),
            (h.certifications, &self.certifications),
            (h.languages, &self.languages),
        ] {
            if !items.is_empty() {
                out.extend([String::new(), heading.to_uppercase(), items.join(", ")]);
            }
        }
        out.join("\n")
    }

    /// Encabezados de las secciones presentes, en orden (para la prueba de ida y vuelta).
    pub fn section_headings(&self) -> Vec<&'static str> {
        let h = &self.headings;
        [
            (h.summary, self.summary.is_some()),
            (h.experience, !self.experience.is_empty()),
            (h.education, !self.education.is_empty()),
            (h.skills, !self.skills.is_empty()),
            (h.certifications, !self.certifications.is_empty()),
            (h.languages, !self.languages.is_empty()),
        ]
        .into_iter()
        .filter_map(|(name, present)| present.then_some(name))
        .collect()
    }

    /// Nombre de archivo del plan: `Nombre_Apellido_Puesto_Empresa` (sin acentos ni símbolos).
    pub fn file_stem(&self, job_title: &str, company: Option<&str>) -> String {
        [self.name.as_str(), job_title, company.unwrap_or("")]
            .iter()
            .flat_map(|p| p.split_whitespace())
            .map(|w| {
                let clean: String = deunicode::deunicode(w)
                    .chars()
                    .filter(|c| c.is_ascii_alphanumeric())
                    .collect();
                let mut cs = clean.chars();
                cs.next()
                    .map(|f| f.to_ascii_uppercase().to_string() + cs.as_str())
                    .unwrap_or_default()
            })
            .filter(|w| !w.is_empty())
            .collect::<Vec<_>>()
            .join("_")
    }
}

const MONTHS_ES: [&str; 12] = [
    "ene", "feb", "mar", "abr", "may", "jun", "jul", "ago", "sep", "oct", "nov", "dic",
];
const MONTHS_EN: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// "2022-01" → "ene 2022"; "2016" → "2016"; otro formato se deja como está.
fn format_date(d: &str, lang: DocLang) -> String {
    match d.split_once('-') {
        Some((y, m)) => match m.parse::<usize>() {
            Ok(m @ 1..=12) => {
                let months = if lang == DocLang::Es {
                    MONTHS_ES
                } else {
                    MONTHS_EN
                };
                format!("{} {y}", months[m - 1])
            }
            _ => d.to_string(),
        },
        None => d.to_string(),
    }
}

/// Rango con un solo formato en todo el CV. `open_ended`: sin fin = "actual"/"Present".
fn date_range(start: Option<&str>, end: Option<&str>, open_ended: bool, lang: DocLang) -> String {
    let current = if lang == DocLang::Es {
        "actual"
    } else {
        "Present"
    };
    let start = start
        .filter(|s| !s.is_empty())
        .map(|s| format_date(s, lang));
    let end = match end.filter(|s| !s.is_empty()) {
        Some(e) => Some(format_date(e, lang)),
        None if open_ended => Some(current.to_string()),
        None => None,
    };
    match (start, end) {
        (Some(s), Some(e)) => format!("{s} – {e}"),
        (Some(s), None) => s,
        (None, Some(e)) => e,
        (None, None) => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ats::keywords::SkillDict;
    use crate::ats::score::ats_score;

    fn master() -> Result<CvMaster, serde_json::Error> {
        serde_json::from_str(include_str!("../tests/fixtures/cv_editor_video.json"))
    }

    #[test]
    fn cv_optimizado_respeta_guardian_y_decisiones() -> Result<(), Box<dyn std::error::Error>> {
        use crate::guard::{Draft, DraftBullet, DraftText, validate};
        let m = master()?;
        let dict = SkillDict::builtin()?;
        let b = |exp: &str, text: &str, src: &[&str]| DraftBullet {
            exp_id: exp.into(),
            text: text.into(),
            source_ids: src.iter().map(|s| s.to_string()).collect(),
        };
        let draft = Draft {
            summary: Some(DraftText {
                text: "Editor de video con 6 años de experiencia.".into(),
                source_ids: vec!["exp_01".into()],
            }),
            bullets: vec![
                b(
                    "exp_01",
                    "Reduje 30% el tiempo de entrega con plantillas de edición reutilizables",
                    &["b_02"],
                ),
                b("exp_01", "Edité 500 videos para YouTube", &["b_01"]), // cifra inventada → original
            ],
            skills_order: vec!["sk_03".into()],
        };
        let report = validate(&dict, &m, &draft, "2026-11");
        let all = Choices::accept_all(&report);
        assert_eq!(all.bullets, [true, false]);

        let doc = CvDocument::from_report(&m, &report, &all, DocLang::Es);
        assert_eq!(
            doc.summary.as_deref(),
            Some("Editor de video con 6 años de experiencia.")
        );
        assert_eq!(
            doc.experience[0].bullets,
            [
                "Reduje 30% el tiempo de entrega con plantillas de edición reutilizables",
                "Edité 120 videos para YouTube con Premiere y After Effects",
            ]
        );
        // La experiencia que la propuesta no tocó conserva sus logros.
        assert_eq!(
            doc.experience[1].bullets,
            ["Corrección de color y sonido para comerciales de TV"]
        );
        assert_eq!(
            doc.skills,
            ["DaVinci Resolve", "Adobe Premiere Pro", "After Effects"]
        );

        // El usuario rechaza el resumen y el primer cambio.
        let mine = Choices {
            summary: false,
            bullets: vec![false, false],
        };
        let doc = CvDocument::from_report(&m, &report, &mine, DocLang::Es);
        assert_eq!(doc.summary, None);
        assert_eq!(
            doc.experience[0].bullets[0],
            "Reduje el tiempo de entrega 30% creando plantillas de edición reutilizables"
        );
        Ok(())
    }

    #[test]
    fn fechas_uniformes_por_idioma() {
        assert_eq!(
            date_range(Some("2022-01"), None, true, DocLang::Es),
            "ene 2022 – actual"
        );
        assert_eq!(
            date_range(Some("2020-06"), Some("2021-12"), true, DocLang::En),
            "Jun 2020 – Dec 2021"
        );
        assert_eq!(
            date_range(Some("2016"), Some("2020"), false, DocLang::Es),
            "2016 – 2020"
        );
        assert_eq!(date_range(None, None, false, DocLang::Es), "");
    }

    #[test]
    fn desde_el_maestro_con_encabezados_estandar() -> Result<(), serde_json::Error> {
        let doc = CvDocument::from_master(&master()?, DocLang::Es);
        assert_eq!(
            doc.contact,
            [
                "ejemplo@correo.com",
                "+52 55 0000 0000",
                "Ciudad de México",
                "https://portafolio.ejemplo.com"
            ]
        );
        assert_eq!(doc.experience[0].dates, "ene 2022 – actual");
        assert_eq!(doc.languages, ["Español (Nativo)", "Inglés (B2)"]);
        assert_eq!(
            doc.section_headings(),
            [
                "Experiencia profesional",
                "Educación",
                "Habilidades",
                "Idiomas"
            ]
        );
        Ok(())
    }

    #[test]
    fn el_texto_plano_obtiene_secciones_y_legibilidad_completas()
    -> Result<(), Box<dyn std::error::Error>> {
        let doc = CvDocument::from_master(&master()?, DocLang::Es);
        let dict = SkillDict::builtin()?;
        let r = ats_score(&dict, &doc.to_plain_text(), "Editor de video", &[]);
        assert_eq!(r.breakdown.sections, 10.0, "{:?}", r.suggestions);
        assert_eq!(r.breakdown.legibility, 20.0, "{:?}", r.suggestions);
        assert_eq!(r.breakdown.title, 10.0);
        Ok(())
    }

    #[test]
    fn detecta_idioma_de_la_vacante() {
        assert_eq!(
            DocLang::detect(include_str!("../tests/fixtures/jd_frontend_jr_en.txt")),
            DocLang::En
        );
        assert_eq!(
            DocLang::detect(include_str!("../tests/fixtures/jd_editor_video_es.txt")),
            DocLang::Es
        );
    }

    #[test]
    fn nombre_de_archivo() -> Result<(), serde_json::Error> {
        let doc = CvDocument::from_master(&master()?, DocLang::Es);
        assert_eq!(
            doc.file_stem("Editor de Vídeo (Remoto)", Some("Agencia Ñu S.A.")),
            "Persona_De_Ejemplo_Editor_De_Video_Remoto_Agencia_Nu_SA"
        );
        Ok(())
    }
}
