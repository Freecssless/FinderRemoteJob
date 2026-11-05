//! Cliente de IA (plan §2, "Integración de IA híbrida").
//!
//! Dos papeles, cada uno con su propio backend:
//! - **local** (Ollama, compatible con OpenAI): extrae requisitos de vacantes, transcribe el CV
//!   importado y calcula embeddings. Nada sale del equipo.
//! - **redactor** (`writer`): reescribe CV y cartas (módulo 9). Por defecto Anthropic si hay clave;
//!   si no, el mismo modelo local.
//!
//! Dos implementaciones cubren todos los proveedores:
//! [`openai_compat`] (Ollama, OpenAI, Gemini, Groq, OpenRouter…) y [`anthropic`] (Claude).
//! Se usa un `enum` en vez de un trait con `dyn`: los métodos async en traits no son
//! "dyn-compatibles" y con dos variantes no hace falta más.
//!
//! Configuración por variables de entorno (ver `.env.example` y el README).

pub mod anthropic;
pub mod openai_compat;
pub mod tasks;

use std::time::Duration;

use reqwest::{Client, StatusCode};
use serde_json::Value;

#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("configuración de IA inválida: {0}")]
    Config(String),
    #[error("no se pudo conectar con la IA ({0}). ¿Está corriendo Ollama?")]
    Http(#[from] reqwest::Error),
    #[error("la IA respondió {status}: {message}")]
    Api { status: StatusCode, message: String },
    #[error("el modelo se negó a responder ({0})")]
    Refusal(String),
    #[error("la respuesta se cortó por longitud; intenta con un texto más corto")]
    Truncated,
    #[error("la IA no devolvió un JSON válido: {0}")]
    InvalidJson(String),
    #[error("este proveedor no ofrece embeddings")]
    NoEmbeddings,
}

impl LlmError {
    /// ¿Vale la pena reintentar más tarde? (límite de uso, sobrecarga o red)
    /// shortcut: no hay reintento automático; la interfaz ofrece "Reintentar". Agregar backoff si molesta.
    pub fn is_retryable(&self) -> bool {
        match self {
            LlmError::Http(_) => true,
            LlmError::Api { status, .. } => {
                status.as_u16() == 429 || status.as_u16() == 529 || status.is_server_error()
            }
            _ => false,
        }
    }
}

pub type Result<T> = std::result::Result<T, LlmError>;

#[derive(Debug, Clone, PartialEq)]
pub enum Backend {
    OpenAiCompat {
        base_url: String,
        api_key: Option<String>,
    },
    Anthropic {
        api_key: String,
        effort: String,
    },
}

/// Un modelo concreto en un backend concreto.
#[derive(Debug, Clone, PartialEq)]
pub struct Endpoint {
    pub backend: Backend,
    pub model: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LlmConfig {
    pub local: Endpoint,
    pub embed_model: String,
    pub writer: Endpoint,
}

pub const DEFAULT_LOCAL_URL: &str = "http://localhost:11434/v1";
pub const DEFAULT_LOCAL_MODEL: &str = "qwen3:8b";
pub const DEFAULT_EMBED_MODEL: &str = "bge-m3";
pub const DEFAULT_ANTHROPIC_MODEL: &str = "claude-opus-5-5";
pub const DEFAULT_ANTHROPIC_EFFORT: &str = "medium";

impl LlmConfig {
    /// Lee la configuración de las variables de entorno del proceso.
    pub fn from_env() -> Result<Self> {
        Self::from_lookup(|k| std::env::var(k).ok())
    }

    /// Igual que [`LlmConfig::from_env`], con una función de búsqueda inyectable (para pruebas).
    pub fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Result<Self> {
        let get = |k: &str| {
            get(k)
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
        };
        let local = Endpoint {
            backend: Backend::OpenAiCompat {
                base_url: get("FRJ_LOCAL_URL").unwrap_or_else(|| DEFAULT_LOCAL_URL.into()),
                api_key: None,
            },
            model: get("FRJ_LOCAL_MODEL").unwrap_or_else(|| DEFAULT_LOCAL_MODEL.into()),
        };
        let anthropic_key = get("ANTHROPIC_API_KEY");
        let writer_kind = get("FRJ_WRITER").unwrap_or_else(|| {
            if anthropic_key.is_some() {
                "anthropic".into()
            } else {
                "local".into()
            }
        });
        let writer = match writer_kind.as_str() {
            "local" => local.clone(),
            "anthropic" => Endpoint {
                backend: Backend::Anthropic {
                    api_key: anthropic_key.ok_or_else(|| {
                        LlmError::Config("FRJ_WRITER=anthropic requiere ANTHROPIC_API_KEY".into())
                    })?,
                    effort: get("FRJ_ANTHROPIC_EFFORT")
                        .unwrap_or_else(|| DEFAULT_ANTHROPIC_EFFORT.into()),
                },
                model: get("FRJ_ANTHROPIC_MODEL").unwrap_or_else(|| DEFAULT_ANTHROPIC_MODEL.into()),
            },
            "openai" => Endpoint {
                backend: Backend::OpenAiCompat {
                    base_url: get("FRJ_OPENAI_URL").ok_or_else(|| {
                        LlmError::Config("FRJ_WRITER=openai requiere FRJ_OPENAI_URL".into())
                    })?,
                    api_key: get("FRJ_OPENAI_API_KEY"),
                },
                model: get("FRJ_OPENAI_MODEL").ok_or_else(|| {
                    LlmError::Config("FRJ_WRITER=openai requiere FRJ_OPENAI_MODEL".into())
                })?,
            },
            other => {
                return Err(LlmError::Config(format!(
                    "FRJ_WRITER=«{other}» no es válido (usa local, anthropic u openai)"
                )));
            }
        };
        Ok(Self {
            local,
            embed_model: get("FRJ_EMBED_MODEL").unwrap_or_else(|| DEFAULT_EMBED_MODEL.into()),
            writer,
        })
    }
}

/// Cliente para un [`Endpoint`].
#[derive(Debug, Clone)]
pub struct LlmClient {
    http: Client,
    pub endpoint: Endpoint,
}

impl LlmClient {
    pub fn new(endpoint: Endpoint) -> Result<Self> {
        // Los modelos locales pueden tardar minutos en CPU: tiempo de espera amplio.
        let http = Client::builder()
            .timeout(Duration::from_secs(600))
            .build()?;
        Ok(Self { http, endpoint })
    }

    /// Pide una respuesta JSON que cumpla `schema` (JSON Schema con `additionalProperties: false`).
    pub async fn complete_json(&self, system: &str, user: &str, schema: &Value) -> Result<Value> {
        match &self.endpoint.backend {
            Backend::OpenAiCompat { base_url, api_key } => {
                openai_compat::complete_json(
                    &self.http,
                    base_url,
                    api_key.as_deref(),
                    &self.endpoint.model,
                    system,
                    user,
                    schema,
                )
                .await
            }
            Backend::Anthropic { api_key, effort } => {
                anthropic::complete_json(
                    &self.http,
                    api_key,
                    &self.endpoint.model,
                    effort,
                    system,
                    user,
                    schema,
                )
                .await
            }
        }
    }

    /// Embeddings de cada texto, con `model` (solo backends compatibles con OpenAI, como Ollama).
    pub async fn embed(&self, model: &str, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        match &self.endpoint.backend {
            Backend::OpenAiCompat { base_url, api_key } => {
                openai_compat::embed(&self.http, base_url, api_key.as_deref(), model, texts).await
            }
            Backend::Anthropic { .. } => Err(LlmError::NoEmbeddings),
        }
    }
}

/// Convierte el texto de la respuesta en JSON. Tolera lo que algunos modelos locales agregan:
/// un bloque `<think>…</think>` o el JSON dentro de ```json … ```.
pub(crate) fn parse_json_text(text: &str) -> Result<Value> {
    let text = match text.find("</think>") {
        Some(i) => &text[i + "</think>".len()..],
        None => text,
    };
    let trimmed = text.trim();
    if let Ok(v) = serde_json::from_str(trimmed) {
        return Ok(v);
    }
    // Último recurso: del primer "{" al último "}".
    match (trimmed.find('{'), trimmed.rfind('}')) {
        (Some(a), Some(b)) if a < b => {
            serde_json::from_str(&trimmed[a..=b]).map_err(|e| LlmError::InvalidJson(e.to_string()))
        }
        _ => Err(LlmError::InvalidJson(format!(
            "sin objeto JSON en: «{}»",
            trimmed.chars().take(120).collect::<String>()
        ))),
    }
}

/// Mensaje de error legible a partir del cuerpo de una respuesta fallida.
pub(crate) async fn api_error(resp: reqwest::Response) -> LlmError {
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    let message = serde_json::from_str::<Value>(&body)
        .ok()
        .and_then(|v| {
            v.pointer("/error/message")
                .or_else(|| v.pointer("/error"))
                .and_then(Value::as_str)
                .map(String::from)
        })
        .unwrap_or_else(|| body.chars().take(300).collect());
    LlmError::Api { status, message }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn cfg(vars: &[(&str, &str)]) -> Result<LlmConfig> {
        let map: HashMap<String, String> = vars
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        LlmConfig::from_lookup(|k| map.get(k).cloned())
    }

    #[test]
    fn sin_variables_todo_es_local() -> Result<()> {
        let c = cfg(&[])?;
        assert_eq!(c.local.model, DEFAULT_LOCAL_MODEL);
        assert_eq!(c.writer, c.local);
        assert_eq!(c.embed_model, DEFAULT_EMBED_MODEL);
        Ok(())
    }

    #[test]
    fn con_clave_de_anthropic_el_redactor_es_claude() -> Result<()> {
        let c = cfg(&[("ANTHROPIC_API_KEY", "sk-test")])?;
        assert_eq!(c.writer.model, DEFAULT_ANTHROPIC_MODEL);
        assert!(
            matches!(c.writer.backend, Backend::Anthropic { ref effort, .. } if effort == "medium")
        );
        // Se puede forzar el local aunque haya clave.
        let c = cfg(&[("ANTHROPIC_API_KEY", "sk-test"), ("FRJ_WRITER", "local")])?;
        assert_eq!(c.writer, c.local);
        Ok(())
    }

    #[test]
    fn errores_de_configuracion_claros() {
        assert!(
            matches!(cfg(&[("FRJ_WRITER", "anthropic")]), Err(LlmError::Config(m)) if m.contains("ANTHROPIC_API_KEY"))
        );
        assert!(
            matches!(cfg(&[("FRJ_WRITER", "openai")]), Err(LlmError::Config(m)) if m.contains("FRJ_OPENAI_URL"))
        );
        assert!(matches!(
            cfg(&[("FRJ_WRITER", "gpt")]),
            Err(LlmError::Config(_))
        ));
        // Variables vacías = no definidas.
        assert!(cfg(&[("ANTHROPIC_API_KEY", "  ")]).is_ok_and(|c| c.writer == c.local));
    }

    #[test]
    fn json_tolerante_a_think_y_bloques_de_codigo() -> Result<()> {
        assert_eq!(parse_json_text("{\"a\":1}")?, serde_json::json!({"a": 1}));
        assert_eq!(
            parse_json_text("<think>pienso…</think>\n{\"a\":2}")?,
            serde_json::json!({"a": 2})
        );
        assert_eq!(
            parse_json_text("```json\n{\"a\":3}\n```")?,
            serde_json::json!({"a": 3})
        );
        assert!(matches!(
            parse_json_text("no hay json"),
            Err(LlmError::InvalidJson(_))
        ));
        Ok(())
    }

    #[test]
    fn errores_reintentables() {
        let api = |s: u16| LlmError::Api {
            status: StatusCode::from_u16(s).unwrap_or(StatusCode::OK),
            message: String::new(),
        };
        assert!(api(429).is_retryable() && api(529).is_retryable() && api(500).is_retryable());
        assert!(!api(400).is_retryable() && !LlmError::Truncated.is_retryable());
    }
}
