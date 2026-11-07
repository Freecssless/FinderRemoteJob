//! Importación del CV del usuario: de lo que extrae la IA a un [`CvMaster`] revisable.
//!
//! La IA solo transcribe el CV a una estructura ([`RawCv`], sin ids). Aquí, de forma determinista:
//! - se asignan los ids (`exp_01`, `b_01`, `sk_01`…): nunca los inventa la IA;
//! - se calculan las cifras de cada logro;
//! - se normalizan las fechas ("2022-01", "2022", trabajo actual = sin fin);
//! - se marca con un aviso todo dato que **no aparece tal cual en el archivo original**, para que el
//!   usuario lo revise en el formulario antes de guardar. Nada se guarda sin su confirmación.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::ats::keywords::normalize;
use crate::guard::numbers;
use crate::model::{
    Basics, Bullet, Certification, CvMaster, Education, Experience, Language, Skill,
};

/// Estructura que devuelve la IA (ver el esquema JSON en `finder_infra::llm::tasks`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RawCv {
    #[serde(default)]
    pub basics: Basics,
    #[serde(default)]
    pub experience: Vec<RawExperience>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub education: Vec<RawEducation>,
    #[serde(default)]
    pub certifications: Vec<RawCertification>,
    #[serde(default)]
    pub languages: Vec<Language>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RawExperience {
    pub company: String,
    pub title: String,
    #[serde(default)]
    pub start: String,
    #[serde(default)]
    pub end: Option<String>,
    #[serde(default)]
    pub bullets: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RawEducation {
    pub institution: String,
    pub degree: String,
    #[serde(default)]
    pub start: Option<String>,
    #[serde(default)]
    pub end: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RawCertification {
    pub name: String,
    #[serde(default)]
    pub issuer: Option<String>,
    #[serde(default)]
    pub date: Option<String>,
}

/// CV listo para el formulario de revisión, con avisos de lo que conviene revisar.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ImportResult {
    pub cv: CvMaster,
    pub warnings: Vec<String>,
}

/// Fracción mínima de palabras de un dato que deben estar en el original para no avisar.
/// Tolera cambios de puntuación o de mayúsculas, no frases nuevas.
const MIN_WORD_COVERAGE: f32 = 0.8;

pub fn finalize(raw: RawCv, source_text: &str) -> ImportResult {
    let source = Source::new(source_text);
    let mut warnings = Vec::new();

    let basics = raw.basics;
    warnings.extend(source.warn("El nombre", &basics.name));
    warnings.extend(source.warn("El titular", &basics.headline));

    let mut experience = Vec::new();
    let mut bullet_n = 0;
    for (i, e) in raw
        .experience
        .into_iter()
        .filter(|e| !(e.company.trim().is_empty() && e.title.trim().is_empty()))
        .enumerate()
    {
        warnings.extend(source.warn("La empresa", &e.company));
        warnings.extend(source.warn("El puesto", &e.title));
        // Una fecha que no está en el CV la inventó la IA: se deja vacía para que la completes.
        let mut start = normalize_date(&e.start);
        if !start.is_empty() && !source.has_year_of(&start) {
            warnings.push(format!(
                "No encontramos en tu CV la fecha de inicio de «{}»; complétala.",
                e.title
            ));
            start.clear();
        }
        let end = match e.end.as_deref().and_then(normalize_end) {
            Some(d) if !source.has_year_of(&d) => {
                warnings.push(format!(
                    "No encontramos en tu CV la fecha de fin de «{}»; complétala.",
                    e.title
                ));
                Some(String::new())
            }
            end => end,
        };
        let bullets = e
            .bullets
            .into_iter()
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .map(|text| {
                warnings.extend(source.warn("El logro", &text));
                bullet_n += 1;
                Bullet {
                    id: format!("b_{bullet_n:02}"),
                    metrics: numbers(&text),
                    text,
                }
            })
            .collect();
        experience.push(Experience {
            id: format!("exp_{:02}", i + 1),
            company: e.company.trim().to_string(),
            title: e.title.trim().to_string(),
            start,
            end,
            bullets,
        });
    }

    // Habilidades sin repetir (ignorando mayúsculas y acentos).
    let mut seen = HashSet::new();
    let skills: Vec<Skill> = raw
        .skills
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && seen.insert(normalize(s)))
        .enumerate()
        .map(|(i, name)| {
            warnings.extend(source.warn("La habilidad", &name));
            Skill {
                id: format!("sk_{:02}", i + 1),
                name,
                level: None,
                years: None,
            }
        })
        .collect();

    let education = raw
        .education
        .into_iter()
        .enumerate()
        .map(|(i, e)| {
            warnings.extend(source.warn("La institución", &e.institution));
            warnings.extend(source.warn("El título", &e.degree));
            Education {
                id: format!("edu_{:02}", i + 1),
                institution: e.institution.trim().to_string(),
                degree: e.degree.trim().to_string(),
                // Fechas de estudios que no están en el CV: se quitan (sin aviso: son opcionales).
                start: e
                    .start
                    .as_deref()
                    .map(normalize_date)
                    .filter(|d| source.has_year_of(d)),
                end: e
                    .end
                    .as_deref()
                    .and_then(normalize_end)
                    .filter(|d| source.has_year_of(d)),
            }
        })
        .collect();

    let certifications = raw
        .certifications
        .into_iter()
        .enumerate()
        .map(|(i, c)| {
            warnings.extend(source.warn("La certificación", &c.name));
            Certification {
                id: format!("cert_{:02}", i + 1),
                name: c.name.trim().to_string(),
                issuer: c.issuer,
                date: c.date,
            }
        })
        .collect();

    ImportResult {
        cv: CvMaster {
            basics,
            experience,
            skills,
            education,
            certifications,
            languages: raw.languages,
        },
        warnings,
    }
}

/// Fechas a "AAAA-MM" o "AAAA": "2022-1", "03/2022", "Mar 2022", "marzo de 2022", "March 2022".
/// Lo que no se entiende se conserva para que el usuario lo corrija.
pub(crate) fn normalize_date(s: &str) -> String {
    let s = s.trim();
    let n = normalize(s);
    let parts: Vec<&str> = n
        .split(|c: char| !c.is_alphanumeric())
        .filter(|p| !p.is_empty() && *p != "de" && *p != "of")
        .collect();
    let year = parts
        .iter()
        .find(|p| p.len() == 4 && p.chars().all(|c| c.is_ascii_digit()));
    let month = parts.iter().find_map(|p| {
        if p.chars().all(|c| c.is_ascii_digit()) {
            p.parse::<u8>()
                .ok()
                .filter(|m| p.len() <= 2 && (1..=12).contains(m))
        } else {
            month_number(p)
        }
    });
    match (year, month, parts.len()) {
        (Some(y), Some(m), 2) => format!("{y}-{m:02}"),
        (Some(y), None, 1) => y.to_string(),
        _ => s.to_string(),
    }
}

fn month_number(word: &str) -> Option<u8> {
    const MONTHS: [&[&str]; 12] = [
        &["ene", "enero", "jan", "january"],
        &["feb", "febrero", "february"],
        &["mar", "marzo", "march"],
        &["abr", "abril", "apr", "april"],
        &["may", "mayo"],
        &["jun", "junio", "june"],
        &["jul", "julio", "july"],
        &["ago", "agosto", "aug", "august"],
        &["sep", "sept", "septiembre", "setiembre", "september"],
        &["oct", "octubre", "october"],
        &["nov", "noviembre", "november"],
        &["dic", "diciembre", "dec", "december"],
    ];
    MONTHS
        .iter()
        .position(|m| m.contains(&word))
        .map(|i| i as u8 + 1)
}

/// Fin de un periodo: "actual", "presente", "present"… = trabajo actual (`None`).
fn normalize_end(s: &str) -> Option<String> {
    const CURRENT: &[&str] = &[
        "",
        "actual",
        "actualidad",
        "presente",
        "present",
        "current",
        "now",
        "hoy",
        "la fecha",
        "a la fecha",
        "actualmente",
        "en curso",
        "today",
    ];
    let n = normalize(s.trim());
    (!CURRENT.contains(&n.as_str())).then(|| normalize_date(s))
}

/// Texto original indexado por palabras normalizadas.
struct Source {
    words: HashSet<String>,
}

impl Source {
    fn new(text: &str) -> Self {
        Self {
            words: words(text).collect(),
        }
    }

    /// Aviso si `text` no aparece (casi) tal cual en el original.
    fn warn(&self, what: &str, text: &str) -> Option<String> {
        (!text.trim().is_empty() && !self.contains(text))
            .then(|| format!("{what} «{text}» no aparece tal cual en tu CV original; revísalo."))
    }

    /// ¿Al menos el 80 % de las palabras significativas del dato están en el original?
    fn contains(&self, text: &str) -> bool {
        let wanted: Vec<String> = words(text).collect();
        if wanted.is_empty() {
            return true;
        }
        let found = wanted.iter().filter(|w| self.words.contains(*w)).count();
        found as f32 / wanted.len() as f32 >= MIN_WORD_COVERAGE
    }

    /// ¿El año de la fecha aparece en el original?
    fn has_year_of(&self, date: &str) -> bool {
        date.get(..4).is_some_and(|y| self.words.contains(y))
    }
}

/// Palabras normalizadas de 3+ caracteres o cifras (años, cantidades).
fn words(text: &str) -> impl Iterator<Item = String> {
    normalize(text)
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 3 || w.chars().all(|c| c.is_ascii_digit()) && !w.is_empty())
        .map(String::from)
        .collect::<Vec<_>>()
        .into_iter()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = include_str!("../tests/fixtures/cv_editor_video.txt");

    fn raw() -> RawCv {
        RawCv {
            basics: Basics { name: "Persona de Ejemplo".into(), headline: "Editor de video".into(), ..Default::default() },
            experience: vec![
                RawExperience {
                    company: "Productora X".into(),
                    title: "Editor de video".into(),
                    start: "2022-1".into(),
                    end: Some("Actual".into()),
                    bullets: vec![
                        "Edité 120 videos para YouTube con Premiere y After Effects".into(),
                        "  ".into(),
                        "Reduje el tiempo de entrega 30% creando plantillas de edición reutilizables".into(),
                    ],
                },
                RawExperience { company: " ".into(), title: "".into(), ..Default::default() },
                RawExperience {
                    company: "Agencia Y".into(),
                    title: "Asistente de postproducción".into(),
                    start: "2020-06".into(),
                    end: Some("2021-12".into()),
                    bullets: vec!["Corrección de color y sonido para comerciales de TV".into()],
                },
            ],
            skills: vec!["Adobe Premiere Pro".into(), "adobe premiere pro".into(), "DaVinci Resolve".into()],
            education: vec![RawEducation {
                institution: "Universidad Ejemplo".into(),
                degree: "Lic. en Comunicación".into(),
                start: Some("2016".into()),
                end: Some("2020".into()),
            }],
            ..Default::default()
        }
    }

    #[test]
    fn entiende_fechas_con_nombre_de_mes() {
        for (input, want) in [
            ("Mar 2022", "2022-03"),
            ("marzo de 2022", "2022-03"),
            ("March 2022", "2022-03"),
            ("03/2022", "2022-03"),
            ("2022/3", "2022-03"),
            ("2022-1", "2022-01"),
            ("Dic. 2018", "2018-12"),
            ("2019", "2019"),
            ("verano 2019", "verano 2019"),
        ] {
            assert_eq!(normalize_date(input), want, "{input}");
        }
        assert_eq!(normalize_end("A la fecha"), None);
        assert_eq!(normalize_end("Presente"), None);
    }

    #[test]
    fn asigna_ids_cifras_y_fechas() {
        let r = finalize(raw(), SOURCE);
        assert!(r.warnings.is_empty(), "{:?}", r.warnings);
        let cv = &r.cv;
        assert_eq!(cv.experience.len(), 2, "se descarta la experiencia vacía");
        assert_eq!(cv.experience[0].id, "exp_01");
        assert_eq!(cv.experience[0].start, "2022-01");
        assert_eq!(cv.experience[0].end, None, "«Actual» = trabajo actual");
        let ids: Vec<_> = cv
            .experience
            .iter()
            .flat_map(|e| &e.bullets)
            .map(|b| b.id.as_str())
            .collect();
        assert_eq!(
            ids,
            ["b_01", "b_02", "b_03"],
            "numeración global y sin bullets vacíos"
        );
        assert_eq!(cv.experience[0].bullets[1].metrics, ["30"]);
        assert_eq!(cv.skills.len(), 2, "sin habilidades repetidas");
        assert_eq!(cv.skills[1].id, "sk_02");
        assert_eq!(cv.education[0].id, "edu_01");
    }

    #[test]
    fn avisa_de_lo_que_no_esta_en_el_original() {
        let mut r = raw();
        r.experience[0]
            .bullets
            .push("Dirigí un equipo de 15 personas en Netflix".into());
        r.skills.push("Final Cut Pro".into());
        r.experience[2].start = "2015-03".into();
        let out = finalize(r, SOURCE);
        assert_eq!(out.warnings.len(), 3, "{:?}", out.warnings);
        assert!(
            out.warnings
                .iter()
                .any(|w| w.contains("fecha de inicio de «Asistente de postproducción»"))
        );
        // La fecha inventada se quita (los textos se conservan con su aviso).
        assert_eq!(out.cv.experience[1].start, "");
        assert!(out.warnings.iter().any(|w| w.contains("Netflix")));
        assert!(out.warnings.iter().any(|w| w.contains("Final Cut Pro")));
        // Se conserva para que el usuario decida: el aviso no borra nada.
        assert_eq!(out.cv.skills.len(), 3);
    }

    #[test]
    fn lee_el_json_de_la_ia_con_campos_opcionales() -> Result<(), serde_json::Error> {
        let json = r#"{"basics":{"name":"Ana"},"experience":[{"company":"X","title":"Y","start":"2023","end":null,"bullets":[]}],"skills":["Excel"]}"#;
        let raw: RawCv = serde_json::from_str(json)?;
        let r = finalize(raw, "Ana X Y 2023 Excel");
        assert!(r.warnings.is_empty(), "{:?}", r.warnings);
        assert_eq!(r.cv.experience[0].start, "2023");
        Ok(())
    }
}
