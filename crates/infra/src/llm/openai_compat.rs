//! Backends compatibles con la API de OpenAI: Ollama (local), OpenAI, Gemini, Groq, OpenRouter…
//! JSON con `response_format: json_schema` (Ollama lo soporta desde la versión 0.5).

use reqwest::Client;
use serde_json::{Value, json};

use super::{LlmError, Result, api_error, parse_json_text};

pub(crate) fn chat_body(model: &str, system: &str, user: &str, schema: &Value) -> Value {
    json!({
        "model": model,
        "messages": [
            { "role": "system", "content": system },
            { "role": "user", "content": user }
        ],
        "response_format": {
            "type": "json_schema",
            "json_schema": { "name": "respuesta", "strict": true, "schema": schema }
        },
        // Transcribir y clasificar no necesita creatividad.
        "temperature": 0.1
    })
}

fn url(base_url: &str, path: &str) -> String {
    format!("{}/{path}", base_url.trim_end_matches('/'))
}

fn with_key(req: reqwest::RequestBuilder, api_key: Option<&str>) -> reqwest::RequestBuilder {
    match api_key {
        Some(k) => req.bearer_auth(k),
        None => req,
    }
}

pub(crate) async fn complete_json(
    http: &Client,
    base_url: &str,
    api_key: Option<&str>,
    model: &str,
    system: &str,
    user: &str,
    schema: &Value,
) -> Result<Value> {
    let req = http
        .post(url(base_url, "chat/completions"))
        .json(&chat_body(model, system, user, schema));
    let resp = with_key(req, api_key).send().await?;
    if !resp.status().is_success() {
        return Err(fail(resp, model).await);
    }
    parse_chat(&resp.json::<Value>().await?)
}

/// Ollama responde 404 cuando el modelo no está descargado: se explica qué hacer.
async fn fail(resp: reqwest::Response, model: &str) -> LlmError {
    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return LlmError::ModelNotFound(model.to_string());
    }
    api_error(resp).await
}

pub(crate) fn parse_chat(v: &Value) -> Result<Value> {
    let choice = v
        .pointer("/choices/0")
        .ok_or_else(|| LlmError::InvalidJson("respuesta sin «choices»".into()))?;
    match choice.get("finish_reason").and_then(Value::as_str) {
        Some("length") => return Err(LlmError::Truncated),
        Some("content_filter") => return Err(LlmError::Refusal("content_filter".into())),
        _ => {}
    }
    let text = choice
        .pointer("/message/content")
        .and_then(Value::as_str)
        .unwrap_or_default();
    parse_json_text(text)
}

pub(crate) async fn embed(
    http: &Client,
    base_url: &str,
    api_key: Option<&str>,
    model: &str,
    texts: &[String],
) -> Result<Vec<Vec<f32>>> {
    let req = http
        .post(url(base_url, "embeddings"))
        .json(&json!({ "model": model, "input": texts }));
    let resp = with_key(req, api_key).send().await?;
    if !resp.status().is_success() {
        return Err(fail(resp, model).await);
    }
    parse_embeddings(&resp.json::<Value>().await?, texts.len())
}

pub(crate) fn parse_embeddings(v: &Value, expected: usize) -> Result<Vec<Vec<f32>>> {
    let data = v
        .get("data")
        .and_then(Value::as_array)
        .ok_or_else(|| LlmError::InvalidJson("respuesta sin «data»".into()))?;
    let out: Vec<Vec<f32>> = data
        .iter()
        .map(|d| {
            d.get("embedding")
                .and_then(Value::as_array)
                .map(|e| {
                    e.iter()
                        .filter_map(Value::as_f64)
                        .map(|f| f as f32)
                        .collect()
                })
                .unwrap_or_default()
        })
        .collect();
    if out.len() != expected || out.iter().any(Vec::is_empty) {
        return Err(LlmError::InvalidJson(format!(
            "se esperaban {expected} embeddings"
        )));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cuerpo_y_url() {
        let b = chat_body("qwen3:8b", "s", "u", &json!({"type": "object"}));
        assert_eq!(b["response_format"]["json_schema"]["strict"], true);
        assert_eq!(b["messages"][0]["role"], "system");
        assert_eq!(
            url("http://localhost:11434/v1/", "chat/completions"),
            "http://localhost:11434/v1/chat/completions"
        );
    }

    #[test]
    fn lee_respuesta_de_chat() -> Result<()> {
        let ok =
            json!({"choices": [{"finish_reason": "stop", "message": {"content": "{\"x\": 1}"}}]});
        assert_eq!(parse_chat(&ok)?, json!({"x": 1}));
        let cut = json!({"choices": [{"finish_reason": "length", "message": {"content": "{"}}]});
        assert!(matches!(parse_chat(&cut), Err(LlmError::Truncated)));
        assert!(parse_chat(&json!({})).is_err());
        Ok(())
    }

    #[test]
    fn lee_embeddings() -> Result<()> {
        let v = json!({"data": [{"embedding": [0.1, 0.2]}, {"embedding": [0.3, 0.4]}]});
        assert_eq!(parse_embeddings(&v, 2)?.len(), 2);
        assert!(parse_embeddings(&v, 3).is_err());
        Ok(())
    }
}
