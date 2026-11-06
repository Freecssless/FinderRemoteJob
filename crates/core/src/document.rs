//! CV listo para imprimir: lo que reciben las plantillas de PDF y DOCX.
//!
//! Aquí se deciden, de forma determinista, el orden de secciones, los encabezados estándar
//! (en el idioma de la vacante) y el formato único de fechas. Las plantillas solo dibujan.
//! El contenido viene del CV maestro ([`CvDocument::from_master`]); el optimizador (módulo 9)
//! sustituye resumen, bullets y orden de habilidades por los ya validados por el guardián.

use serde::{Deserialize, Serialize};

use crate::ats::keywords::{SkillDict, contains_word, normalize};
use crate::guard::{Checked, GuardReport};
use crate::model::{CvMaster, Term};

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

/// Plantilla del CV (plan §4.6). Se guarda como `variant` para las estadísticas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Template {
    /// Una columna, espaciado cómodo.
    #[default]
    Clasica,
    /// Letra y márgenes más chicos para caber en una página.
    Compacta,
    /// Habilidades justo después del resumen, antes de la experiencia.
    Tecnica,
}

impl Template {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Clasica => "clasica",
            Self::Compacta => "compacta",
            Self::Tecnica => "tecnica",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        [Self::Clasica, Self::Compacta, Self::Tecnica]
            .into_iter()
            .find(|t| t.as_str() == s)
    }

    fn order(self) -> [Section; 6] {
        use Section::*;
        match self {
            Self::Clasica | Self::Compacta => [
                Summary,
                Experience,
                Education,
                Skills,
                Certifications,
                Languages,
            ],
            Self::Tecnica => [
                Summary,
                Skills,
                Experience,
                Education,
                Certifications,
                Languages,
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Section {
    Summary,
    Experience,
    Education,
    Skills,
    Certifications,
    Languages,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CvDocument {
    pub lang: DocLang,
    pub template: Template,
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
            template: Template::default(),
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
    /// - Logros: por experiencia, en el orden propuesto; los rechazados vuelven a su texto original
    ///   y los que ninguna propuesta citó van al final. Nunca se pierde un logro del maestro.
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
            let proposed: Vec<(bool, &Checked)> = report
                .bullets
                .iter()
                .enumerate()
                .filter(|(_, c)| c.exp_id.as_deref() == Some(exp.id.as_str()))
                .map(|(i, c)| {
                    (
                        c.accepted() && choices.bullets.get(i).copied().unwrap_or(true),
                        c,
                    )
                })
                .collect();
            if proposed.is_empty() {
                continue;
            }
            // Logros ya contenidos en una propuesta que se usa: un rechazo no los repite.
            let covered: Vec<&str> = proposed
                .iter()
                .filter(|(used, _)| *used)
                .flat_map(|(_, c)| c.source_ids.iter().map(String::as_str))
                .collect();
            let mut bullets: Vec<String> = Vec::new();
            for (used, c) in proposed {
                let texts: Vec<&str> = if used {
                    vec![c.proposed.as_str()]
                } else {
                    exp.bullets
                        .iter()
                        .filter(|b| {
                            c.source_ids.contains(&b.id) && !covered.contains(&b.id.as_str())
                        })
                        .map(|b| b.text.as_str())
                        .collect()
                };
                for t in texts {
                    // Un original puede repetirse si dos propuestas citaban el mismo logro.
                    if !bullets.iter().any(|b| b == t) {
                        bullets.push(t.to_string());
                    }
                }
            }
            // Reescribir no es borrar: los logros que ninguna propuesta usada cubre se conservan.
            for b in &exp.bullets {
                if !covered.contains(&b.id.as_str()) && !bullets.contains(&b.text) {
                    bullets.push(b.text.clone());
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

    /// Escribe cada habilidad como la nombra la vacante cuando el diccionario dice que es la misma
    /// ("Color Grading" → "Corrección de color (Color Grading)"): algunos ATS comparan literal. No agrega
    /// habilidades: solo renombra las que ya están. Si la forma del anuncio es el puesto
    /// ("Editor de Video"), se usa el nombre de la habilidad ("Edición de video").
    pub fn align_skills(&mut self, dict: &SkillDict, terms: &[Term], job_title: &str) {
        let title = normalize(job_title);
        let mut out: Vec<String> = Vec::new();
        for s in &self.skills {
            let job_form = dict.lookup(s).and_then(|i| {
                terms
                    .iter()
                    .find(|t| dict.lookup(&t.canonical) == Some(i))
                    .map(|t| {
                        if contains_word(&title, &normalize(&t.surface)) {
                            dict.skill(i).name.as_str()
                        } else {
                            t.surface.as_str()
                        }
                    })
            });
            let name = match job_form {
                Some(j)
                    if normalize(j) != normalize(s) && !normalize(j).contains(&normalize(s)) =>
                {
                    format!("{j} ({s})")
                }
                Some(j) => j.to_string(),
                None => s.clone(),
            };
            if !out.contains(&name) {
                out.push(name);
            }
        }
        self.skills = out;
    }

    /// Secciones con contenido, en el orden de la plantilla. PDF, DOCX y texto plano siguen este
    /// orden; la prueba de ida y vuelta lo verifica.
    pub fn sections(&self) -> Vec<Section> {
        self.template
            .order()
            .into_iter()
            .filter(|s| match s {
                Section::Summary => self.summary.is_some(),
                Section::Experience => !self.experience.is_empty(),
                Section::Education => !self.education.is_empty(),
                Section::Skills => !self.skills.is_empty(),
                Section::Certifications => !self.certifications.is_empty(),
                Section::Languages => !self.languages.is_empty(),
            })
            .collect()
    }

    pub fn heading(&self, s: Section) -> &'static str {
        let h = &self.headings;
        match s {
            Section::Summary => h.summary,
            Section::Experience => h.experience,
            Section::Education => h.education,
            Section::Skills => h.skills,
            Section::Certifications => h.certifications,
            Section::Languages => h.languages,
        }
    }

    /// Texto plano en el mismo orden que las plantillas: lo que un ATS debería poder leer.
    pub fn to_plain_text(&self) -> String {
        let mut out = vec![self.name.clone()];
        if !self.headline.is_empty() {
            out.push(self.headline.clone());
        }
        out.push(self.contact.join(" | "));
        for sec in self.sections() {
            out.extend([String::new(), self.heading(sec).to_uppercase()]);
            match sec {
                Section::Summary => out.extend(self.summary.clone()),
                Section::Experience => {
                    for e in &self.experience {
                        out.push(format!("{} — {}", e.title, e.company));
                        out.push(e.dates.clone());
                        out.extend(e.bullets.iter().map(|b| format!("- {b}")));
                    }
                }
                Section::Education => out.extend(
                    self.education
                        .iter()
                        .map(|e| format!("{} — {}, {}", e.degree, e.institution, e.dates)),
                ),
                Section::Skills => out.push(self.skills.join(", ")),
                Section::Certifications => out.push(self.certifications.join(", ")),
                Section::Languages => out.push(self.languages.join(", ")),
            }
        }
        out.join("\n")
    }

    /// Encabezados de las secciones presentes, en orden (para la prueba de ida y vuelta).
    pub fn section_headings(&self) -> Vec<&'static str> {
        self.sections()
            .into_iter()
            .map(|s| self.heading(s))
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

    /// Caso real con qwen3:8b: una propuesta aceptada fusiona b_01 y b_02, y otra rechazada (habilidad
    /// inventada) también cita b_01. El rechazo no debe devolver b_01: ya está dentro de la fusión.
    #[test]
    fn rechazo_no_duplica_un_logro_ya_usado() -> Result<(), Box<dyn std::error::Error>> {
        use crate::guard::{Draft, DraftBullet, validate};
        let m = master()?;
        let dict = SkillDict::builtin()?;
        let b = |text: &str, src: &[&str]| DraftBullet {
            exp_id: "exp_01".into(),
            text: text.into(),
            source_ids: src.iter().map(|s| s.to_string()).collect(),
        };
        let draft = Draft {
            summary: None,
            bullets: vec![
                b(
                    "Edité 120 videos para YouTube con Premiere y After Effects y reduje 30% el tiempo de entrega",
                    &["b_01", "b_02"],
                ),
                b("Edité 120 videos de TikTok con After Effects", &["b_01"]), // TikTok no está en el CV
            ],
            skills_order: vec![],
        };
        let report = validate(&dict, &m, &draft, "2026-11");
        let all = Choices::accept_all(&report);
        assert_eq!(all.bullets, [true, false]);
        let doc = CvDocument::from_report(&m, &report, &all, DocLang::Es);
        assert_eq!(
            doc.experience[0].bullets,
            [
                "Edité 120 videos para YouTube con Premiere y After Effects y reduje 30% el tiempo de entrega"
            ]
        );

        // Si el usuario rechaza la fusión, vuelven ambos originales, sin repetir b_01.
        let none = Choices {
            summary: false,
            bullets: vec![false, false],
        };
        let doc = CvDocument::from_report(&m, &report, &none, DocLang::Es);
        assert_eq!(doc.experience[0].bullets.len(), 2);
        Ok(())
    }

    /// Caso real: el redactor resumió 7 logros en 4 y el CV perdió los que no citó (entre ellos el
    /// de "700 mil a 1.2 millones" de visualizaciones). Reescribir no es borrar.
    #[test]
    fn un_logro_que_nadie_cito_no_se_pierde() -> Result<(), Box<dyn std::error::Error>> {
        use crate::guard::{Draft, DraftBullet, validate};
        let m = master()?;
        let dict = SkillDict::builtin()?;
        let draft = Draft {
            summary: None,
            bullets: vec![DraftBullet {
                exp_id: "exp_01".into(),
                text: "Reduje 30% el tiempo de entrega con plantillas de edición reutilizables"
                    .into(),
                source_ids: vec!["b_02".into()],
            }],
            skills_order: vec![],
        };
        let report = validate(&dict, &m, &draft, "2026-11");
        let doc = CvDocument::from_report(&m, &report, &Choices::accept_all(&report), DocLang::Es);
        assert_eq!(
            doc.experience[0].bullets,
            [
                "Reduje 30% el tiempo de entrega con plantillas de edición reutilizables",
                "Edité 120 videos para YouTube con Premiere y After Effects",
            ]
        );
        Ok(())
    }

    #[test]
    fn habilidades_con_la_forma_del_anuncio() -> Result<(), Box<dyn std::error::Error>> {
        use crate::model::{Term, TermKind};
        let dict = SkillDict::builtin()?;
        let term = |s: &str| Term {
            surface: s.into(),
            canonical: dict
                .lookup(s)
                .map(|i| dict.skill(i).name.clone())
                .unwrap_or_else(|| s.into()),
            kind: TermKind::Tool,
            required: true,
            weight: 3.0,
        };
        let mut doc = CvDocument::from_master(&master()?, DocLang::Es);
        doc.skills = vec![
            "Premiere".into(),
            "Color Grading".into(),
            "Video Editing".into(),
            "Excel".into(),
        ];
        let terms = [
            term("Adobe Premiere Pro"),
            term("Corrección de color"),
            term("Editor de Video"),
        ];
        doc.align_skills(&dict, &terms, "Editor de Video para Anuncios");
        assert_eq!(
            doc.skills,
            [
                "Adobe Premiere Pro",
                "Corrección de color (Color Grading)",
                "Edición de video (Video Editing)",
                "Excel"
            ]
        );
        // Ya con la forma exacta, o contenida en ella: no se repite.
        doc.skills = vec!["Adobe Premiere Pro".into(), "Premiere Pro".into()];
        doc.align_skills(&dict, &[term("Adobe Premiere Pro")], "Editor");
        assert_eq!(doc.skills, ["Adobe Premiere Pro"]);
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
