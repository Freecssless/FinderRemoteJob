//! Compatibilidad a tu medida (plan Fase 2, «reajuste de pesos con tus datos»): sugiere pesos a
//! partir de qué postulaciones avanzaron. Nunca se aplican solos: el usuario los acepta o vuelve
//! a los normales desde Ajustes.

use std::collections::HashMap;

use finder_infra::core::matching::{
    LEARN_MIN_EACH, LEARN_MIN_SAMPLES, MatchBreakdown, Weights, learn_weights, match_score,
};
use finder_infra::core::model::Status;
use finder_infra::db::BoardFilter;
use serde::Serialize;
use tauri::State;

use crate::{AppState, CmdResult, err, jobs};

const KEY: &str = "scoring.weights";
/// Una postulación sin respuesta cuenta como «no avanzó» pasados estos días (como las estadísticas).
const NO_ANSWER_DAYS: u32 = 14;

/// Pesos guardados al arrancar (o los normales).
pub(crate) async fn load(db: &finder_infra::db::Db) -> Weights {
    db.setting(KEY)
        .await
        .ok()
        .flatten()
        .and_then(|v| serde_json::from_str(&v).ok())
        .unwrap_or_default()
}

#[derive(Debug, Serialize)]
pub struct WeightsStatus {
    /// Postulaciones con resultado claro, y cuántas avanzaron / no avanzaron.
    samples: usize,
    advanced: usize,
    not_advanced: usize,
    needed: usize,
    min_each: usize,
    current: Weights,
    default: Weights,
    suggested: Option<Weights>,
}

#[tauri::command]
pub async fn weights_status(state: State<'_, AppState>) -> CmdResult<WeightsStatus> {
    let samples = samples(&state).await?;
    let advanced = samples.iter().filter(|(_, ok)| *ok).count();
    Ok(WeightsStatus {
        samples: samples.len(),
        advanced,
        not_advanced: samples.len() - advanced,
        needed: LEARN_MIN_SAMPLES,
        min_each: LEARN_MIN_EACH,
        current: *state.weights.read().map_err(err)?,
        default: Weights::default(),
        suggested: learn_weights(&samples),
    })
}

#[tauri::command]
pub async fn apply_weights(state: State<'_, AppState>, weights: Weights) -> CmdResult<()> {
    let sum = weights.skills
        + weights.semantic
        + weights.title
        + weights.seniority
        + weights.years
        + weights.logistics;
    if !(99.0..=101.0).contains(&sum)
        || [
            weights.skills,
            weights.semantic,
            weights.title,
            weights.seniority,
            weights.years,
            weights.logistics,
        ]
        .iter()
        .any(|w| *w < 0.0)
    {
        return Err("Los pesos deben ser positivos y sumar 100.".into());
    }
    let json = serde_json::to_string(&weights).map_err(err)?;
    state.db.set_setting(KEY, Some(&json)).await.map_err(err)?;
    *state.weights.write().map_err(err)? = weights;
    Ok(())
}

#[tauri::command]
pub async fn reset_weights(state: State<'_, AppState>) -> CmdResult<()> {
    state.db.set_setting(KEY, None).await.map_err(err)?;
    *state.weights.write().map_err(err)? = Weights::default();
    Ok(())
}

/// ¿Avanzó? `None` si aún no hay resultado claro (o nunca te postulaste).
fn outcome(
    status: Status,
    applied_at: Option<&str>,
    responded: bool,
    cutoff: &str,
) -> Option<bool> {
    let applied = applied_at?;
    match status {
        Status::EnProceso | Status::Entrevista | Status::Oferta => Some(true),
        Status::Rechazada | Status::SinRespuesta => Some(false),
        Status::Postulada if !responded && applied < cutoff => Some(false),
        _ => None,
    }
}

/// Desglose con los pesos normales de cada postulación con resultado claro, y si avanzó.
async fn samples(state: &AppState) -> CmdResult<Vec<(MatchBreakdown, bool)>> {
    let rows = state
        .db
        .list_board(&BoardFilter::default())
        .await
        .map_err(err)?;
    let cutoff = state.db.days_ago(NO_ANSWER_DAYS).await.map_err(err)?;
    let today = jobs::this_month(state).await?;
    let mut ctx = HashMap::new();
    let mut out = Vec::new();
    for row in rows {
        let a = &row.application;
        let Some(advanced) = outcome(
            a.status,
            a.applied_at.as_deref(),
            a.first_response_at.is_some(),
            &cutoff,
        ) else {
            continue;
        };
        if !ctx.contains_key(&a.profile_id) {
            let c = jobs::profile_and_cv(state, Some(&a.profile_id)).await?;
            let v = jobs::vectors(state, Some(&a.profile_id)).await?;
            ctx.insert(a.profile_id.clone(), (c, v));
        }
        let (Some(c), vectors) = &ctx[&a.profile_id] else {
            continue;
        };
        let Some(job) = state.db.get_job(&a.job_id).await.map_err(err)? else {
            continue;
        };
        let req = jobs::requirements(&state.dict, &job);
        let report = match_score(
            &state.dict,
            &finder_infra::core::matching::MatchInput {
                profile: &c.0,
                master: &c.1,
                job: &job,
                requirements: &req,
                semantic: vectors.similarity(&job.id),
                today: &today,
            },
        );
        out.push((report.breakdown, advanced));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clasifica_solo_resultados_claros() {
        let cut = "2026-10-24T00:00:00Z";
        let old = Some("2026-10-01T00:00:00Z");
        let recent = Some("2026-11-01T00:00:00Z");
        assert_eq!(outcome(Status::Entrevista, old, true, cut), Some(true));
        assert_eq!(outcome(Status::Oferta, recent, true, cut), Some(true));
        assert_eq!(outcome(Status::Rechazada, recent, true, cut), Some(false));
        // Postulada hace más de 14 días sin respuesta: no avanzó.
        assert_eq!(outcome(Status::Postulada, old, false, cut), Some(false));
        // Reciente, o con respuesta pero sin cambiar de estado: aún no se sabe.
        assert_eq!(outcome(Status::Postulada, recent, false, cut), None);
        assert_eq!(outcome(Status::Postulada, old, true, cut), None);
        // Sin postularte no cuenta.
        assert_eq!(outcome(Status::Encontrada, None, false, cut), None);
        assert_eq!(outcome(Status::Entrevista, None, true, cut), None);
    }
}
