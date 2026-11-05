//! Tareas de IA del MVP. Cada una: instrucciones + esquema JSON + validación determinista en `core`.
//!
//! - [`import_cv`]: PDF/DOCX → texto → la IA lo transcribe → `core::import::finalize` (ids y avisos).
//! - [`extract_requirements`]: diccionario + IA → `core::ats::requirements::merge` (verificado contra el anuncio).
//! - [`semantic_similarity`]: coseno entre embeddings del CV y de la vacante.
//!
//! Por privacidad, estas tareas usan el modelo **local** (el CV no sale del equipo).

use finder_core::ats::keywords::{SkillDict, extract_terms};
use finder_core::ats::requirements::{LlmRequirements, merge};
use finder_core::import::{ImportResult, RawCv, finalize};
use finder_core::matching::cosine;
use finder_core::model::JobRequirements;
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

pub fn cv_schema() -> Value {
    obj(&[
        (
            "basics",
            obj(&[
                ("name", s()),
                ("email", s()),
                ("phone", s()),
                ("headline", s()),
                ("location", s()),
                ("links", list(s())),
            ]),
        ),
        (
            "experience",
            list(obj(&[
                ("company", s()),
                ("title", s()),
                ("start", s()),
                ("end", s_or_null()),
                ("bullets", list(s())),
            ])),
        ),
        ("skills", list(s())),
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
        ("languages", list(obj(&[("name", s()), ("level", s())]))),
    ])
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

const CV_SYSTEM: &str = "Transcribes currículums a JSON. Reglas estrictas:
- Copia cada dato tal como aparece en el CV: no traduzcas, no resumas, no corrijas, no completes.
- No inventes nada. Si un dato no está, usa cadena vacía, null o lista vacía.
- Fechas: formato AAAA-MM o AAAA. Si el empleo sigue vigente, \"end\" es null.
- Cada logro o responsabilidad de un empleo es un elemento de \"bullets\", copiado literal.
- \"skills\": solo las habilidades que el CV lista explícitamente.
- \"headline\": el titular profesional que aparece bajo el nombre, si existe.";

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

pub async fn import_cv(client: &LlmClient, bytes: &[u8], kind: CvFileKind) -> Result<ImportedCv> {
    let source_text = extract_text(bytes, kind)?;
    let user = format!("CV:\n<<<\n{source_text}\n>>>");
    let value = client.complete_json(CV_SYSTEM, &user, &cv_schema()).await?;
    let raw: RawCv = serde_json::from_value(value)?;
    Ok(ImportedCv {
        result: finalize(raw, &source_text),
        source_text,
    })
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
        assert_strict(&cv_schema(), "cv");
        assert_strict(&requirements_schema(), "req");
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

    /// Requiere Ollama con el modelo local. Correr a mano:
    /// `cargo test -p finder-infra llm -- --ignored --nocapture`
    #[tokio::test]
    #[ignore]
    async fn en_vivo_ollama_importa_cv_y_extrae_requisitos()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let cfg = LlmConfig::from_env()?;
        let client = LlmClient::new(cfg.local.clone())?;

        let imported = import_cv(
            &client,
            include_bytes!("../../tests/fixtures/cv_simple.pdf"),
            CvFileKind::Pdf,
        )
        .await?;
        println!(
            "CV: {:#?}\nAvisos: {:?}",
            imported.result.cv, imported.result.warnings
        );
        assert_eq!(imported.result.cv.basics.name, "Persona de Ejemplo");

        let dict = SkillDict::builtin()?;
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
