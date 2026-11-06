//! Puente Tauri: comandos delgados sobre `finder-infra`. Aquí no hay lógica de dominio.

mod board;
mod envfile;
mod jobs;
mod optimizer;

use finder_infra::core::ats::keywords::SkillDict;
use finder_infra::core::guard::numbers;
use finder_infra::core::model::{CvMaster, Profile, RemotePref};
use finder_infra::cvfile::CvFileKind;
use finder_infra::db::Db;
use finder_infra::llm::tasks::import_cv as llm_import_cv;
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
    llm: Result<LlmConfig, String>,
    dict: SkillDict,
    http: finder_infra::sources::Client,
    /// Carpeta de datos de la app (base de datos y `cvs/`).
    data_dir: std::path::PathBuf,
    /// Propuestas del optimizador pendientes de guardar, por id de vacante.
    sessions: std::sync::Mutex<std::collections::HashMap<String, optimizer::Session>>,
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

#[derive(Debug, Serialize)]
struct ImportPreview {
    cv: CvMaster,
    warnings: Vec<String>,
    source_text: String,
}

#[tauri::command]
fn app_info(state: State<'_, AppState>) -> AppInfo {
    state.info.clone()
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
    state.db.delete_profile(&id).await.map_err(err)
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
#[tauri::command]
async fn import_cv(
    state: State<'_, AppState>,
    file_name: String,
    bytes: Vec<u8>,
) -> CmdResult<ImportPreview> {
    let kind = CvFileKind::from_name(&file_name)
        .ok_or("Solo se pueden importar archivos .pdf o .docx.")?;
    let config = state.llm.as_ref().map_err(Clone::clone)?;
    let client = LlmClient::new(config.local.clone()).map_err(err)?;
    let imported = llm_import_cv(&client, &bytes, kind).await.map_err(err)?;
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
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("finder.db");
            let db = tauri::async_runtime::block_on(Db::open(&db_path))?;

            let env_file = envfile::find(&app.path().app_config_dir()?);
            let file_vars = env_file
                .as_deref()
                .and_then(|p| std::fs::read_to_string(p).ok())
                .map(|t| envfile::parse(&t))
                .unwrap_or_default();
            let llm = LlmConfig::from_lookup(|k| {
                std::env::var(k).ok().or_else(|| file_vars.get(k).cloned())
            })
            .map_err(err);

            let info = AppInfo {
                db_path: db_path.display().to_string(),
                env_file: env_file.map(|p| p.display().to_string()),
                local_model: llm.as_ref().ok().map(|c| c.local.model.clone()),
                writer_model: llm.as_ref().ok().map(|c| c.writer.model.clone()),
                llm_error: llm.as_ref().err().cloned(),
            };
            app.manage(AppState {
                db,
                llm,
                dict: SkillDict::builtin()?,
                http: finder_infra::sources::http_client()?,
                data_dir,
                sessions: Default::default(),
                info,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_info,
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
            board::list_board,
            board::set_application_status,
            board::save_application_notes,
            board::application_detail,
            board::add_to_board,
        ])
        .run(tauri::generate_context!())
        .expect("no se pudo iniciar la app de Tauri");
}
