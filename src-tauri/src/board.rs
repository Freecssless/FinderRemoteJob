//! Comandos del Tablero: postulaciones por estado, cambios de estado, notas y ficha.

use finder_infra::core::model::{Application, CvVersion, Status, StatusChange};
use finder_infra::db::{BoardFilter, BoardRow, Reminder};
use serde::Serialize;
use tauri::State;

use crate::{AppState, CmdResult, err, jobs};

/// Lo que la lista del tablero no trae: historial y CV enviado.
#[derive(Debug, Serialize)]
pub struct ApplicationDetail {
    history: Vec<StatusChange>,
    cv: Option<CvVersion>,
    /// Recordatorios pendientes (vencidos o no).
    reminders: Vec<Reminder>,
}

/// Postulaciones del perfil (mayor compatibilidad primero). La interfaz las agrupa por estado.
#[tauri::command]
pub async fn list_board(
    state: State<'_, AppState>,
    profile_id: String,
) -> CmdResult<Vec<BoardRow>> {
    state
        .db
        .list_board(&BoardFilter {
            profile_id: Some(profile_id),
            ..Default::default()
        })
        .await
        .map_err(err)
}

/// Mueve la postulación de columna (fija "postulada el" y "primera respuesta" si toca).
#[tauri::command]
pub async fn set_application_status(
    state: State<'_, AppState>,
    application_id: String,
    status: Status,
) -> CmdResult<Application> {
    state
        .db
        .set_status(&application_id, status)
        .await
        .map_err(err)
}

/// Notas y próximo paso. Un campo vacío se guarda como vacío (`NULL`).
#[tauri::command]
pub async fn save_application_notes(
    state: State<'_, AppState>,
    application_id: String,
    notes: String,
    next_step: String,
    next_step_at: String,
) -> CmdResult<()> {
    let opt = |s: &str| Some(s.trim()).filter(|s| !s.is_empty()).map(String::from);
    state
        .db
        .update_application_notes(
            &application_id,
            opt(&notes).as_deref(),
            opt(&next_step).as_deref(),
            opt(&next_step_at).as_deref(),
        )
        .await
        .map_err(err)
}

#[tauri::command]
pub async fn application_detail(
    state: State<'_, AppState>,
    application_id: String,
) -> CmdResult<ApplicationDetail> {
    let app = state
        .db
        .get_application(&application_id)
        .await
        .map_err(err)?
        .ok_or("La postulación ya no existe.")?;
    let cv = match &app.cv_version_id {
        Some(id) => state.db.get_cv_version(id).await.map_err(err)?,
        None => None,
    };
    Ok(ApplicationDetail {
        history: state.db.status_history(&app.id).await.map_err(err)?,
        reminders: state.db.pending_reminders(&app.id).await.map_err(err)?,
        cv,
    })
}

/// Agrega la vacante al tablero del perfil como "Encontrada" (si ya estaba, solo actualiza el puntaje).
#[tauri::command]
pub async fn add_to_board(
    state: State<'_, AppState>,
    job_id: String,
    profile_id: String,
) -> CmdResult<String> {
    ensure_application(&state, &job_id, &profile_id).await
}

/// Id de la postulación de la vacante en el tablero del perfil (la crea si no existe).
pub(crate) async fn ensure_application(
    state: &AppState,
    job_id: &str,
    profile_id: &str,
) -> CmdResult<String> {
    let job = state
        .db
        .get_job(job_id)
        .await
        .map_err(err)?
        .ok_or("La vacante ya no existe.")?;
    let ctx = jobs::profile_and_cv(state, Some(profile_id)).await?.ok_or(
        "Este perfil no tiene CV maestro: créalo en Perfiles para calcular la compatibilidad.",
    )?;
    let today = jobs::this_month(state).await?;
    let vectors = jobs::vectors(state, Some(profile_id)).await?;
    let score = jobs::score(state, &ctx, &job, &today, &vectors);
    state
        .db
        .upsert_found(&job.id, profile_id, score.total)
        .await
        .map_err(err)
}

/// Estadísticas (plan §5.5) de un perfil, o de todos con `None`.
#[tauri::command]
pub async fn get_stats(
    state: State<'_, AppState>,
    profile_id: Option<String>,
) -> CmdResult<finder_infra::core::stats::Stats> {
    let rows = state
        .db
        .stat_inputs(profile_id.as_deref())
        .await
        .map_err(err)?;
    let today = state.db.now().await.map_err(err)?;
    Ok(finder_infra::core::stats::compute(&rows, &today))
}
