//! API nativa de Ollama (`/api/chat`) para el modelo local. A diferencia de la compatible con
//! OpenAI, permite apagar el razonamiento (`think: false`: con qwen3 es ~8 veces más rápido en
//! CPU) y fijar el contexto (`num_ctx`: por defecto Ollama usa 4096 tokens y recorta en silencio
//! el principio de los prompts largos, que es donde están las instrucciones).

use reqwest::Client;
use serde_json::{Value, json};

use super::{LlmError, Result, api_error, parse_json_text};

/// Cabe el prompt del redactor (vacante + CV) y su respuesta.
const NUM_CTX: u32 = 8192;

/// `FRJ_LOCAL_URL` apunta a la API compatible (`…/v1`); la nativa está en la raíz.
fn root(base_url: &str) -> &str {
    let root = base_url.trim_end_matches('/');
    root.strip_suffix("/v1").unwrap_or(root)
}

fn chat_url(base_url: &str) -> String {
    format!("{}/api/chat", root(base_url))
}

/// Modelos descargados en Ollama (`/api/tags`). Falla si Ollama no está abierto.
pub async fn installed_models(http: &Client, base_url: &str) -> Result<Vec<String>> {
    let v: Value = http
        .get(format!("{}/api/tags", root(base_url)))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(v["models"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|m| m["name"].as_str().map(str::to_string))
        .collect())
}

/// Ollama nombra `qwen3` como `qwen3:latest`.
pub fn has_model(installed: &[String], model: &str) -> bool {
    let full = |m: &str| if m.contains(':') { m.to_string() } else { format!("{m}:latest") };
    installed.iter().any(|m| full(m) == full(model))
}

pub(crate) fn chat_body(model: &str, system: &str, user: &str, schema: &Value) -> Value {
    json!({
        "model": model,
        "messages": [
            { "role": "system", "content": system },
            { "role": "user", "content": user }
        ],
        "format": schema,
        "stream": false,
        "think": false,
        "options": { "temperature": 0.1, "num_ctx": NUM_CTX }
    })
}

pub(crate) async fn complete_json(
    http: &Client,
    base_url: &str,
    model: &str,
    system: &str,
    user: &str,
    schema: &Value,
) -> Result<Value> {
    let resp = http
        .post(chat_url(base_url))
        .json(&chat_body(model, system, user, schema))
        .send()
        .await?;
    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(LlmError::ModelNotFound(model.to_string()));
    }
    if !resp.status().is_success() {
        return Err(api_error(resp).await);
    }
    parse_chat(&resp.json::<Value>().await?)
}

pub(crate) fn parse_chat(v: &Value) -> Result<Value> {
    if v.get("done_reason").and_then(Value::as_str) == Some("length") {
        return Err(LlmError::Truncated);
    }
    let text = v
        .pointer("/message/content")
        .and_then(Value::as_str)
        .ok_or_else(|| LlmError::InvalidJson("respuesta sin «message»".into()))?;
    parse_json_text(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modelo_instalado_con_o_sin_etiqueta() {
        let m = vec!["qwen3:8b".to_string(), "bge-m3:latest".to_string()];
        assert!(has_model(&m, "qwen3:8b"));
        assert!(has_model(&m, "bge-m3"));
        assert!(!has_model(&m, "qwen3"));
        assert!(!has_model(&m, "qwen3:14b"));
    }

    #[test]
    fn url_y_cuerpo() {
        assert_eq!(
            chat_url("http://localhost:11434/v1/"),
            "http://localhost:11434/api/chat"
        );
        assert_eq!(chat_url("http://pc:11434"), "http://pc:11434/api/chat");
        let b = chat_body("qwen3:8b", "s", "u", &json!({"type": "object"}));
        assert_eq!(b["think"], false);
        assert_eq!(b["options"]["num_ctx"], NUM_CTX);
        assert_eq!(b["format"]["type"], "object");
    }

    #[test]
    fn lee_respuesta() -> Result<()> {
        let ok = json!({"message": {"content": "{\"x\": 1}"}, "done_reason": "stop"});
        assert_eq!(parse_chat(&ok)?, json!({"x": 1}));
        let cut = json!({"message": {"content": "{"}, "done_reason": "length"});
        assert!(matches!(parse_chat(&cut), Err(LlmError::Truncated)));
        assert!(parse_chat(&json!({})).is_err());
        Ok(())
    }
}
