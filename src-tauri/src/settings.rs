//! Ajustes de IA desde la interfaz (plan Fase 2): proveedor del redactor, modelos y URLs en la
//! tabla `settings`; las claves de API en el llavero del sistema (KWallet / GNOME Keyring /
//! Credential Manager / Keychain), nunca en la base de datos ni en texto plano.
//!
//! Prioridad de cada valor: variable de entorno > `.env` > lo guardado aquí > valor por defecto.

use std::collections::HashMap;

use finder_infra::db::Db;
use finder_infra::letter;
use finder_infra::llm::{Backend, LlmConfig};
use finder_infra::optimizer::{self, TokenEstimate};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{AppState, CmdResult, err};

const KEYRING_SERVICE: &str = "FinderRemoteJob";
const PREFIX: &str = "llm.";
/// Valores normales (se guardan en la base de datos).
const PLAIN: [&str; 12] = [
    "FRJ_WRITER",
    "FRJ_LOCAL_URL",
    "FRJ_LOCAL_MODEL",
    "FRJ_EMBED_MODEL",
    "FRJ_ANTHROPIC_MODEL",
    "FRJ_ANTHROPIC_EFFORT",
    "FRJ_OPENAI_URL",
    "FRJ_OPENAI_MODEL",
    // Precios en USD por millón de tokens (los escribe el usuario; solo sirven para el estimado).
    "FRJ_ANTHROPIC_PRICE_IN",
    "FRJ_ANTHROPIC_PRICE_OUT",
    "FRJ_OPENAI_PRICE_IN",
    "FRJ_OPENAI_PRICE_OUT",
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
        if k.contains("_PRICE_") && !v.is_empty() && price(v).is_none() {
            return Err(format!(
                "Precio inválido: «{v}». Escribe un número en USD por millón de tokens (p. ej. 3 o 0.25)."
            ));
        }
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

fn price(v: &str) -> Option<f64> {
    v.replace(',', ".")
        .parse::<f64>()
        .ok()
        .filter(|p| p.is_finite() && *p >= 0.0)
}

#[derive(Debug, Serialize)]
pub struct Cost {
    #[serde(flatten)]
    tokens: TokenEstimate,
    /// Sin precios en Ajustes, solo se muestran los tokens.
    usd: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct CostView {
    cv: Cost,
    letter: Cost,
}

/// Costo aproximado de la propuesta de CV y de la carta con el redactor externo en uso.
/// `None` si el redactor es local (no cuesta).
#[tauri::command]
pub async fn estimate_cost(
    state: State<'_, AppState>,
    job_id: String,
    profile_id: String,
) -> CmdResult<Option<CostView>> {
    let prefix = match state.llm()?.writer.backend {
        Backend::Ollama { .. } => return Ok(None),
        Backend::Anthropic { .. } => "FRJ_ANTHROPIC",
        Backend::OpenAiCompat { .. } => "FRJ_OPENAI",
    };
    let job = state
        .db
        .get_job(&job_id)
        .await
        .map_err(err)?
        .ok_or("La vacante ya no existe.")?;
    let Some((_, master)) = crate::jobs::profile_and_cv(&state, Some(&profile_id)).await? else {
        return Ok(None);
    };
    let stored: HashMap<String, String> = state
        .db
        .settings_with_prefix(PREFIX)
        .await
        .map_err(err)?
        .into_iter()
        .collect();
    let prices = ["IN", "OUT"].map(|side| {
        let key = format!("{prefix}_PRICE_{side}");
        external(&state.file_vars, &key)
            .or_else(|| stored.get(&key).cloned())
            .as_deref()
            .and_then(price)
    });
    let cost = |tokens: TokenEstimate| Cost {
        usd: match prices {
            [Some(i), Some(o)] => {
                Some((f64::from(tokens.input) * i + f64::from(tokens.output) * o) / 1e6)
            }
            _ => None,
        },
        tokens,
    };
    Ok(Some(CostView {
        cv: cost(optimizer::estimate_tokens(&master, &job).map_err(err)?),
        letter: cost(letter::estimate_tokens(&master, &job).map_err(err)?),
    }))
}

#[cfg(test)]
mod tests {
    #[test]
    fn precios_con_coma_o_punto() {
        assert_eq!(super::price("0,25"), Some(0.25));
        assert_eq!(super::price("3"), Some(3.0));
        assert_eq!(super::price("-1"), None);
        assert_eq!(super::price("tres"), None);
    }

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
