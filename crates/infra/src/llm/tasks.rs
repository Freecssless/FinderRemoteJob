//! Tareas de IA del MVP. Cada una: instrucciones + esquema JSON + validación determinista en `core`.
//!
//! - [`import_cv`]: PDF/DOCX → texto → la IA lo transcribe → `core::import::finalize` (ids y avisos).
//! - [`extract_requirements`]: diccionario + IA → `core::ats::requirements::merge` (verificado contra el anuncio).
//! - [`semantic_similarity`]: coseno entre embeddings del CV y de la vacante.
//!
//! Por privacidad, estas tareas usan el modelo **local** (el CV no sale del equipo).

use finder_core::ats::keywords::{SkillDict, extract_terms};
use finder_core::ats::requirements::{LlmRequirements, merge};
use finder_core::cvparse::{LineRange, ParsedCv, tidy_name};
use finder_core::import::{
    ImportResult, RawCertification, RawCv, RawEducation, RawExperience, finalize,
};
use finder_core::matching::cosine;
use finder_core::model::JobRequirements;
use finder_core::model::{Basics, Language};
use serde::Deserialize;
use serde_json::{Value, json};

use super::{LlmClient, LlmError};
use crate::cvfile::{CvFileError, CvFileKind, extract_text};

#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error(transparent)]
    File(#[from] CvFileError),
    #[error(transparent)]
    Llm(#[from] LlmError),
    #[error("la IA devolvió datos con otra forma: {0}")]
    Shape(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, TaskError>;

// ---------------------------------------------------------------------------
// Esquemas (todas las propiedades requeridas y `additionalProperties: false`, como exige
// la salida estructurada; lo opcional se expresa con `null`).
// ---------------------------------------------------------------------------

fn s() -> Value {
    json!({"type": "string"})
}
fn s_or_null() -> Value {
    json!({"anyOf": [{"type": "string"}, {"type": "null"}]})
}
fn list(items: Value) -> Value {
    json!({"type": "array", "items": items})
}
fn obj(props: &[(&str, Value)]) -> Value {
    let properties: serde_json::Map<String, Value> = props
        .iter()
        .map(|(k, v)| (k.to_string(), v.clone()))
        .collect();
    let required: Vec<&str> = props.iter().map(|(k, _)| *k).collect();
    json!({"type": "object", "additionalProperties": false, "properties": properties, "required": required})
}

fn int() -> Value {
    json!({"type": "integer"})
}

/// Lo que la IA devuelve al leer un CV: datos cortos y las líneas de los logros de cada empleo
/// ([`LineRange`]); el texto lo copia y separa [`ParsedCv::bullets`]. Habilidades e idiomas solo si el CV no
/// tiene esas secciones (si las tiene, se leen sin IA).
pub fn cv_refs_schema(skills: bool, languages: bool) -> Value {
    let mut props = vec![
        ("name", s()),
        ("headline", s()),
        ("location", s()),
        (
            "experience",
            list(obj(&[
                ("company", s()),
                ("title", s()),
                ("start", s()),
                ("end", s_or_null()),
                (
                    "description",
                    json!({"anyOf": [obj(&[("from", int()), ("to", int())]), {"type": "null"}]}),
                ),
            ])),
        ),
        (
            "education",
            list(obj(&[
                ("institution", s()),
                ("degree", s()),
                ("start", s_or_null()),
                ("end", s_or_null()),
            ])),
        ),
        (
            "certifications",
            list(obj(&[
                ("name", s()),
                ("issuer", s_or_null()),
                ("date", s_or_null()),
            ])),
        ),
    ];
    if skills {
        props.push(("skills", list(s())));
    }
    if languages {
        props.push(("languages", list(obj(&[("name", s()), ("level", s())]))));
    }
    obj(&props)
}

pub fn requirements_schema() -> Value {
    obj(&[
        (
            "terms",
            list(obj(&[
                ("canonical", s()),
                ("required", json!({"type": "boolean"})),
            ])),
        ),
        (
            "extra_skills",
            list(obj(&[
                ("surface", s()),
                ("required", json!({"type": "boolean"})),
                (
                    "kind",
                    json!({"type": "string", "enum": ["Hard", "Tool", "Soft", "Language"]}),
                ),
            ])),
        ),
        (
            "seniority",
            json!({"anyOf": [{"type": "string", "enum": ["jr", "mid", "sr", "lead"]}, {"type": "null"}]}),
        ),
        (
            "years_min",
            json!({"anyOf": [{"type": "number"}, {"type": "null"}]}),
        ),
        ("languages", list(s())),
        ("blockers", list(s())),
    ])
}

// ---------------------------------------------------------------------------
// Instrucciones
// ---------------------------------------------------------------------------

const CV_SYSTEM: &str = "Lees un currículum con cada línea numerada («12| texto») y respondes en JSON. Reglas estrictas:
- Copia nombres, puestos, empresas, instituciones, títulos y fechas tal como aparecen: no traduzcas, no corrijas, no completes.
- No inventes nada. Si un dato no está, usa cadena vacía, null o lista vacía.
- \"name\": el nombre de la persona. \"headline\": su titular profesional (bajo el nombre), si existe. \"location\": su ciudad o país, si aparece arriba.
- En cada empleo, \"description\" son las líneas de sus logros o responsabilidades: el número de la primera (\"from\") y de la última (\"to\"). No incluyas la línea del puesto, de la empresa ni de las fechas. Si el empleo no tiene descripción, null.
- Fechas tal como están (por ejemplo «Mar 2022» o «2019»). Si el empleo sigue vigente, \"end\" es null.
- \"skills\" (si se pide): solo las habilidades que el CV lista explícitamente.";

const REQ_SYSTEM: &str = "Analizas anuncios de empleo y respondes solo con información que esté escrita en el anuncio.
- \"terms\": para cada término de la lista dada, indica si el anuncio lo exige (required=true) o si es deseable, un plus o solo una responsabilidad secundaria (required=false). Usa el nombre exactamente como viene en la lista.
- \"extra_skills\": habilidades, herramientas o idiomas que pide el anuncio y no están en la lista. Copia la forma exacta del anuncio en \"surface\".
- \"seniority\": jr, mid, sr o lead solo si el anuncio lo dice; si no, null.
- \"years_min\": años mínimos de experiencia solo si el anuncio da una cifra; si no, null.
- \"languages\": idiomas que el anuncio exige.
- \"blockers\": restricciones que impedirían postular desde México o LATAM (por ejemplo, residir en otro país, ciudadanía, permiso de trabajo). Copia la frase del anuncio. Si no hay, lista vacía.";

// ---------------------------------------------------------------------------
// Tareas
// ---------------------------------------------------------------------------

/// CV importado: lo que va al formulario de revisión + el texto original (se guarda para el ATS "antes").
#[derive(Debug, Clone, PartialEq)]
pub struct ImportedCv {
    pub result: ImportResult,
    pub source_text: String,
}

pub async fn import_cv(
    client: &LlmClient,
    dict: &SkillDict,
    bytes: &[u8],
    kind: CvFileKind,
) -> Result<ImportedCv> {
    import_text(client, dict, extract_text(bytes, kind)?).await
}

/// Igual que [`import_cv`], con el texto ya extraído.
pub async fn import_text(
    client: &LlmClient,
    dict: &SkillDict,
    source_text: String,
) -> Result<ImportedCv> {
    let parsed = ParsedCv::new(&source_text);
    let user = format!("CV:\n<<<\n{}\n>>>", parsed.for_llm());
    let schema = cv_refs_schema(parsed.needs_skills(), parsed.needs_languages());
    let value = client.complete_json(CV_SYSTEM, &user, &schema).await?;
    let refs: CvRefs = serde_json::from_value(value)?;
    let (raw, mut warnings) = assemble(&parsed, dict, refs);
    let mut result = finalize(raw, &source_text);
    warnings.append(&mut result.warnings);
    result.warnings = warnings;
    Ok(ImportedCv {
        result,
        source_text,
    })
}

/// Sin IA (Ollama cerrado o sin modelo): contacto, habilidades e idiomas, para no empezar de cero.
pub fn import_without_ai(dict: &SkillDict, source_text: String) -> ImportedCv {
    let parsed = ParsedCv::new(&source_text);
    let name = source_text
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("");
    let refs = CvRefs {
        name: name.to_string(),
        ..Default::default()
    };
    let (raw, _) = assemble(&parsed, dict, refs);
    let mut result = finalize(raw, &source_text);
    result.warnings.insert(
        0,
        "La IA local no respondió: se leyeron tus datos de contacto, habilidades e idiomas. Agrega tu experiencia y educación a mano, o abre Ollama y vuelve a importar.".into(),
    );
    ImportedCv {
        result,
        source_text,
    }
}

#[derive(Debug, Default, Deserialize)]
struct CvRefs {
    #[serde(default)]
    name: String,
    #[serde(default)]
    headline: String,
    #[serde(default)]
    location: String,
    #[serde(default)]
    experience: Vec<ExpRefs>,
    #[serde(default)]
    education: Vec<RawEducation>,
    #[serde(default)]
    certifications: Vec<RawCertification>,
    #[serde(default)]
    skills: Option<Vec<String>>,
    #[serde(default)]
    languages: Option<Vec<Language>>,
}

#[derive(Debug, Deserialize)]
struct ExpRefs {
    company: String,
    title: String,
    #[serde(default)]
    start: String,
    #[serde(default)]
    end: Option<String>,
    #[serde(default)]
    description: Option<LineRange>,
}

/// Junta lo que leyó la IA con lo que se leyó sin ella. Los logros se copian del CV por sus líneas.
fn assemble(parsed: &ParsedCv, dict: &SkillDict, refs: CvRefs) -> (RawCv, Vec<String>) {
    let mut warnings = Vec::new();
    let experience = refs
        .experience
        .into_iter()
        .map(|e| {
            let bullets = match e.description {
                Some(r) => {
                    let b = parsed.bullets(r);
                    if b.is_empty() {
                        warnings.push(format!(
                            "No se pudieron leer los logros de «{}» (líneas {}–{}); revísalos en tu CV.",
                            e.title, r.from, r.to
                        ));
                    }
                    b
                }
                None => Vec::new(),
            };
            RawExperience {
                company: e.company,
                title: e.title,
                start: e.start,
                end: e.end,
                bullets,
            }
        })
        .collect();
    let listed = parsed.skills.clone().or(refs.skills).unwrap_or_default();
    let raw = RawCv {
        basics: Basics {
            name: tidy_name(&refs.name),
            headline: refs.headline,
            location: refs.location,
            ..parsed.contact.clone()
        },
        experience,
        skills: parsed.merge_skills(dict, listed),
        education: refs.education,
        certifications: refs.certifications,
        languages: parsed
            .languages
            .clone()
            .or(refs.languages)
            .unwrap_or_default(),
    };
    (raw, warnings)
}

/// Requisitos de una vacante. Si la IA no está disponible, quien llama puede usar solo el
/// diccionario (`extract_terms`) y reintentar después.
pub async fn extract_requirements(
    client: &LlmClient,
    dict: &SkillDict,
    title: &str,
    description: &str,
) -> Result<JobRequirements> {
    let terms = extract_terms(dict, title, description);
    let names: Vec<&str> = terms.iter().map(|t| t.canonical.as_str()).collect();
    let user = format!(
        "Puesto: {title}\n\nAnuncio:\n<<<\n{description}\n>>>\n\nTérminos a clasificar: {}",
        serde_json::to_string(&names)?
    );
    let value = client
        .complete_json(REQ_SYSTEM, &user, &requirements_schema())
        .await?;
    let llm: LlmRequirements = serde_json::from_value(value)?;
    Ok(merge(dict, title, description, terms, llm))
}

/// Coseno entre los embeddings de los dos textos (se pasa tal cual a `MatchInput::semantic`).
pub async fn semantic_similarity(
    client: &LlmClient,
    embed_model: &str,
    cv_text: &str,
    job_text: &str,
) -> Result<Option<f32>> {
    let v = client
        .embed(embed_model, &[cv_text.to_string(), job_text.to_string()])
        .await?;
    Ok(match v.as_slice() {
        [a, b] => cosine(a, b),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::LlmConfig;

    /// Todo objeto del esquema debe declarar todas sus propiedades como requeridas y
    /// `additionalProperties: false`; si no, la API de Anthropic lo rechaza.
    fn assert_strict(v: &Value, path: &str) {
        match v {
            Value::Object(o) => {
                if o.get("type") == Some(&json!("object")) {
                    assert_eq!(o.get("additionalProperties"), Some(&json!(false)), "{path}");
                    let props: Vec<&String> = o["properties"]
                        .as_object()
                        .map(|p| p.keys().collect())
                        .unwrap_or_default();
                    let req: Vec<&str> = o["required"]
                        .as_array()
                        .map(|r| r.iter().filter_map(Value::as_str).collect())
                        .unwrap_or_default();
                    assert_eq!(
                        props.len(),
                        req.len(),
                        "{path}: todas las propiedades deben ser requeridas"
                    );
                }
                for (k, child) in o {
                    assert_strict(child, &format!("{path}.{k}"));
                }
            }
            Value::Array(a) => a.iter().for_each(|c| assert_strict(c, path)),
            _ => {}
        }
    }

    #[test]
    fn esquemas_estrictos() {
        assert_strict(&cv_refs_schema(true, true), "cv_refs");
        assert_strict(&requirements_schema(), "req");
    }

    /// Lo que respondería la IA para `cv_realista.txt`: datos cortos y líneas de cada logro.
    #[test]
    fn arma_el_cv_con_los_logros_copiados_de_sus_lineas()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let text = include_str!("../../../core/tests/fixtures/cv_realista.txt");
        let n = |needle: &str| {
            text.lines()
                .position(|l| l.contains(needle))
                .map(|i| i + 1)
                .unwrap_or(0)
        };
        let r = |a: usize, b: usize| json!({"from": a, "to": b});
        let answer = json!({
            "name": "MARÍA FERNANDA LÓPEZ RUIZ", "headline": "Diseñadora UX/UI y Front-end", "location": "Guadalajara, Jalisco",
            "experience": [
                {"company": "Banco Digital Nova", "title": "Diseñadora UX/UI Senior", "start": "Mar 2022", "end": null,
                 "description": r(n("Rediseñé"), n("Colaboré con 4") + 1)},
                {"company": "TiendaYa", "title": "Diseñadora de Producto", "start": "Ene 2019", "end": "Feb 2022",
                 "description": r(999, 1000)},
                {"company": "Agencia Pixel", "title": "Desarrolladora Front-end Jr.", "start": "Jun 2017", "end": "Dic 2018",
                 "description": r(n("Maqueté"), n("Integré CMS"))}
            ],
            "education": [{"institution": "Universidad de Guadalajara", "degree": "Licenciatura en Diseño Digital", "start": "2013", "end": "2017"}],
            "certifications": []
        });
        let refs: CvRefs = serde_json::from_value(answer)?;
        let parsed = ParsedCv::new(text);
        let dict = SkillDict::builtin()?;
        let (raw, warnings) = assemble(&parsed, &dict, refs);
        let cv = finalize(raw, text).cv;
        assert_eq!(cv.basics.name, "María Fernanda López Ruiz");
        assert_eq!(cv.basics.email, "maria.lopez@correo.com");
        assert_eq!(cv.experience[0].start, "2022-03");
        assert_eq!(cv.experience[2].end.as_deref(), Some("2018-12"));
        assert_eq!(
            cv.experience[0].bullets[0].text,
            "Rediseñé el flujo de apertura de cuenta y aumenté la conversión 18% en seis meses."
        );
        assert_eq!(cv.experience[0].bullets[0].metrics, ["18"]);
        assert_eq!(cv.experience[0].bullets.len(), 4);
        assert_eq!(cv.experience[2].bullets.len(), 2);
        assert!(
            cv.experience[1].bullets.is_empty(),
            "el rango inválido se descarta con un aviso"
        );
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert_eq!(cv.languages.len(), 3);
        assert!(cv.skills.iter().any(|s| s.name == "WordPress"));
        Ok(())
    }

    #[test]
    fn sin_ia_al_menos_contacto_habilidades_e_idiomas()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dict = SkillDict::builtin()?;
        let text = include_str!("../../../core/tests/fixtures/cv_realista.txt").to_string();
        let r = import_without_ai(&dict, text).result;
        assert_eq!(r.cv.basics.name, "María Fernanda López Ruiz");
        assert_eq!(r.cv.basics.phone, "+52 33 1234 5678");
        assert!(r.cv.skills.len() >= 13);
        assert!(r.cv.experience.is_empty());
        assert!(r.warnings[0].contains("La IA local no respondió"));
        Ok(())
    }

    #[test]
    fn un_json_valido_segun_el_esquema_se_lee_en_core() -> std::result::Result<(), serde_json::Error>
    {
        let cv = json!({
            "basics": {"name": "Ana", "email": "", "phone": "", "headline": "", "location": "", "links": []},
            "experience": [{"company": "X", "title": "Y", "start": "2023", "end": null, "bullets": ["Hice algo"]}],
            "skills": ["Excel"], "education": [], "certifications": [], "languages": []
        });
        let _: RawCv = serde_json::from_value(cv)?;
        let req = json!({
            "terms": [{"canonical": "React", "required": true}],
            "extra_skills": [{"surface": "Storybook", "required": false, "kind": "Tool"}],
            "seniority": null, "years_min": 2, "languages": ["Inglés"], "blockers": []
        });
        let _: LlmRequirements = serde_json::from_value(req)?;
        Ok(())
    }

    /// Mide la lectura de un CV realista con la IA local y muestra lo que capturó. Correr a mano:
    /// `cargo test -p finder-infra --lib banco_importacion -- --ignored --nocapture`
    #[tokio::test]
    #[ignore]
    async fn banco_importacion() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let cfg = LlmConfig::from_env()?;
        let client = LlmClient::new(cfg.local.clone())?;
        let text = include_str!("../../../core/tests/fixtures/cv_realista.txt").to_string();
        let dict = SkillDict::builtin()?;
        let _ = import_text(&client, &dict, "Ana\nEditora".into()).await; // carga el modelo en memoria
        let t = std::time::Instant::now();
        let r = import_text(&client, &dict, text).await?;
        println!(
            "tardó {:.1?}\n{:#?}\navisos: {:#?}",
            t.elapsed(),
            r.result.cv,
            r.result.warnings
        );
        Ok(())
    }

    /// Requiere Ollama con el modelo local. Correr a mano:
    /// `cargo test -p finder-infra llm -- --ignored --nocapture`
    #[tokio::test]
    #[ignore]
    async fn en_vivo_ollama_importa_cv_y_extrae_requisitos()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let cfg = LlmConfig::from_env()?;
        let client = LlmClient::new(cfg.local.clone())?;

        let dict = SkillDict::builtin()?;
        let imported = import_cv(
            &client,
            &dict,
            include_bytes!("../../tests/fixtures/cv_simple.pdf"),
            CvFileKind::Pdf,
        )
        .await?;
        println!(
            "CV: {:#?}\nAvisos: {:?}",
            imported.result.cv, imported.result.warnings
        );
        assert_eq!(imported.result.cv.basics.name, "Persona de Ejemplo");

        let jd = include_str!("../../../core/tests/fixtures/jd_editor_video_es.txt");
        let req =
            extract_requirements(&client, &dict, "Editor de Video (Remoto - LATAM)", jd).await?;
        for t in &req.terms {
            println!(
                "  [{}] {}",
                if t.required { "req" } else { "des" },
                t.surface
            );
        }
        println!(
            "seniority={:?} años={:?} idiomas={:?} bloqueantes={:?}",
            req.seniority, req.years_min, req.languages, req.blockers
        );

        let sim = semantic_similarity(
            &client,
            &cfg.embed_model,
            "Editor de video con Premiere",
            jd,
        )
        .await?;
        println!("similitud semántica: {sim:?}");
        Ok(())
    }
}
