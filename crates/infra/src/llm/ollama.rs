//! API nativa de Ollama (`/api/chat`) para el modelo local. A diferencia de la compatible con
//! OpenAI, permite apagar el razonamiento (`think: false`: con qwen3 es ~8 veces más rápido en
//! CPU) y fijar el contexto (`num_ctx`: por defecto Ollama usa 4096 tokens y recorta en silencio
//! el principio de los prompts largos, que es donde están las instrucciones).

use reqwest::Client;
use serde_json::{Value, json};

use super::{LlmError, Result, api_error, parse_json_text};

/// Cabe el prompt del redactor (vacante + CV) y su respuesta.
const NUM_CTX: u32 = 8192;
const NUM_PREDICT: u32 = 4096;

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
    let full = |m: &str| {
        if m.contains(':') {
            m.to_string()
        } else {
            format!("{m}:latest")
        }
    };
    installed.iter().any(|m| full(m) == full(model))
}

/// Descarga `model` en Ollama (`/api/pull`) y avisa el avance (bytes bajados, total).
/// Usa su propio cliente: la descarga (~5 GB) dura más que cualquier tiempo límite razonable.
pub async fn pull(base_url: &str, model: &str, mut progress: impl FnMut(u64, u64)) -> Result<()> {
    let mut resp = Client::new()
        .post(format!("{}/api/pull", root(base_url)))
        .json(&json!({ "model": model, "stream": true }))
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(api_error(resp).await);
    }
    // Ollama manda una línea JSON por evento; un trozo de red puede cortar una línea a la mitad.
    let mut buf = Vec::new();
    while let Some(chunk) = resp.chunk().await? {
        buf.extend_from_slice(&chunk);
        while let Some(i) = buf.iter().position(|&b| b == b'\n') {
            let line: Vec<u8> = buf.drain(..=i).collect();
            pull_line(&line, &mut progress)?;
        }
    }
    pull_line(&buf, &mut progress)
}

/// Descarga `url` a `dest` (el instalador de Ollama) y avisa el avance (bytes bajados, total).
pub async fn download(
    url: &str,
    dest: &std::path::Path,
    mut progress: impl FnMut(u64, u64),
) -> Result<()> {
    use std::io::Write;
    let mut resp = Client::new().get(url).send().await?.error_for_status()?;
    let total = resp.content_length().unwrap_or(0);
    let io =
        |e: std::io::Error| LlmError::Config(format!("no se pudo guardar {}: {e}", dest.display()));
    // shortcut: escritura bloqueante por trozo (son de pocos KB a disco local); pasar a tokio::fs si se nota.
    let mut file = std::fs::File::create(dest).map_err(io)?;
    let mut done = 0;
    while let Some(chunk) = resp.chunk().await? {
        file.write_all(&chunk).map_err(io)?;
        done += chunk.len() as u64;
        progress(done, total);
    }
    file.flush().map_err(io)
}

fn pull_line(line: &[u8], progress: &mut impl FnMut(u64, u64)) -> Result<()> {
    let Ok(v) = serde_json::from_slice::<Value>(line) else {
        return Ok(()); // línea vacía
    };
    if let Some(e) = v["error"].as_str() {
        return Err(LlmError::Api {
            status: reqwest::StatusCode::BAD_GATEWAY,
            message: e.to_string(),
        });
    }
    if let (Some(done), Some(total)) = (v["completed"].as_u64(), v["total"].as_u64()) {
        progress(done, total);
    }
    Ok(())
}

/// Sin esto, el modelo sangra el JSON: los espacios y saltos de línea son ~35 % de lo que genera
/// (y en CPU, generar es lo lento).
const COMPACT: &str =
    "\n- Responde con JSON compacto en una sola línea, sin saltos de línea ni sangría.";

pub(crate) fn chat_body(model: &str, system: &str, user: &str, schema: &Value) -> Value {
    json!({
        "model": model,
        "messages": [
            { "role": "system", "content": format!("{system}{COMPACT}") },
            { "role": "user", "content": user }
        ],
        "format": schema,
        "stream": false,
        "think": false,
        // El modelo se queda cargado entre tareas (importar, analizar vacantes, adaptar): cargarlo
        // de nuevo en CPU cuesta varios segundos cada vez.
        "keep_alive": "30m",
        // Tope de respuesta: si el modelo entra en un bucle (repite «]}}» sin fin) se corta aquí en
        // vez de seguir hasta el timeout. La mitad del contexto: sobra para un CV adaptado completo.
        "options": { "temperature": 0.1, "num_ctx": NUM_CTX, "num_predict": NUM_PREDICT }
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
        assert_eq!(b["keep_alive"], "30m");
        assert!(
            b["messages"][0]["content"]
                .as_str()
                .is_some_and(|c| c.ends_with("sin saltos de línea ni sangría."))
        );
        assert_eq!(b["options"]["num_ctx"], NUM_CTX);
        assert_eq!(b["options"]["num_predict"], NUM_PREDICT);
        assert_eq!(b["format"]["type"], "object");
    }

    #[test]
    fn avance_de_descarga() {
        let mut seen = vec![];
        let mut f = |d, t| seen.push((d, t));
        assert!(pull_line(br#"{"status":"pulling manifest"}"#, &mut f).is_ok());
        assert!(
            pull_line(
                br#"{"status":"pulling a1","completed":5,"total":10}"#,
                &mut f
            )
            .is_ok()
        );
        assert!(pull_line(b"", &mut f).is_ok());
        assert!(
            pull_line(
                br#"{"error":"pull model manifest: file does not exist"}"#,
                &mut f
            )
            .is_err()
        );
        assert_eq!(seen, vec![(5, 10)]);
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
