//! Comandos del Optimizador: generar la propuesta para una vacante, revisarla y guardar el CV.
//!
//! La propuesta se guarda en memoria por vacante: la interfaz solo envía las decisiones del
//! usuario al guardar, y puede volver a la pantalla sin repetir las llamadas a la IA.

use finder_infra::core::document::{Choices, Template};
use finder_infra::core::guard::Checked;
use finder_infra::core::model::{CvMaster, Job};
use finder_infra::llm::LlmClient;
use finder_infra::optimizer::{self, Optimization, OptimizeInput, SaveTarget};
use serde::Serialize;
use tauri::State;

use crate::{AppState, CmdResult, err, jobs};

/// Lo que hace falta para guardar sin volver a pedir nada a la IA.
pub struct Session {
    profile_id: String,
    master: CvMaster,
    original_text: Option<String>,
    job: Job,
    today: String,
    opt: Optimization,
}

/// Propuesta lista para la pantalla de revisión.
#[derive(Debug, Serialize)]
pub struct OptimizerView {
    profile_id: String,
    opt: Optimization,
    /// Motivos de rechazo ya redactados (los `Issue` se serializan como datos, no como texto).
    summary_issues: Vec<String>,
    bullet_issues: Vec<Vec<String>>,
    /// "Puesto — Empresa" de cada experiencia, por id, para agrupar los logros.
    experiences: Vec<(String, String)>,
}

#[derive(Debug, Serialize)]
pub struct SavedCv {
    ats_before: Option<u8>,
    ats_after: Option<u8>,
    pdf_path: Option<String>,
    docx_path: Option<String>,
    missing: Vec<String>,
}

fn messages(c: &Checked) -> Vec<String> {
    c.issues.iter().map(ToString::to_string).collect()
}

fn view(s: &Session) -> OptimizerView {
    OptimizerView {
        profile_id: s.profile_id.clone(),
        summary_issues: s
            .opt
            .review
            .summary
            .as_ref()
            .map(messages)
            .unwrap_or_default(),
        bullet_issues: s.opt.review.bullets.iter().map(messages).collect(),
        experiences: s
            .master
            .experience
            .iter()
            .map(|e| (e.id.clone(), format!("{} — {}", e.title, e.company)))
            .collect(),
        opt: s.opt.clone(),
    }
}

fn make_input<'a>(
    master: &'a CvMaster,
    original_text: Option<&'a str>,
    job: &'a Job,
    today: &'a str,
) -> OptimizeInput<'a> {
    OptimizeInput {
        master,
        original_text,
        job,
        today,
    }
}

fn input(s: &Session) -> OptimizeInput<'_> {
    make_input(&s.master, s.original_text.as_deref(), &s.job, &s.today)
}

/// Propuesta ya generada para esta vacante y perfil, si la hay.
#[tauri::command]
pub fn get_optimization(
    state: State<'_, AppState>,
    job_id: String,
    profile_id: String,
) -> CmdResult<Option<OptimizerView>> {
    let sessions = state.sessions.lock().map_err(err)?;
    Ok(sessions
        .get(&job_id)
        .filter(|s| s.profile_id == profile_id)
        .map(view))
}

/// Requisitos (IA local, o diccionario si no responde) → ATS antes → borrador del redactor →
/// guardián → PDF de prueba → ATS después. Puede tardar minutos con modelos locales.
#[tauri::command]
pub async fn run_optimizer(
    state: State<'_, AppState>,
    job_id: String,
    profile_id: String,
) -> CmdResult<OptimizerView> {
    let job = state
        .db
        .get_job(&job_id)
        .await
        .map_err(err)?
        .ok_or("La vacante ya no existe.")?;
    let stored = state
        .db
        .get_cv_master(&profile_id)
        .await
        .map_err(err)?
        .ok_or("Este perfil no tiene CV maestro. Créalo o impórtalo en Perfiles.")?;
    if stored.cv.experience.is_empty() {
        return Err("Tu CV maestro no tiene experiencia: no hay nada que adaptar.".into());
    }
    let config = state.llm()?;
    let local = LlmClient::new(config.local.clone()).map_err(err)?;
    let writer = LlmClient::new(config.writer.clone()).map_err(err)?;

    let (master, original_text, mut job) = (stored.cv, stored.original_file_text, job);
    let today: String = state.db.now().await.map_err(err)?.chars().take(7).collect();
    let prepared = optimizer::prepare(
        Some(&local),
        &state.dict,
        &make_input(&master, original_text.as_deref(), &job, &today),
    )
    .await;
    // Si la IA local analizó la vacante, se guarda para no repetirlo (y mejora la compatibilidad).
    if job.requirements.is_none() && prepared.notes.is_empty() {
        state
            .db
            .set_job_requirements(&job.id, &prepared.requirements)
            .await
            .map_err(err)?;
        job.requirements = Some(prepared.requirements.clone());
    }
    let opt = optimizer::optimize(
        &writer,
        &state.dict,
        &make_input(&master, original_text.as_deref(), &job, &today),
        prepared,
    )
    .await
    .map_err(err)?;

    let s = Session {
        profile_id,
        master,
        original_text,
        job,
        today,
        opt,
    };
    let v = view(&s);
    state
        .sessions
        .lock()
        .map_err(err)?
        .insert(s.job.id.clone(), s);
    Ok(v)
}

/// Guarda el CV con tus decisiones: PDF y DOCX en `<datos de la app>/cvs/`, versión vinculada a la
/// vacante en el tablero y estado "CV adaptado".
#[tauri::command]
pub async fn save_optimized(
    state: State<'_, AppState>,
    job_id: String,
    choices: Choices,
    template: Template,
) -> CmdResult<SavedCv> {
    // Se saca de la sesión para no mantener el candado durante el guardado (hay `await`).
    let s =
        state.sessions.lock().map_err(err)?.remove(&job_id).ok_or(
            "La propuesta ya no está en memoria (¿reiniciaste la app?). Genérala de nuevo.",
        )?;
    if choices.bullets.len() != s.opt.review.bullets.len() {
        let msg = "Las decisiones no corresponden a esta propuesta. Recarga la pantalla.";
        state.sessions.lock().map_err(err)?.insert(job_id, s);
        return Err(msg.into());
    }
    let result = save(&state, &s, &choices, template).await;
    // Se conserva la sesión: el usuario puede cambiar decisiones y guardar otra versión.
    state.sessions.lock().map_err(err)?.insert(job_id, s);
    result
}

async fn save(
    state: &AppState,
    s: &Session,
    choices: &Choices,
    template: Template,
) -> CmdResult<SavedCv> {
    let profile = state
        .db
        .get_profile(&s.profile_id)
        .await
        .map_err(err)?
        .ok_or("El perfil ya no existe.")?;
    let score = jobs::score(state, &(profile, s.master.clone()), &s.job, &s.today);
    let application_id = state
        .db
        .upsert_found(&s.job.id, &s.profile_id, score.total)
        .await
        .map_err(err)?;
    let out_dir = state.data_dir.join("cvs");
    let v = optimizer::save(
        &state.db,
        &state.dict,
        &input(s),
        &s.opt,
        choices,
        template,
        &SaveTarget {
            profile_id: &s.profile_id,
            application_id: &application_id,
            out_dir: &out_dir,
        },
    )
    .await
    .map_err(err)?;
    Ok(SavedCv {
        ats_before: v.ats_before,
        ats_after: v.ats_after,
        pdf_path: v.pdf_path,
        docx_path: v.docx_path,
        missing: v.missing,
    })
}
