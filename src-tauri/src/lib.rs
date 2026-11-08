//! Puente Tauri: comandos delgados sobre `finder-infra`. Aquí no hay lógica de dominio.

mod backup;
mod board;
mod envfile;
mod jobs;
mod optimizer;
mod queue;
mod registro;
mod reminders;
mod settings;
mod weights;

use finder_infra::core::ats::keywords::SkillDict;
use finder_infra::core::guard::numbers;
use finder_infra::core::model::{CvMaster, Profile, RemotePref};
use finder_infra::cvfile::{CvFileKind, extract_text};
use finder_infra::db::Db;
use finder_infra::llm::tasks::{self as ai_tasks, TaskError};
use finder_infra::llm::{LlmClient, LlmConfig};
use serde::Serialize;
use tauri::{Manager, State};

/// Los errores llegan a la interfaz como texto ya en español.
pub(crate) type CmdResult<T> = Result<T, String>;

pub(crate) fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

pub(crate) struct AppState {
    db: Db,
    /// Se recarga al guardar los Ajustes.
    llm: std::sync::RwLock<Result<LlmConfig, String>>,
    /// Variables del `.env` (para recargar la configuración).
    file_vars: std::collections::HashMap<String, String>,
    dict: SkillDict,
    http: finder_infra::sources::Client,
    /// Carpeta de datos de la app (base de datos y `cvs/`).
    data_dir: std::path::PathBuf,
    /// Propuestas del optimizador pendientes de guardar, por id de vacante.
    sessions: std::sync::Mutex<std::collections::HashMap<String, optimizer::Session>>,
    /// Pesos de la compatibilidad (los normales salvo que el usuario aplique los sugeridos).
    weights: std::sync::RwLock<finder_infra::core::matching::Weights>,
    info: AppInfo,
}

/// Datos de diagnóstico para el pie de la interfaz.
#[derive(Debug, Clone, Serialize)]
struct AppInfo {
    db_path: String,
    env_file: Option<String>,
    local_model: Option<String>,
    writer_model: Option<String>,
    llm_error: Option<String>,
}

impl AppState {
    /// Configuración de IA vigente (o el motivo por el que no es válida).
    pub(crate) fn llm(&self) -> CmdResult<LlmConfig> {
        self.llm.read().map_err(err)?.clone()
    }
}

#[derive(Debug, Serialize)]
struct ImportPreview {
    cv: CvMaster,
    warnings: Vec<String>,
    source_text: String,
}

#[tauri::command]
fn app_info(state: State<'_, AppState>) -> AppInfo {
    let llm = state.llm();
    AppInfo {
        local_model: llm.as_ref().ok().map(|c| c.local.model.clone()),
        writer_model: llm.as_ref().ok().map(|c| settings::describe(&c.writer)),
        llm_error: llm.err(),
        ..state.info.clone()
    }
}

/// Estado de Ollama para la bienvenida: importar el CV siempre usa la IA local.
#[derive(Debug, Serialize)]
struct OllamaStatus {
    /// Ollama responde en la URL configurada.
    running: bool,
    /// Modelo local configurado (el que hay que descargar).
    model: String,
    /// El modelo ya está descargado.
    installed: bool,
}

#[tauri::command]
async fn ollama_status(state: State<'_, AppState>) -> CmdResult<OllamaStatus> {
    use finder_infra::llm::{Backend, ollama};
    let local = state.llm()?.local;
    let Backend::Ollama { base_url } = &local.backend else {
        // La IA local apunta a otro servidor: no hay nada de Ollama que comprobar.
        return Ok(OllamaStatus {
            running: true,
            model: local.model,
            installed: true,
        });
    };
    let models = ollama::installed_models(&state.http, base_url).await;
    Ok(OllamaStatus {
        running: models.is_ok(),
        installed: models.is_ok_and(|m| ollama::has_model(&m, &local.model)),
        model: local.model,
    })
}

/// Descarga en Ollama lo que falte: el modelo local y el de similitud semántica. El avance llega a
/// la interfaz como evento `ollama-pull` con (nombre, bajado, total).
#[tauri::command]
async fn pull_model(app: tauri::AppHandle, state: State<'_, AppState>) -> CmdResult<()> {
    use finder_infra::llm::{Backend, ollama};
    use tauri::Emitter;
    let config = state.llm()?;
    let Backend::Ollama { base_url } = &config.local.backend else {
        return Err("La IA local no es Ollama: descarga el modelo en tu servidor.".into());
    };
    let have = ollama::installed_models(&state.http, base_url)
        .await
        .map_err(err)?;
    for model in [&config.local.model, &config.embed_model] {
        if ollama::has_model(&have, model) {
            continue;
        }
        ollama::pull(base_url, model, |done, total| {
            let _ = app.emit("ollama-pull", (model, done, total));
        })
        .await
        .map_err(err)?;
    }
    Ok(())
}

/// Windows: baja el instalador oficial de Ollama y lo abre (el usuario termina la instalación).
#[tauri::command]
async fn install_ollama(app: tauri::AppHandle) -> CmdResult<()> {
    use tauri::Emitter;
    if !cfg!(windows) {
        return Err("La instalación automática solo está disponible en Windows.".into());
    }
    // Carpeta propia de la app, no la temporal compartida: otro programa no puede cambiar el
    // instalador entre la descarga y la ejecución.
    let dir = app.path().app_cache_dir().map_err(err)?;
    std::fs::create_dir_all(&dir).map_err(err)?;
    let dest = dir.join("OllamaSetup.exe");
    finder_infra::llm::ollama::download(
        "https://ollama.com/download/OllamaSetup.exe",
        &dest,
        |done, total| {
            let _ = app.emit("ollama-pull", ("Ollama", done, total));
        },
    )
    .await
    .map_err(err)?;
    std::process::Command::new(&dest)
        .spawn()
        .map_err(|e| format!("No se pudo abrir el instalador de Ollama: {e}"))?;
    Ok(())
}

/// Avisos del menú: vacantes nuevas (24 h), pendientes de postular y recordatorios vencidos.
#[derive(Debug, Serialize)]
struct NavCounts {
    new_jobs: i64,
    queue: usize,
    due: usize,
}

#[tauri::command]
async fn nav_counts(
    state: State<'_, AppState>,
    profile_id: Option<String>,
) -> CmdResult<NavCounts> {
    let new_jobs = state.db.count_jobs_found_within(24).await.map_err(err)?;
    let Some(pid) = profile_id else {
        return Ok(NavCounts {
            new_jobs,
            queue: 0,
            due: 0,
        });
    };
    let due = state
        .db
        .list_due_reminders(Some(&pid))
        .await
        .map_err(err)?
        .len();
    let queue = queue::application_queue(state, pid).await?.len();
    Ok(NavCounts {
        new_jobs,
        queue,
        due,
    })
}

#[tauri::command]
async fn list_profiles(state: State<'_, AppState>) -> CmdResult<Vec<Profile>> {
    state.db.list_profiles().await.map_err(err)
}

#[tauri::command]
async fn create_profile(
    state: State<'_, AppState>,
    name: String,
    target_titles: Vec<String>,
    remote_pref: RemotePref,
    locations: Vec<String>,
) -> CmdResult<Profile> {
    let name = name.trim();
    if name.is_empty() {
        return Err("El perfil necesita un nombre.".into());
    }
    state
        .db
        .create_profile(name, &target_titles, remote_pref, &locations)
        .await
        .map_err(err)
}

#[tauri::command]
async fn update_profile(state: State<'_, AppState>, profile: Profile) -> CmdResult<()> {
    if profile.name.trim().is_empty() {
        return Err("El perfil necesita un nombre.".into());
    }
    state.db.update_profile(&profile).await.map_err(err)
}

#[tauri::command]
async fn delete_profile(state: State<'_, AppState>, id: String) -> CmdResult<()> {
    // Los PDF/DOCX generados para el perfil también son su CV: se borran con él (solo los de `cvs/`).
    let versions = state.db.list_cv_versions(&id, None).await.map_err(err)?;
    state.db.delete_profile(&id).await.map_err(err)?;
    let cvs = state.data_dir.join("cvs");
    for path in versions
        .iter()
        .flat_map(|v| [&v.pdf_path, &v.docx_path])
        .flatten()
        .map(std::path::PathBuf::from)
        .filter(|p| p.parent() == Some(cvs.as_path()))
    {
        let _ = std::fs::remove_file(path);
    }
    queue::delete_faq(&state, &id).await
}

#[tauri::command]
async fn get_cv(state: State<'_, AppState>, profile_id: String) -> CmdResult<Option<CvMaster>> {
    Ok(state
        .db
        .get_cv_master(&profile_id)
        .await
        .map_err(err)?
        .map(|s| s.cv))
}

/// Guarda el CV revisado. `source_text` solo viene tras importar un archivo.
#[tauri::command]
async fn save_cv(
    state: State<'_, AppState>,
    profile_id: String,
    mut cv: CvMaster,
    source_text: Option<String>,
) -> CmdResult<()> {
    // El usuario pudo editar los logros: las cifras que usa el guardián se recalculan del texto.
    for b in cv.experience.iter_mut().flat_map(|e| &mut e.bullets) {
        b.metrics = numbers(&b.text);
    }
    state
        .db
        .save_cv_master(&profile_id, &cv, source_text.as_deref())
        .await
        .map_err(err)
}

/// Lee un PDF/DOCX y lo transcribe con la IA **local** (el CV completo no sale de tu equipo).
/// No guarda nada: el resultado va al formulario de revisión.
/// Un CV pesa unos cientos de KB; más de 20 MB no es un CV. El contenido debe ser del tipo que
/// dice la extensión (un `.pdf` que en realidad es otra cosa se rechaza antes de abrirlo).
const MAX_CV_BYTES: usize = 20 << 20;

fn check_upload(bytes: &[u8], kind: CvFileKind) -> CmdResult<()> {
    if bytes.len() > MAX_CV_BYTES {
        return Err("El archivo pesa más de 20 MB: no parece un CV.".into());
    }
    let magic: &[u8] = match kind {
        CvFileKind::Pdf => b"%PDF-",
        CvFileKind::Docx => b"PK\x03\x04",
    };
    // Algunos PDF traen basura antes de la cabecera; el estándar la admite en los primeros 1024 bytes.
    let head = &bytes[..bytes.len().min(1024)];
    if !head.windows(magic.len()).any(|w| w == magic) {
        return Err("El archivo no es un PDF o DOCX válido (¿cambiaste la extensión?).".into());
    }
    Ok(())
}

#[tauri::command]
async fn import_cv(
    state: State<'_, AppState>,
    file_name: String,
    bytes: Vec<u8>,
) -> CmdResult<ImportPreview> {
    let kind = CvFileKind::from_name(&file_name)
        .ok_or("Solo se pueden importar archivos .pdf o .docx.")?;
    check_upload(&bytes, kind)?;
    let config = state.llm()?;
    let client = LlmClient::new(config.local.clone()).map_err(err)?;
    let text = extract_text(&bytes, kind).map_err(err)?;
    // Si la IA local no responde, se importa lo que se puede leer sin ella (con un aviso).
    let imported = match ai_tasks::import_text(&client, &state.dict, text.clone()).await {
        Ok(i) => i,
        Err(TaskError::Llm(_)) => ai_tasks::import_without_ai(&state.dict, text),
        Err(e) => return Err(err(e)),
    };
    Ok(ImportPreview {
        cv: imported.result.cv,
        warnings: imported.result.warnings,
        source_text: imported.source_text,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            // Tu CV y tus postulaciones: solo tu usuario puede entrar a la carpeta (en Windows ya
            // lo garantiza AppData).
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&data_dir, std::fs::Permissions::from_mode(0o700))?;
            }
            if let Err(e) = registro::apply_delete(&data_dir) {
                return Err(format!("No se pudieron borrar tus datos: {e}").into());
            }
            registro::init(&data_dir);
            let db_path = data_dir.join("finder.db");
            // shortcut: si falla, se reintenta en el próximo arranque y solo queda en el registro.
            let origin = backup::apply_pending(&data_dir).unwrap_or_else(|e| {
                registro::log(&format!("no se pudo restaurar el respaldo: {e}"));
                None
            });
            let db = tauri::async_runtime::block_on(Db::open(&db_path))?;
            let here = data_dir.display().to_string();
            if let Some(old) = &origin
                && !old.is_empty()
                && *old != here
            {
                let sep = std::path::MAIN_SEPARATOR;
                tauri::async_runtime::block_on(
                    db.rebase_cv_paths(&format!("{old}{sep}"), &format!("{here}{sep}")),
                )?;
            }
            if origin.is_some() {
                // Un respaldo ajeno podría apuntar la IA a su servidor y recibir ahí tus claves
                // (del llavero) y tu CV: tras restaurar se vuelve a la IA local por defecto.
                tauri::async_runtime::block_on(settings::reset_after_restore(&db))?;
            }

            let env_file = envfile::find(&app.path().app_config_dir()?);
            let file_vars = env_file
                .as_deref()
                .and_then(|p| std::fs::read_to_string(p).ok())
                .map(|t| envfile::parse(&t))
                .unwrap_or_default();
            let llm = tauri::async_runtime::block_on(settings::load(&db, &file_vars));
            let weights = tauri::async_runtime::block_on(weights::load(&db));

            let info = AppInfo {
                db_path: db_path.display().to_string(),
                env_file: env_file.map(|p| p.display().to_string()),
                local_model: None,
                writer_model: None,
                llm_error: None,
            };
            app.manage(AppState {
                db,
                llm: std::sync::RwLock::new(llm),
                file_vars,
                dict: SkillDict::builtin()?,
                http: finder_infra::sources::http_client()?,
                data_dir,
                sessions: Default::default(),
                weights: std::sync::RwLock::new(weights),
                info,
            });
            reminders::spawn_checker(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_info,
            ollama_status,
            pull_model,
            install_ollama,
            nav_counts,
            weights::weights_status,
            weights::apply_weights,
            weights::reset_weights,
            backup::create_backup,
            backup::restore_backup,
            list_profiles,
            create_profile,
            update_profile,
            delete_profile,
            get_cv,
            save_cv,
            import_cv,
            jobs::list_jobs,
            jobs::get_job,
            jobs::analyze_job,
            jobs::sync_jobs,
            jobs::import_job_url,
            jobs::import_job_text,
            jobs::assisted_portals,
            optimizer::get_optimization,
            optimizer::run_optimizer,
            optimizer::save_optimized,
            optimizer::edit_proposal,
            optimizer::preview_cv,
            optimizer::list_saved_cvs,
            optimizer::update_saved_cv,
            board::list_board,
            board::set_application_status,
            board::save_application_notes,
            board::application_detail,
            board::add_to_board,
            board::get_stats,
            reminders::list_due_reminders,
            reminders::complete_reminder,
            reminders::get_reminder_days,
            reminders::set_reminder_days,
            settings::get_llm_settings,
            settings::save_llm_settings,
            settings::delete_api_key,
            jobs::embed_jobs,
            jobs::get_company_boards,
            jobs::save_company_boards,
            queue::application_queue,
            queue::get_faq,
            queue::save_faq,
            registro::log_frontend_error,
            registro::error_report,
            registro::delete_all_data,
            registro::legal_accepted,
            registro::accept_legal,
        ])
        .run(tauri::generate_context!())
        .expect("no se pudo iniciar la app de Tauri");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subida_de_cv_valida_tamano_y_tipo() {
        assert!(check_upload(b"%PDF-1.7 ...", CvFileKind::Pdf).is_ok());
        assert!(check_upload(b"\n\n%PDF-1.4", CvFileKind::Pdf).is_ok());
        assert!(check_upload(b"PK\x03\x04resto", CvFileKind::Docx).is_ok());
        assert!(check_upload(b"MZ\x90\x00 un .exe", CvFileKind::Pdf).is_err());
        assert!(check_upload(b"%PDF-1.7", CvFileKind::Docx).is_err());
        assert!(check_upload(b"", CvFileKind::Pdf).is_err());
        let mut big = b"%PDF-".to_vec();
        big.resize(MAX_CV_BYTES + 1, b' ');
        assert!(check_upload(&big, CvFileKind::Pdf).is_err());
    }
}
