//! Carta de presentación (plan §4.6): el redactor la propone, el guardián revisa cada párrafo y tú
//! la editas y la guardas en la postulación.

use finder_infra::letter;
use finder_infra::llm::LlmClient;
use serde::Serialize;
use tauri::State;

use crate::{AppState, CmdResult, board, err, jobs};

#[derive(Debug, Serialize)]
pub struct LetterParagraph {
    text: String,
    /// Motivos de rechazo; vacío = entró en la carta.
    issues: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct LetterView {
    paragraphs: Vec<LetterParagraph>,
    text: String,
}

/// Puede tardar con modelos locales. No guarda nada.
#[tauri::command]
pub async fn write_cover_letter(
    state: State<'_, AppState>,
    job_id: String,
    profile_id: String,
) -> CmdResult<LetterView> {
    let job = state
        .db
        .get_job(&job_id)
        .await
        .map_err(err)?
        .ok_or("La vacante ya no existe.")?;
    let (_, master) = jobs::profile_and_cv(&state, Some(&profile_id))
        .await?
        .ok_or("Este perfil no tiene CV maestro. Créalo o impórtalo en Perfiles.")?;
    if master.experience.is_empty() {
        return Err("Tu CV maestro no tiene experiencia: no hay logros que contar.".into());
    }
    let config = state.llm()?;
    let writer = LlmClient::new(config.writer.clone()).map_err(err)?;
    let today = jobs::this_month(&state).await?;
    let l = letter::write(&writer, &state.dict, &master, &job, &today)
        .await
        .map_err(err)?;
    Ok(LetterView {
        paragraphs: l
            .paragraphs
            .into_iter()
            .map(|p| LetterParagraph {
                issues: p.issues.iter().map(ToString::to_string).collect(),
                text: p.proposed,
            })
            .collect(),
        text: l.text,
    })
}

/// Guarda la carta (tal como la dejaste) en la postulación; la agrega al tablero si no estaba.
#[tauri::command]
pub async fn save_cover_letter(
    state: State<'_, AppState>,
    job_id: String,
    profile_id: String,
    text: String,
) -> CmdResult<()> {
    let id = board::ensure_application(&state, &job_id, &profile_id).await?;
    let text = Some(text.trim()).filter(|t| !t.is_empty());
    state.db.set_cover_letter(&id, text).await.map_err(err)
}
