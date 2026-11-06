//! Comandos de la pantalla Vacantes: sincronizar fuentes, importar a mano, listar con
//! compatibilidad y analizar requisitos con la IA local.

use finder_infra::core::ats::keywords::{SkillDict, extract_terms};
use finder_infra::core::matching::{MatchInput, MatchReport, match_score};
use finder_infra::core::model::{CvMaster, Job, JobRequirements, Profile};
use finder_infra::llm::LlmClient;
use finder_infra::llm::tasks::extract_requirements;
use finder_infra::sources::{self, ASSISTED_PORTALS, Query, Source, manual};
use serde::Serialize;
use tauri::State;

use crate::{AppState, CmdResult, err};

/// Máximo de vacantes que se puntúan por consulta (el puntaje es local y rápido).
const LIST_LIMIT: u32 = 500;

/// Fila de la lista: sin la descripción completa, para no mover megas por cada recarga.
#[derive(Debug, Serialize)]
pub struct JobRow {
    id: String,
    title: String,
    company: Option<String>,
    location: Option<String>,
    source_id: String,
    url: String,
    posted_at: Option<String>,
    found_at: String,
    /// `true` si los requisitos ya se analizaron con IA (si no, se usó solo el diccionario).
    analyzed: bool,
    /// `None` si no hay perfil con CV maestro con el cual comparar.
    score: Option<MatchReport>,
}

#[derive(Debug, Serialize)]
pub struct JobDetail {
    job: Job,
    score: Option<MatchReport>,
}

#[derive(Debug, Serialize)]
pub struct SyncSummary {
    source: &'static str,
    skipped_too_soon: bool,
    fetched: usize,
    kept: usize,
    new: usize,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PortalLink {
    name: &'static str,
    url: String,
}

#[derive(Debug, Serialize)]
pub struct ImportedJob {
    id: String,
    created: bool,
}

/// Requisitos guardados o, si aún no se analizaron con IA, los del diccionario.
fn requirements(dict: &SkillDict, job: &Job) -> JobRequirements {
    job.requirements.clone().unwrap_or_else(|| JobRequirements {
        terms: extract_terms(dict, &job.title, &job.description),
        ..Default::default()
    })
}

/// Perfil y CV maestro con los que se compara; `None` si falta cualquiera de los dos.
pub(crate) async fn profile_and_cv(
    state: &AppState,
    profile_id: Option<&str>,
) -> CmdResult<Option<(Profile, CvMaster)>> {
    let Some(id) = profile_id else {
        return Ok(None);
    };
    let Some(profile) = state.db.get_profile(id).await.map_err(err)? else {
        return Ok(None);
    };
    Ok(state
        .db
        .get_cv_master(id)
        .await
        .map_err(err)?
        .map(|s| (profile, s.cv)))
}

pub(crate) fn score(
    state: &AppState,
    ctx: &(Profile, CvMaster),
    job: &Job,
    today: &str,
) -> MatchReport {
    let req = requirements(&state.dict, job);
    match_score(
        &state.dict,
        &MatchInput {
            profile: &ctx.0,
            master: &ctx.1,
            job,
            requirements: &req,
            semantic: None,
            today,
        },
    )
}

/// "AAAA-MM" de hoy, del reloj de SQLite (igual que el resto de fechas de la app).
pub(crate) async fn this_month(state: &AppState) -> CmdResult<String> {
    Ok(state.db.now().await.map_err(err)?.chars().take(7).collect())
}

/// Vacantes ordenadas por compatibilidad con el perfil (o por fecha si no hay perfil con CV).
/// `query` filtra con la búsqueda de texto completo.
#[tauri::command]
pub async fn list_jobs(
    state: State<'_, AppState>,
    profile_id: Option<String>,
    query: String,
) -> CmdResult<Vec<JobRow>> {
    let jobs = state
        .db
        .search_jobs(&query, LIST_LIMIT)
        .await
        .map_err(err)?;
    let ctx = profile_and_cv(&state, profile_id.as_deref()).await?;
    let today = this_month(&state).await?;
    let mut rows: Vec<JobRow> = jobs
        .into_iter()
        .map(|job| JobRow {
            score: ctx.as_ref().map(|c| score(&state, c, &job, &today)),
            analyzed: job.requirements.is_some(),
            id: job.id,
            title: job.title,
            company: job.company,
            location: job.location,
            source_id: job.source_id,
            url: job.url,
            posted_at: job.posted_at,
            found_at: job.found_at,
        })
        .collect();
    // Estable: a igual puntaje se conserva el orden de la búsqueda (relevancia o fecha).
    rows.sort_by_key(|r| std::cmp::Reverse(r.score.as_ref().map_or(0, |s| s.total)));
    Ok(rows)
}

#[tauri::command]
pub async fn get_job(
    state: State<'_, AppState>,
    job_id: String,
    profile_id: Option<String>,
) -> CmdResult<JobDetail> {
    let job = state
        .db
        .get_job(&job_id)
        .await
        .map_err(err)?
        .ok_or("La vacante ya no existe.")?;
    let ctx = profile_and_cv(&state, profile_id.as_deref()).await?;
    let today = this_month(&state).await?;
    Ok(JobDetail {
        score: ctx.as_ref().map(|c| score(&state, c, &job, &today)),
        job,
    })
}

/// Analiza los requisitos con la IA local y los guarda (se hace una vez por vacante).
#[tauri::command]
pub async fn analyze_job(
    state: State<'_, AppState>,
    job_id: String,
    profile_id: Option<String>,
) -> CmdResult<JobDetail> {
    let job = state
        .db
        .get_job(&job_id)
        .await
        .map_err(err)?
        .ok_or("La vacante ya no existe.")?;
    let config = state.llm.as_ref().map_err(Clone::clone)?;
    let client = LlmClient::new(config.local.clone()).map_err(err)?;
    let req = extract_requirements(&client, &state.dict, &job.title, &job.description)
        .await
        .map_err(err)?;
    state
        .db
        .set_job_requirements(&job_id, &req)
        .await
        .map_err(err)?;
    get_job(state, job_id, profile_id).await
}

/// Consulta las fuentes automáticas con los puestos buscados del perfil. Cada fuente respeta su
/// intervalo mínimo; si una falla, las demás siguen.
#[tauri::command]
pub async fn sync_jobs(
    state: State<'_, AppState>,
    profile_id: String,
) -> CmdResult<Vec<SyncSummary>> {
    let profile = state
        .db
        .get_profile(&profile_id)
        .await
        .map_err(err)?
        .ok_or("El perfil ya no existe.")?;
    let q = Query {
        keywords: profile.target_titles,
    };
    let mut out = Vec::new();
    for source in Source::ALL {
        out.push(
            match sources::sync(&state.db, &state.http, source, &q).await {
                Ok(r) => SyncSummary {
                    source: r.source,
                    skipped_too_soon: r.skipped_too_soon,
                    fetched: r.fetched,
                    kept: r.kept,
                    new: r.new_jobs.len(),
                    error: None,
                },
                Err(e) => SyncSummary {
                    source: source.id(),
                    skipped_too_soon: false,
                    fetched: 0,
                    kept: 0,
                    new: 0,
                    error: Some(e.to_string()),
                },
            },
        );
    }
    Ok(out)
}

#[tauri::command]
pub async fn import_job_url(state: State<'_, AppState>, url: String) -> CmdResult<ImportedJob> {
    let url = url.trim();
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err("Pega un enlace que empiece con http:// o https://.".into());
    }
    let raw = manual::fetch_url(&state.http, url).await.map_err(err)?;
    let r = manual::save(&state.db, raw).await.map_err(err)?;
    Ok(ImportedJob {
        id: r.id,
        created: r.created,
    })
}

#[tauri::command]
pub async fn import_job_text(
    state: State<'_, AppState>,
    title: String,
    company: Option<String>,
    url: Option<String>,
    text: String,
) -> CmdResult<ImportedJob> {
    if title.trim().is_empty() || text.trim().is_empty() {
        return Err("Escribe al menos el puesto y el texto del anuncio.".into());
    }
    let blank_to_none = |s: Option<String>| s.filter(|s| !s.trim().is_empty());
    let raw = manual::from_text(
        title.trim(),
        blank_to_none(company).as_deref(),
        blank_to_none(url).as_deref(),
        &text,
    );
    let r = manual::save(&state.db, raw).await.map_err(err)?;
    Ok(ImportedJob {
        id: r.id,
        created: r.created,
    })
}

/// Enlaces de búsqueda en portales sin API (LinkedIn, OCC…): se abren en tu navegador.
#[tauri::command]
pub fn assisted_portals(keywords: String) -> Vec<PortalLink> {
    ASSISTED_PORTALS
        .iter()
        .map(|p| PortalLink {
            name: p.name,
            url: p.search_url(&keywords),
        })
        .collect()
}
