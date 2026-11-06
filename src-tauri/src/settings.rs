//! Ajustes de IA desde la interfaz (plan Fase 2): proveedor del redactor, modelos y URLs en la
//! tabla `settings`; las claves de API en el llavero del sistema (KWallet / GNOME Keyring /
//! Credential Manager / Keychain), nunca en la base de datos ni en texto plano.
//!
//! Prioridad de cada valor: variable de entorno > `.env` > lo guardado aquí > valor por defecto.

use std::collections::HashMap;

use finder_infra::db::Db;
use finder_infra::llm::{Backend, LlmConfig};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{AppState, CmdResult, err};

const KEYRING_SERVICE: &str = "FinderRemoteJob";
const PREFIX: &str = "llm.";
/// Valores normales (se guardan en la base de datos).
const PLAIN: [&str; 8] = [
    "FRJ_WRITER",
    "FRJ_LOCAL_URL",
    "FRJ_LOCAL_MODEL",
    "FRJ_EMBED_MODEL",
    "FRJ_ANTHROPIC_MODEL",
    "FRJ_ANTHROPIC_EFFORT",
    "FRJ_OPENAI_URL",
    "FRJ_OPENAI_MODEL",
];
/// Claves de API (se guardan en el llavero).
const SECRETS: [&str; 2] = ["ANTHROPIC_API_KEY", "FRJ_OPENAI_API_KEY"];

fn entry(key: &str) -> keyring::Result<keyring::Entry> {
    keyring::Entry::new(KEYRING_SERVICE, key)
}

fn read_secret(key: &str) -> Option<String> {
    entry(key).ok()?.get_password().ok()
}

/// Valores que vienen del entorno o del `.env` (mandan sobre lo guardado en la app).
fn external(file_vars: &HashMap<String, String>, key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .or_else(|| file_vars.get(key).cloned())
        .filter(|v| !v.trim().is_empty())
}

/// Configuración vigente con todas las fuentes.
pub(crate) async fn load(
    db: &Db,
    file_vars: &HashMap<String, String>,
) -> Result<LlmConfig, String> {
    let stored: HashMap<String, String> = db
        .settings_with_prefix(PREFIX)
        .await
        .map_err(err)?
        .into_iter()
        .collect();
    LlmConfig::from_lookup(|k| {
        external(file_vars, k)
            .or_else(|| stored.get(k).cloned())
            .or_else(|| SECRETS.contains(&k).then(|| read_secret(k)).flatten())
    })
    .map_err(err)
}

#[derive(Debug, Serialize)]
pub struct SettingsView {
    /// Lo guardado en la app (sin claves).
    values: HashMap<String, String>,
    /// Qué claves hay en el llavero.
    saved_keys: Vec<String>,
    /// Ajustes fijados por el entorno o el `.env`: ganan sobre lo de la app.
    overridden: Vec<String>,
    env_file: Option<String>,
    /// Configuración en uso ("Claude · claude-opus-5-5"), o el error si es inválida.
    writer: Option<String>,
    local: Option<String>,
    error: Option<String>,
    /// Si el llavero del sistema no está disponible (las claves no se pueden guardar).
    keyring_error: Option<String>,
}

#[tauri::command]
pub async fn get_llm_settings(state: State<'_, AppState>) -> CmdResult<SettingsView> {
    view(&state).await
}

async fn view(state: &AppState) -> CmdResult<SettingsView> {
    let values = state
        .db
        .settings_with_prefix(PREFIX)
        .await
        .map_err(err)?
        .into_iter()
        .collect();
    let keyring_error = match entry("probe").and_then(|e| e.get_password()) {
        Ok(_) | Err(keyring::Error::NoEntry) => None,
        Err(e) => Some(e.to_string()),
    };
    let config = state.llm.read().map_err(err)?.clone();
    Ok(SettingsView {
        values,
        saved_keys: SECRETS
            .iter()
            .filter(|k| read_secret(k).is_some())
            .map(|k| k.to_string())
            .collect(),
        overridden: PLAIN
            .iter()
            .chain(&SECRETS)
            .filter(|k| external(&state.file_vars, k).is_some())
            .map(|k| k.to_string())
            .collect(),
        env_file: state.info.env_file.clone(),
        writer: config.as_ref().ok().map(|c| describe(&c.writer)),
        local: config.as_ref().ok().map(|c| describe(&c.local)),
        error: config.err(),
        keyring_error,
    })
}

pub(crate) fn describe(e: &finder_infra::llm::Endpoint) -> String {
    let who = match &e.backend {
        Backend::Ollama { .. } => "Local (Ollama)",
        Backend::OpenAiCompat { .. } => "Compatible con OpenAI",
        Backend::Anthropic { .. } => "Claude (Anthropic)",
    };
    format!("{who} · {}", e.model)
}

#[derive(Debug, Deserialize)]
pub struct SettingsInput {
    /// Valores normales; vacío = volver al valor por defecto.
    values: HashMap<String, String>,
    /// Claves nuevas; las que no vienen (o vienen vacías) no se tocan.
    keys: HashMap<String, String>,
}

#[tauri::command]
pub async fn save_llm_settings(
    state: State<'_, AppState>,
    input: SettingsInput,
) -> CmdResult<SettingsView> {
    for (k, v) in &input.values {
        if !PLAIN.contains(&k.as_str()) {
            return Err(format!("Ajuste desconocido: {k}"));
        }
        let v = v.trim();
        state
            .db
            .set_setting(&format!("{PREFIX}{k}"), (!v.is_empty()).then_some(v))
            .await
            .map_err(err)?;
    }
    for (k, v) in &input.keys {
        if !SECRETS.contains(&k.as_str()) {
            return Err(format!("Clave desconocida: {k}"));
        }
        let v = v.trim();
        if !v.is_empty() {
            entry(k).and_then(|e| e.set_password(v)).map_err(|e| {
                format!("No se pudo guardar la clave en el llavero del sistema: {e}")
            })?;
        }
    }
    reload(&state).await
}

/// Borra una clave de API del llavero.
#[tauri::command]
pub async fn delete_api_key(state: State<'_, AppState>, key: String) -> CmdResult<SettingsView> {
    if !SECRETS.contains(&key.as_str()) {
        return Err(format!("Clave desconocida: {key}"));
    }
    match entry(&key).and_then(|e| e.delete_credential()) {
        Ok(()) | Err(keyring::Error::NoEntry) => reload(&state).await,
        Err(e) => Err(format!("No se pudo borrar la clave: {e}")),
    }
}

async fn reload(state: &AppState) -> CmdResult<SettingsView> {
    let config = load(&state.db, &state.file_vars).await;
    *state.llm.write().map_err(err)? = config;
    view(state).await
}

#[cfg(test)]
mod tests {
    /// Toca el llavero real del sistema: `cargo test -p finder-app llavero -- --ignored`.
    #[test]
    #[ignore]
    fn llavero_guarda_lee_y_borra() -> keyring::Result<()> {
        let e = super::entry("prueba_finder")?;
        e.set_password("secreto")?;
        assert_eq!(e.get_password()?, "secreto");
        e.delete_credential()?;
        assert!(matches!(e.get_password(), Err(keyring::Error::NoEntry)));
        Ok(())
    }
}
