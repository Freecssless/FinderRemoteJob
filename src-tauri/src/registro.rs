//! Registro de errores (`<datos>/registro.log`), reporte de problemas y borrado de todos los datos.
//! Nada sale del equipo solo: el usuario ve el reporte y decide si lo publica en GitHub.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::{CmdResult, err};

const FILE_NAME: &str = "registro.log";
/// Al pasar de aquí se conserva la mitad más reciente.
const MAX_BYTES: u64 = 512 * 1024;
/// Marca que deja «Borrar todos mis datos»; se aplica al arrancar, antes de abrir la base.
const DELETE_MARK: &str = "borrar-todo";

static LOG: OnceLock<PathBuf> = OnceLock::new();

/// Empieza a registrar en `data_dir` y anota los fallos internos (pánicos) antes de que ocurran.
pub(crate) fn init(data_dir: &Path) {
    let _ = LOG.set(data_dir.join(FILE_NAME));
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        log(&format!("fallo interno: {info}"));
        previous(info);
    }));
}

/// Anota un error con fecha (UTC). Nunca falla: si no se puede escribir, solo va a la consola.
pub(crate) fn log(msg: &str) {
    eprintln!("{msg}");
    let Some(path) = LOG.get() else { return };
    if fs::metadata(path).is_ok_and(|m| m.len() > MAX_BYTES)
        && let Ok(text) = fs::read_to_string(path)
    {
        // Se corta después de un salto de línea (ASCII): nunca a mitad de una «ñ».
        let half = text.len() / 2;
        let keep = text.as_bytes()[half..]
            .iter()
            .position(|&b| b == b'\n')
            .map_or("", |i| &text[half + i + 1..]);
        let _ = fs::write(path, keep);
    }
    let line = format!("{} {}\n", utc_now(), msg.replace('\n', " | "));
    if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = f.write_all(line.as_bytes());
    }
}

fn utc_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    civil(secs)
}

/// Segundos Unix → "AAAA-MM-DD HH:MM:SS" (algoritmo de días civiles de H. Hinnant).
fn civil(secs: u64) -> String {
    let (days, rest) = ((secs / 86_400) as i64, secs % 86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!(
        "{y:04}-{m:02}-{d:02} {:02}:{:02}:{:02}",
        rest / 3600,
        rest % 3600 / 60,
        rest % 60
    )
}

/// Errores de la interfaz (JavaScript) al mismo registro.
#[tauri::command]
pub fn log_frontend_error(message: String) {
    log(&format!(
        "interfaz: {}",
        message.chars().take(2000).collect::<String>()
    ));
}

#[derive(Debug, Serialize)]
pub struct ErrorReport {
    version: &'static str,
    os: &'static str,
    arch: &'static str,
    log_path: String,
    /// Últimas líneas del registro (vacío si no hubo errores).
    recent: Vec<String>,
}

/// Datos para «Reportar un problema». Solo versión, sistema y errores: ningún dato del CV.
#[tauri::command]
pub fn error_report() -> ErrorReport {
    let path = LOG.get().cloned().unwrap_or_default();
    let text = fs::read_to_string(&path).unwrap_or_default();
    let lines: Vec<&str> = text.lines().collect();
    ErrorReport {
        version: env!("CARGO_PKG_VERSION"),
        os: std::env::consts::OS,
        arch: std::env::consts::ARCH,
        log_path: path.display().to_string(),
        recent: lines[lines.len().saturating_sub(20)..]
            .iter()
            .map(|l| l.to_string())
            .collect(),
    }
}

/// Borra las claves del llavero, deja la marca y reinicia: al arrancar se vacía la carpeta de
/// datos (base, CVs generados, registro, respaldos pendientes) antes de abrir nada.
#[tauri::command]
pub fn delete_all_data(app: AppHandle) -> CmdResult<()> {
    crate::settings::delete_all_keys()?;
    let dir = app.path().app_data_dir().map_err(err)?;
    fs::write(dir.join(DELETE_MARK), b"").map_err(err)?;
    app.restart()
}

/// Al arrancar: si se pidió borrar todo, vacía `data_dir`. Devuelve si borró.
pub(crate) fn apply_delete(data_dir: &Path) -> std::io::Result<bool> {
    if !data_dir.join(DELETE_MARK).exists() {
        return Ok(false);
    }
    for entry in fs::read_dir(data_dir)? {
        let path = entry?.path();
        if path.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            fs::remove_file(&path)?;
        }
    }
    Ok(true)
}

/// Versión de la política de privacidad y los términos que aceptó el usuario (None = ninguna).
#[tauri::command]
pub async fn legal_accepted(state: tauri::State<'_, crate::AppState>) -> CmdResult<Option<String>> {
    state.db.setting("legal.accepted").await.map_err(err)
}

#[tauri::command]
pub async fn accept_legal(
    state: tauri::State<'_, crate::AppState>,
    version: String,
) -> CmdResult<()> {
    if version.len() > 20 {
        return Err("Versión no válida.".into());
    }
    state
        .db
        .set_setting("legal.accepted", Some(&version))
        .await
        .map_err(err)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fecha_civil() {
        assert_eq!(civil(0), "1970-01-01 00:00:00");
        assert_eq!(civil(951_782_400), "2000-02-29 00:00:00");
        assert_eq!(civil(1_794_124_308), "2026-11-08 07:51:48");
    }

    #[test]
    fn el_registro_rota_sin_cortar_caracteres() -> std::io::Result<()> {
        let dir = std::env::temp_dir().join(format!("finder-registro-{}", std::process::id()));
        fs::create_dir_all(&dir)?;
        let _ = LOG.set(dir.join(FILE_NAME));
        let path = LOG.get().ok_or(std::io::ErrorKind::NotFound)?;
        fs::write(path, "ñandú áéí\n".repeat(60_000))?;
        log("después de rotar");
        let text = fs::read_to_string(path)?;
        assert!(text.len() < 500_000 && text.ends_with("después de rotar\n"));
        fs::remove_dir_all(&dir)?;
        Ok(())
    }

    #[test]
    fn borrar_todo_vacia_la_carpeta() -> std::io::Result<()> {
        let dir = std::env::temp_dir().join(format!("finder-borrar-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("cvs"))?;
        fs::write(dir.join("finder.db"), b"x")?;
        fs::write(dir.join("cvs/cv.pdf"), b"x")?;
        assert!(!apply_delete(&dir)?, "sin marca no borra");
        assert!(dir.join("finder.db").exists());
        fs::write(dir.join(DELETE_MARK), b"")?;
        assert!(apply_delete(&dir)?);
        assert_eq!(fs::read_dir(&dir)?.count(), 0);
        fs::remove_dir_all(&dir)?;
        Ok(())
    }
}
