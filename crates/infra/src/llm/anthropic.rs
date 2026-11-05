//! Claude por la Messages API (HTTP directo: no hay SDK oficial de Rust).
//!
//! - JSON garantizado con `output_config.format` (`json_schema`).
//! - `fallbacks: "default"`: si los filtros de seguridad rechazan la petición, la API la repite
//!   con otro modelo en la misma llamada (beta `server-side-fallback-2026-07-01`).
//! - En los modelos actuales el razonamiento no se puede apagar; la profundidad se controla con
//!   `output_config.effort` (por defecto "medium", configurable con `FRJ_ANTHROPIC_EFFORT`).

use reqwest::Client;
use serde_json::{Value, json};

use super::{LlmError, Result, api_error, parse_json_text};

const URL: &str = "https://api.anthropic.com/v1/messages";
const API_VERSION: &str = "2026-10-21";
const FALLBACK_BETA: &str = "server-side-fallback-2026-07-01";
/// Suficiente para un CV completo en JSON sin rozar los tiempos de espera de una petición sin streaming.
const MAX_TOKENS: u32 = 16_000;

pub(crate) fn body(model: &str, effort: &str, system: &str, user: &str, schema: &Value) -> Value {
    json!({
        "model": model,
        "max_tokens": MAX_TOKENS,
        "system": system,
        "messages": [{ "role": "user", "content": user }],
        "output_config": {
            "effort": effort,
            "format": { "type": "json_schema", "schema": schema }
        },
        "fallbacks": "default"
    })
}

pub(crate) async fn complete_json(
    http: &Client,
    api_key: &str,
    model: &str,
    effort: &str,
    system: &str,
    user: &str,
    schema: &Value,
) -> Result<Value> {
    let resp = http
        .post(URL)
        .header("x-api-key", api_key)
        .header("anthropic-version", API_VERSION)
        .header("anthropic-beta", FALLBACK_BETA)
        .json(&body(model, effort, system, user, schema))
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(api_error(resp).await);
    }
    parse_response(&resp.json::<Value>().await?)
}

/// Revisa `stop_reason` antes de leer el contenido, y junta los bloques de texto.
pub(crate) fn parse_response(v: &Value) -> Result<Value> {
    match v.get("stop_reason").and_then(Value::as_str) {
        Some("refusal") => {
            let category = v
                .pointer("/stop_details/category")
                .and_then(Value::as_str)
                .unwrap_or("sin categoría");
            return Err(LlmError::Refusal(category.to_string()));
        }
        Some("max_tokens") => return Err(LlmError::Truncated),
        _ => {}
    }
    // El contenido puede traer bloques `thinking` (vacíos) además del texto: solo interesa el texto.
    let text: String = v
        .get("content")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|b| b.get("type").and_then(Value::as_str) == Some("text"))
        .filter_map(|b| b.get("text").and_then(Value::as_str))
        .collect();
    parse_json_text(&text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cuerpo_de_la_peticion() {
        let schema = json!({"type": "object", "additionalProperties": false, "properties": {}, "required": []});
        let b = body("claude-opus-5-5", "medium", "sistema", "hola", &schema);
        assert_eq!(b["model"], "claude-opus-5-5");
        assert_eq!(b["output_config"]["format"]["type"], "json_schema");
        assert_eq!(b["output_config"]["format"]["schema"], schema);
        assert_eq!(b["output_config"]["effort"], "medium");
        assert_eq!(b["fallbacks"], "default");
        assert!(
            b.get("thinking").is_none(),
            "no se envía: en Opus 5.5 desactivarlo da 400"
        );
        assert!(
            b.get("temperature").is_none(),
            "temperature da 400 en la API 2026-10-21"
        );
    }

    #[test]
    fn lee_texto_ignorando_bloques_de_razonamiento() -> Result<()> {
        let resp = json!({
            "stop_reason": "end_turn",
            "content": [
                {"type": "thinking", "thinking": ""},
                {"type": "text", "text": "{\"ok\": true}"}
            ]
        });
        assert_eq!(parse_response(&resp)?, json!({"ok": true}));
        Ok(())
    }

    #[test]
    fn rechazo_y_corte_dan_errores_claros() {
        let refusal =
            json!({"stop_reason": "refusal", "stop_details": {"category": "cyber"}, "content": []});
        assert!(matches!(parse_response(&refusal), Err(LlmError::Refusal(c)) if c == "cyber"));
        let cut =
            json!({"stop_reason": "max_tokens", "content": [{"type": "text", "text": "{\"a\":"}]});
        assert!(matches!(parse_response(&cut), Err(LlmError::Truncated)));
    }
}
