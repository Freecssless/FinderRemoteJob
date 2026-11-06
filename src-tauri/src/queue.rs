//! Cola de postulación asistida (plan §3): vacantes con compatibilidad alta que aún no tienen
//! postulación enviada, y respuestas frecuentes para pegar en los formularios de los portales.
//! La app no llena formularios ni entra a tus cuentas: abre el anuncio y tú envías.

use std::collections::HashMap;

use finder_infra::core::matching::years_of_experience;
use finder_infra::core::model::Status;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{AppState, CmdResult, err, jobs};

const MIN_SCORE: u8 = 70;
const QUEUE_LIMIT: usize = 20;
const JOBS_SCANNED: u32 = 500;

#[derive(Debug, Serialize)]
pub struct QueueItem {
    job_id: String,
    title: String,
    company: Option<String>,
    url: String,
    score: u8,
    /// `None` si aún no está en el tablero.
    application_id: Option<String>,
    status: Option<Status>,
    pdf_path: Option<String>,
    cover_letter: Option<String>,
    #[serde(skip)]
    cv_version_id: Option<String>,
}

/// Aún no enviada (lo descartado o ya postulado sale de la cola).
fn pending(s: Status) -> bool {
    matches!(
        s,
        Status::Encontrada | Status::Evaluada | Status::CvAdaptado
    )
}

/// Las 20 vacantes más compatibles (≥ 70) sin postulación enviada.
#[tauri::command]
pub async fn application_queue(
    state: State<'_, AppState>,
    profile_id: String,
) -> CmdResult<Vec<QueueItem>> {
    let Some(ctx) = jobs::profile_and_cv(&state, Some(&profile_id)).await? else {
        return Ok(vec![]);
    };
    let board: HashMap<String, _> = state
        .db
        .list_board(&finder_infra::db::BoardFilter {
            profile_id: Some(profile_id),
            ..Default::default()
        })
        .await
        .map_err(err)?
        .into_iter()
        .map(|r| (r.application.job_id.clone(), r.application))
        .collect();
    let today = jobs::this_month(&state).await?;
    let mut items: Vec<QueueItem> = Vec::new();
    for job in state.db.list_jobs(JOBS_SCANNED).await.map_err(err)? {
        let app = board.get(&job.id);
        if app.is_some_and(|a| !pending(a.status)) {
            continue;
        }
        let score = jobs::score(&state, &ctx, &job, &today).total;
        if score < MIN_SCORE {
            continue;
        }
        items.push(QueueItem {
            score,
            application_id: app.map(|a| a.id.clone()),
            status: app.map(|a| a.status),
            pdf_path: None,
            cv_version_id: app.and_then(|a| a.cv_version_id.clone()),
            cover_letter: app.and_then(|a| a.cover_letter.clone()),
            job_id: job.id,
            title: job.title,
            company: job.company,
            url: job.url,
        });
    }
    items.sort_by_key(|i| std::cmp::Reverse(i.score));
    items.truncate(QUEUE_LIMIT);
    for i in &mut items {
        if let Some(v) = &i.cv_version_id {
            i.pdf_path = state
                .db
                .get_cv_version(v)
                .await
                .map_err(err)?
                .and_then(|v| v.pdf_path);
        }
    }
    Ok(items)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Faq {
    question: String,
    answer: String,
}

#[derive(Debug, Serialize)]
pub struct FaqView {
    /// Calculadas de tu CV maestro (no se editan aquí).
    from_cv: Vec<Faq>,
    /// Las tuyas: salario, disponibilidad, etc.
    custom: Vec<Faq>,
}

fn faq_key(profile_id: &str) -> String {
    format!("faq.{profile_id}")
}

/// Preguntas sugeridas la primera vez (sin respuesta: las llenas tú).
fn starter() -> Vec<Faq> {
    [
        "Salario esperado",
        "Disponibilidad para empezar",
        "¿Requieres patrocinio de visa?",
        "Zona horaria y horario disponible",
    ]
    .map(|q| Faq {
        question: q.into(),
        answer: String::new(),
    })
    .into()
}

fn fmt_years(y: f32) -> String {
    let y = (y * 2.0).round() / 2.0;
    if y.fract() == 0.0 {
        format!("{y:.0}")
    } else {
        format!("{y:.1}")
    }
}

#[tauri::command]
pub async fn get_faq(state: State<'_, AppState>, profile_id: String) -> CmdResult<FaqView> {
    let mut from_cv = Vec::new();
    if let Some((_, cv)) = jobs::profile_and_cv(&state, Some(&profile_id)).await? {
        let today = jobs::this_month(&state).await?;
        from_cv.push(Faq {
            question: "Años de experiencia".into(),
            answer: fmt_years(years_of_experience(&cv, &today)),
        });
        for s in &cv.skills {
            if let Some(y) = s.years {
                from_cv.push(Faq {
                    question: format!("Años con {}", s.name),
                    answer: fmt_years(y),
                });
            }
        }
        if !cv.languages.is_empty() {
            from_cv.push(Faq {
                question: "Idiomas".into(),
                answer: cv
                    .languages
                    .iter()
                    .map(|l| format!("{} ({})", l.name, l.level))
                    .collect::<Vec<_>>()
                    .join(", "),
            });
        }
    }
    let stored = state.db.setting(&faq_key(&profile_id)).await.map_err(err)?;
    let custom = match stored {
        Some(json) => serde_json::from_str(&json).map_err(err)?,
        None => starter(),
    };
    Ok(FaqView { from_cv, custom })
}

/// Guarda tus respuestas (las preguntas vacías se quitan).
#[tauri::command]
pub async fn save_faq(
    state: State<'_, AppState>,
    profile_id: String,
    items: Vec<Faq>,
) -> CmdResult<()> {
    let items: Vec<Faq> = items
        .into_iter()
        .map(|f| Faq {
            question: f.question.trim().into(),
            answer: f.answer.trim().into(),
        })
        .filter(|f| !f.question.is_empty())
        .collect();
    let json = serde_json::to_string(&items).map_err(err)?;
    state
        .db
        .set_setting(&faq_key(&profile_id), Some(&json))
        .await
        .map_err(err)
}

/// Al borrar un perfil.
pub(crate) async fn delete_faq(state: &AppState, profile_id: &str) -> CmdResult<()> {
    state
        .db
        .set_setting(&faq_key(profile_id), None)
        .await
        .map_err(err)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solo_lo_no_enviado_entra_a_la_cola() {
        assert!(pending(Status::CvAdaptado));
        assert!(!pending(Status::Postulada));
        assert!(!pending(Status::Descartada));
    }

    #[test]
    fn anos_redondeados_a_medio() {
        assert_eq!(fmt_years(6.0), "6");
        assert_eq!(fmt_years(2.4), "2.5");
        assert_eq!(fmt_years(2.2), "2");
    }
}
