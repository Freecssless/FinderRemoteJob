//! Recordatorios (plan §5.3): comandos para la pantalla y una tarea que avisa de los vencidos
//! con notificaciones del sistema al arrancar la app y después cada hora.

use std::collections::HashSet;
use std::time::Duration;

use finder_infra::core::pipeline::ReminderDays;
use finder_infra::db::Reminder;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_notification::NotificationExt;

use crate::{AppState, CmdResult, err};

const CHECK_EVERY: Duration = Duration::from_secs(60 * 60);

/// Vencidos y pendientes del perfil, más antiguos primero.
#[tauri::command]
pub async fn list_due_reminders(
    state: State<'_, AppState>,
    profile_id: String,
) -> CmdResult<Vec<Reminder>> {
    state
        .db
        .list_due_reminders(Some(&profile_id))
        .await
        .map_err(err)
}

#[tauri::command]
pub async fn complete_reminder(state: State<'_, AppState>, id: i64) -> CmdResult<()> {
    state.db.complete_reminder(id).await.map_err(err)
}

#[tauri::command]
pub async fn get_reminder_days(state: State<'_, AppState>) -> CmdResult<ReminderDays> {
    state.db.reminder_days().await.map_err(err)
}

/// Solo afecta a las postulaciones que pasen a "Postulada" desde ahora.
#[tauri::command]
pub async fn set_reminder_days(state: State<'_, AppState>, days: ReminderDays) -> CmdResult<()> {
    let ok = |d: u32| (1..=365).contains(&d);
    if !ok(days.follow_up) || !ok(days.no_response) {
        return Err("Los plazos deben estar entre 1 y 365 días.".into());
    }
    if days.no_response <= days.follow_up {
        return Err("«Sin respuesta» debe llegar después del seguimiento.".into());
    }
    state.db.set_reminder_days(days).await.map_err(err)
}

/// Revisa al arrancar y cada hora. Cada recordatorio se notifica una vez por sesión de la app
/// (si sigue pendiente, vuelve a avisar la próxima vez que la abras).
pub fn spawn_checker(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut notified = HashSet::new();
        loop {
            check(&app, &mut notified).await;
            tokio::time::sleep(CHECK_EVERY).await;
        }
    });
}

async fn check(app: &AppHandle, notified: &mut HashSet<i64>) {
    let state = app.state::<AppState>();
    let due = match state.db.list_due_reminders(None).await {
        Ok(d) => d,
        Err(e) => {
            eprintln!("recordatorios: {e}");
            return;
        }
    };
    let new: Vec<Reminder> = due.into_iter().filter(|r| notified.insert(r.id)).collect();
    let (title, body) = match new.as_slice() {
        [] => return,
        [r] => (r.kind.label().to_string(), job(r)),
        many => (
            format!("Tienes {} recordatorios en el tablero", many.len()),
            many.iter().take(3).map(job).collect::<Vec<_>>().join("\n"),
        ),
    };
    if let Err(e) = app.notification().builder().title(title).body(body).show() {
        eprintln!("no se pudo mostrar la notificación: {e}");
    }
}

fn job(r: &Reminder) -> String {
    match &r.company {
        Some(c) => format!("{} — {c}", r.job_title),
        None => r.job_title.clone(),
    }
}
