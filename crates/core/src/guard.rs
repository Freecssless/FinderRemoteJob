//! Guardián antiinvención (plan §4.5).
//!
//! La IA nunca escribe hechos estructurales: empresa, puesto, fechas, títulos y certificaciones
//! los pinta la plantilla desde el CV maestro. La IA solo devuelve un [`Draft`] con texto
//! reformulado, y cada frase debe citar los ids del maestro de donde sale (`source_ids`).
//!
//! Reglas (cada una con prueba):
//! 1. Cada bullet cita al menos un id existente y de su misma experiencia.
//! 2. Toda cifra del texto nuevo debe existir en sus fuentes (o, en el resumen, ser tus años de experiencia).
//! 3. Toda habilidad del diccionario que aparezca debe estar en `maestro.skills` o en las fuentes citadas.
//! 4. `skills_order` solo contiene ids del maestro, sin repetir.
//! 5. Un bullet rechazado se sustituye por el texto original de sus fuentes válidas.
//!
//! Límites conocidos: no detecta cifras escritas con letra ("treinta") ni habilidades que no
//! estén en el diccionario. Por eso el usuario revisa siempre el diff antes de aceptar.

use std::collections::HashSet;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::ats::keywords::SkillDict;
use crate::matching::years_of_experience;
use crate::model::{CvMaster, Id};

// ---------------------------------------------------------------------------
// Entrada: borrador de la IA
// ---------------------------------------------------------------------------

/// Lo único que la IA puede proponer. Se deserializa del JSON que devuelve el LLM.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Draft {
    #[serde(default)]
    pub summary: Option<DraftText>,
    #[serde(default)]
    pub bullets: Vec<DraftBullet>,
    #[serde(default)]
    pub skills_order: Vec<Id>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DraftText {
    pub text: String,
    #[serde(default)]
    pub source_ids: Vec<Id>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DraftBullet {
    pub exp_id: Id,
    pub text: String,
    #[serde(default)]
    pub source_ids: Vec<Id>,
}

// ---------------------------------------------------------------------------
// Salida
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "value")]
pub enum Issue {
    Empty,
    NoSources,
    UnknownExperience(Id),
    UnknownSource(Id),
    SourceFromOtherExperience(Id),
    NewNumber(String),
    NewSkill(String),
}

impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Issue::Empty => write!(f, "El texto está vacío."),
            Issue::NoSources => write!(f, "No indica de qué parte de tu CV sale."),
            Issue::UnknownExperience(id) => write!(f, "La experiencia «{id}» no existe en tu CV."),
            Issue::UnknownSource(id) => write!(f, "Cita «{id}», que no existe en tu CV."),
            Issue::SourceFromOtherExperience(id) => {
                write!(f, "Cita «{id}», que pertenece a otra experiencia.")
            }
            Issue::NewNumber(n) => write!(f, "La cifra «{n}» no aparece en tu CV."),
            Issue::NewSkill(s) => write!(f, "Menciona «{s}», que no está en tu CV."),
        }
    }
}

/// Resultado de revisar un texto propuesto.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Checked {
    /// `None` para el resumen.
    pub exp_id: Option<Id>,
    pub source_ids: Vec<Id>,
    pub proposed: String,
    /// Vacío = aceptado.
    pub issues: Vec<Issue>,
    /// Texto original de las fuentes válidas, para mostrar en el diff y usar si se rechaza.
    pub originals: Vec<String>,
}

impl Checked {
    pub fn accepted(&self) -> bool {
        self.issues.is_empty()
    }

    /// Lo que va al CV: el texto propuesto si pasó, o los originales si no.
    pub fn final_texts(&self) -> Vec<&str> {
        if self.accepted() {
            vec![self.proposed.as_str()]
        } else {
            self.originals.iter().map(String::as_str).collect()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GuardReport {
    pub summary: Option<Checked>,
    pub bullets: Vec<Checked>,
    /// Orden de habilidades ya filtrado.
    pub skills_order: Vec<Id>,
    /// Ids de `skills_order` que no existen en el maestro.
    pub invalid_skill_ids: Vec<Id>,
}

impl GuardReport {
    pub fn all_accepted(&self) -> bool {
        self.invalid_skill_ids.is_empty()
            && self
                .summary
                .iter()
                .chain(&self.bullets)
                .all(Checked::accepted)
    }
}

// ---------------------------------------------------------------------------
// Validación
// ---------------------------------------------------------------------------

/// Una pieza citable del maestro.
struct Source<'a> {
    text: String,
    /// Experiencia a la que pertenece (solo bullets).
    exp_id: Option<&'a str>,
}

/// Busca un id entre bullets, experiencias y habilidades.
fn source<'a>(master: &'a CvMaster, id: &str) -> Option<Source<'a>> {
    for e in &master.experience {
        if e.id == id {
            return Some(Source {
                text: format!("{}\n{}", e.title, e.company),
                exp_id: None,
            });
        }
        if let Some(b) = e.bullets.iter().find(|b| b.id == id) {
            return Some(Source {
                text: format!("{}\n{}", b.text, b.metrics.join(" ")),
                exp_id: Some(&e.id),
            });
        }
    }
    master.skill(id).map(|s| Source {
        text: s.name.clone(),
        exp_id: None,
    })
}

/// `today`: mes actual "AAAA-MM", para calcular los años de experiencia que puede citar el resumen.
pub fn validate(dict: &SkillDict, master: &CvMaster, draft: &Draft, today: &str) -> GuardReport {
    // Habilidades respaldadas por el maestro: su lista de skills + lo que dicen su titular y sus puestos
    // (si tu puesto fue "Editor de video", puedes decir "edición de video").
    let titles: Vec<&str> = std::iter::once(master.basics.headline.as_str())
        .chain(master.experience.iter().map(|e| e.title.as_str()))
        .collect();
    let master_skills: HashSet<usize> = master
        .skills
        .iter()
        .filter_map(|s| dict.lookup(&s.name))
        .chain(dict.find(&titles.join("\n")).into_iter().map(|h| h.skill))
        .collect();

    let bullets = draft
        .bullets
        .iter()
        .map(|b| {
            let mut issues = Vec::new();
            if !master.experience.iter().any(|e| e.id == b.exp_id) {
                issues.push(Issue::UnknownExperience(b.exp_id.clone()));
            }
            let mut originals = Vec::new();
            let mut source_text = String::new();
            for id in &b.source_ids {
                match source(master, id) {
                    Some(Source {
                        text,
                        exp_id: Some(owner),
                    }) if owner == b.exp_id => {
                        // Para el diff/respaldo, solo el texto del bullet (sin las métricas añadidas).
                        originals.extend(master.bullet(id).map(|bl| bl.text.clone()));
                        source_text.push_str(&text);
                        source_text.push('\n');
                    }
                    Some(_) => issues.push(Issue::SourceFromOtherExperience(id.clone())),
                    None => issues.push(Issue::UnknownSource(id.clone())),
                }
            }
            check_text(
                dict,
                &b.text,
                &b.source_ids,
                &source_text,
                &master_skills,
                &[],
                &mut issues,
            );
            Checked {
                exp_id: Some(b.exp_id.clone()),
                source_ids: b.source_ids.clone(),
                proposed: b.text.clone(),
                issues,
                originals,
            }
        })
        .collect();

    let summary = draft.summary.as_ref().map(|s| {
        let mut issues = Vec::new();
        let mut source_text = String::new();
        for id in &s.source_ids {
            match source(master, id) {
                Some(src) => {
                    source_text.push_str(&src.text);
                    source_text.push('\n');
                }
                None => issues.push(Issue::UnknownSource(id.clone())),
            }
        }
        // El resumen puede decir "N años de experiencia" si N sale de las fechas del maestro.
        let years = years_of_experience(master, today).floor() as u32;
        let extra = [years.to_string()];
        check_text(
            dict,
            &s.text,
            &s.source_ids,
            &source_text,
            &master_skills,
            &extra,
            &mut issues,
        );
        Checked {
            exp_id: None,
            source_ids: s.source_ids.clone(),
            proposed: s.text.clone(),
            issues,
            originals: Vec::new(),
        }
    });

    let mut seen = HashSet::new();
    let mut invalid_skill_ids = Vec::new();
    let mut skills_order = Vec::new();
    for id in &draft.skills_order {
        if master.skill(id).is_none() {
            invalid_skill_ids.push(id.clone());
        } else if seen.insert(id) {
            skills_order.push(id.clone());
        }
    }

    GuardReport {
        summary,
        bullets,
        skills_order,
        invalid_skill_ids,
    }
}

/// Reglas comunes a bullets y resumen: texto no vacío, fuentes presentes, sin cifras ni habilidades nuevas.
fn check_text(
    dict: &SkillDict,
    text: &str,
    source_ids: &[Id],
    source_text: &str,
    master_skills: &HashSet<usize>,
    extra_numbers: &[String],
    issues: &mut Vec<Issue>,
) {
    if text.trim().is_empty() {
        issues.push(Issue::Empty);
        return;
    }
    if source_ids.is_empty() {
        issues.push(Issue::NoSources);
    }

    // Habilidades: permitidas si están en maestro.skills o en las fuentes citadas.
    let source_skills: HashSet<usize> = dict
        .find(source_text)
        .into_iter()
        .map(|h| h.skill)
        .collect();
    let hits = dict.find(text);
    for h in &hits {
        if !master_skills.contains(&h.skill) && !source_skills.contains(&h.skill) {
            let issue = Issue::NewSkill(h.surface.to_string());
            if !issues.contains(&issue) {
                issues.push(issue);
            }
        }
    }

    // Cifras: se ignoran las que forman parte de un nombre de habilidad ("Cinema 4D", "Animación 2D"),
    // que ya se validaron arriba.
    let mut without_skills = text.to_string();
    for h in &hits {
        without_skills.replace_range(
            h.start..h.start + h.surface.len(),
            &" ".repeat(h.surface.len()),
        );
    }
    let allowed: HashSet<String> = numbers(source_text)
        .into_iter()
        .chain(extra_numbers.iter().cloned())
        .collect();
    for n in numbers(&without_skills) {
        if !allowed.contains(&n) {
            let issue = Issue::NewNumber(n);
            if !issues.contains(&issue) {
                issues.push(issue);
            }
        }
    }
}

/// Cifras del texto en forma canónica: "1,200" y "1.200" → "1200"; "2,5" y "2.5" → "2.5"; "30%" → "30".
pub fn numbers(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if !chars[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        let start = i;
        while i < chars.len()
            && (chars[i].is_ascii_digit()
                || (matches!(chars[i], '.' | ',')
                    && chars.get(i + 1).is_some_and(char::is_ascii_digit)))
        {
            i += 1;
        }
        let raw: String = chars[start..i].iter().collect();
        out.push(canonical_number(&raw));
    }
    out
}

fn canonical_number(raw: &str) -> String {
    let parts: Vec<&str> = raw.split(['.', ',']).collect();
    match parts.as_slice() {
        [single] => single.to_string(),
        // Todos los grupos de 3 dígitos → separador de miles.
        [_, rest @ ..] if rest.iter().all(|p| p.len() == 3) => parts.concat(),
        [int, dec] => format!("{int}.{dec}"),
        _ => parts.concat(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type R = Result<(), Box<dyn std::error::Error>>;
    const TODAY: &str = "2026-11";

    fn setup() -> Result<(SkillDict, CvMaster), Box<dyn std::error::Error>> {
        let dict = SkillDict::builtin()?;
        let master = serde_json::from_str(include_str!("../tests/fixtures/cv_editor_video.json"))?;
        Ok((dict, master))
    }

    fn bullet(exp: &str, text: &str, sources: &[&str]) -> DraftBullet {
        DraftBullet {
            exp_id: exp.into(),
            text: text.into(),
            source_ids: sources.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn check_one(b: DraftBullet) -> Result<Checked, Box<dyn std::error::Error>> {
        let (dict, master) = setup()?;
        let draft = Draft {
            bullets: vec![b],
            ..Default::default()
        };
        let mut r = validate(&dict, &master, &draft, TODAY);
        Ok(r.bullets.remove(0))
    }

    #[test]
    fn reformulacion_fiel_se_acepta() -> R {
        let c = check_one(bullet(
            "exp_01",
            "Edité 120 videos para YouTube usando Adobe Premiere Pro y After Effects",
            &["b_01"],
        ))?;
        assert!(c.accepted(), "{:?}", c.issues);
        assert_eq!(
            c.final_texts(),
            ["Edité 120 videos para YouTube usando Adobe Premiere Pro y After Effects"]
        );
        Ok(())
    }

    #[test]
    fn numero_inventado_se_rechaza_y_vuelve_al_original() -> R {
        let c = check_one(bullet(
            "exp_01",
            "Edité más de 150 videos para YouTube",
            &["b_01"],
        ))?;
        assert_eq!(c.issues, [Issue::NewNumber("150".into())]);
        assert_eq!(
            c.final_texts(),
            ["Edité 120 videos para YouTube con Premiere y After Effects"]
        );
        Ok(())
    }

    #[test]
    fn porcentaje_cambiado_se_rechaza() -> R {
        let c = check_one(bullet(
            "exp_01",
            "Reduje 45% el tiempo de entrega con plantillas reutilizables",
            &["b_02"],
        ))?;
        assert_eq!(c.issues, [Issue::NewNumber("45".into())]);
        Ok(())
    }

    #[test]
    fn habilidad_nueva_se_rechaza_sin_confundir_su_numero() -> R {
        let c = check_one(bullet(
            "exp_01",
            "Edité 120 videos con Final Cut Pro y Cinema 4D",
            &["b_01"],
        ))?;
        // "4" de "Cinema 4D" no cuenta como cifra inventada: el problema es la habilidad.
        assert_eq!(
            c.issues,
            [
                Issue::NewSkill("Final Cut Pro".into()),
                Issue::NewSkill("Cinema 4D".into())
            ]
        );
        Ok(())
    }

    #[test]
    fn habilidad_de_la_fuente_se_permite_aunque_no_este_en_skills() -> R {
        // YouTube no está en maestro.skills, pero sí en el bullet citado.
        let c = check_one(bullet(
            "exp_01",
            "Produje contenido para YouTube (120 videos)",
            &["b_01"],
        ))?;
        assert!(c.accepted(), "{:?}", c.issues);
        Ok(())
    }

    #[test]
    fn fuente_falsa_o_ausente_se_rechaza_sin_respaldo() -> R {
        let falsa = check_one(bullet("exp_01", "Lideré un equipo", &["b_99"]))?;
        assert_eq!(falsa.issues, [Issue::UnknownSource("b_99".into())]);
        assert!(
            falsa.final_texts().is_empty(),
            "sin fuente válida no hay texto que conservar"
        );

        let sin = check_one(bullet("exp_01", "Lideré un equipo", &[]))?;
        assert_eq!(sin.issues, [Issue::NoSources]);
        Ok(())
    }

    #[test]
    fn fuente_de_otra_experiencia_o_experiencia_inexistente() -> R {
        let otra = check_one(bullet("exp_01", "Corrección de color para TV", &["b_03"]))?;
        // Además, "Corrección de color" queda sin respaldo porque la única fuente es ajena.
        assert_eq!(
            otra.issues,
            [
                Issue::SourceFromOtherExperience("b_03".into()),
                Issue::NewSkill("Corrección de color".into())
            ]
        );

        let inexistente = check_one(bullet("exp_09", "Edité 120 videos", &["b_01"]))?;
        assert!(
            inexistente
                .issues
                .contains(&Issue::UnknownExperience("exp_09".into()))
        );
        Ok(())
    }

    #[test]
    fn fusionar_dos_logros_conserva_sus_cifras() -> R {
        let c = check_one(bullet(
            "exp_01",
            "Edité 120 videos y reduje 30% el tiempo de entrega con plantillas",
            &["b_01", "b_02"],
        ))?;
        assert!(c.accepted(), "{:?}", c.issues);
        assert_eq!(c.originals.len(), 2);
        Ok(())
    }

    #[test]
    fn texto_vacio_se_rechaza() -> R {
        let c = check_one(bullet("exp_01", "   ", &["b_01"]))?;
        assert_eq!(c.issues, [Issue::Empty]);
        Ok(())
    }

    #[test]
    fn resumen_puede_citar_anos_reales_pero_no_inventados() -> R {
        let (dict, master) = setup()?;
        let draft = |t: &str| Draft {
            summary: Some(DraftText {
                text: t.into(),
                source_ids: vec!["b_01".into(), "sk_01".into()],
            }),
            ..Default::default()
        };
        // 2020-06 → 2026-11 son 6.3 años: "6" es válido.
        let ok = validate(
            &dict,
            &master,
            &draft("Editor de video con 6 años de experiencia en YouTube y Adobe Premiere Pro"),
            TODAY,
        );
        assert!(ok.all_accepted(), "{:?}", ok.summary);

        let mal = validate(
            &dict,
            &master,
            &draft("Editor de video con 10 años de experiencia"),
            TODAY,
        );
        let s = mal.summary.ok_or("falta el resumen")?;
        assert_eq!(s.issues, [Issue::NewNumber("10".into())]);
        Ok(())
    }

    #[test]
    fn orden_de_habilidades_solo_con_ids_del_maestro() -> R {
        let (dict, master) = setup()?;
        let draft = Draft {
            skills_order: ["sk_03", "sk_99", "sk_01", "sk_03"]
                .map(String::from)
                .to_vec(),
            ..Default::default()
        };
        let r = validate(&dict, &master, &draft, TODAY);
        assert_eq!(r.skills_order, ["sk_03", "sk_01"]);
        assert_eq!(r.invalid_skill_ids, ["sk_99"]);
        assert!(!r.all_accepted());
        Ok(())
    }

    #[test]
    fn cifras_en_forma_canonica() {
        assert_eq!(numbers("1,200 vistas y 1.200 clics"), ["1200", "1200"]);
        assert_eq!(numbers("2,5 millones o 2.5M"), ["2.5", "2.5"]);
        assert_eq!(numbers("30% en 2023."), ["30", "2023"]);
        assert!(numbers("sin cifras").is_empty());
    }

    #[test]
    fn borrador_se_lee_del_json_del_llm() -> R {
        let json = r#"{
            "summary": { "text": "Editor de video", "source_ids": ["exp_01"] },
            "bullets": [{ "exp_id": "exp_01", "text": "Edité 120 videos", "source_ids": ["b_01"] }],
            "skills_order": ["sk_01"]
        }"#;
        let draft: Draft = serde_json::from_str(json)?;
        let (dict, master) = setup()?;
        assert!(validate(&dict, &master, &draft, TODAY).all_accepted());
        Ok(())
    }
}
