//! Similitud semántica (plan §4.3): embeddings del CV maestro y de cada vacante con la IA local,
//! guardados en la base para no recalcularlos. El puntaje de compatibilidad solo los lee.

use std::collections::HashMap;

use finder_core::matching::{cosine, cv_semantic_text, job_semantic_text};
use finder_core::model::{CvMaster, Job};

use crate::db::{Db, DbError};
use crate::llm::{LlmClient, LlmError};

#[derive(Debug, thiserror::Error)]
pub enum SemanticError {
    #[error(transparent)]
    Llm(#[from] LlmError),
    #[error(transparent)]
    Db(#[from] DbError),
}

/// Textos por petición: rápido en CPU y sin rozar el tiempo de espera.
const BATCH: usize = 16;

fn text_hash(s: &str) -> String {
    let h = s.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    });
    format!("{h:016x}")
}

pub fn cv_owner(profile_id: &str) -> String {
    format!("cv:{profile_id}")
}

pub fn job_owner(job_id: &str) -> String {
    format!("job:{job_id}")
}

/// Calcula y guarda los embeddings que faltan o cuyo texto cambió. Devuelve cuántos calculó.
pub async fn ensure(
    db: &Db,
    client: &LlmClient,
    model: &str,
    items: Vec<(String, String)>,
) -> Result<usize, SemanticError> {
    let mut stored: HashMap<String, (String, Vec<f32>)> = HashMap::new();
    for prefix in ["cv:", "job:"] {
        if items.iter().any(|(o, _)| o.starts_with(prefix)) {
            stored.extend(db.embeddings(model, prefix).await?);
        }
    }
    let pending: Vec<(String, String, String)> = items
        .into_iter()
        .map(|(owner, text)| {
            let hash = text_hash(&text);
            (owner, text, hash)
        })
        .filter(|(owner, _, hash)| stored.get(owner).is_none_or(|(h, _)| h != hash))
        .collect();
    for chunk in pending.chunks(BATCH) {
        let texts: Vec<String> = chunk.iter().map(|(_, t, _)| t.clone()).collect();
        let vectors = client.embed(model, &texts).await?;
        for ((owner, _, hash), v) in chunk.iter().zip(vectors) {
            db.put_embedding(owner, model, hash, &v).await?;
        }
    }
    Ok(pending.len())
}

/// Lo que va a [`ensure`] para un CV y unas vacantes.
pub fn items(profile_id: &str, master: &CvMaster, jobs: &[Job]) -> Vec<(String, String)> {
    std::iter::once((cv_owner(profile_id), cv_semantic_text(master)))
        .chain(
            jobs.iter()
                .map(|j| (job_owner(&j.id), job_semantic_text(j))),
        )
        .collect()
}

/// Embeddings guardados de un perfil: el de su CV y los de las vacantes.
#[derive(Debug, Default)]
pub struct Vectors {
    pub cv: Option<Vec<f32>>,
    pub jobs: HashMap<String, Vec<f32>>,
}

impl Vectors {
    pub async fn load(db: &Db, model: &str, profile_id: &str) -> Result<Self, DbError> {
        let cv = db
            .embeddings(model, &cv_owner(profile_id))
            .await?
            .remove(&cv_owner(profile_id))
            .map(|(_, v)| v);
        let jobs = db
            .embeddings(model, "job:")
            .await?
            .into_iter()
            .filter_map(|(o, (_, v))| Some((o.strip_prefix("job:")?.to_string(), v)))
            .collect();
        Ok(Vectors { cv, jobs })
    }

    /// Coseno CV ↔ vacante, si ambos están calculados.
    pub fn similarity(&self, job_id: &str) -> Option<f32> {
        cosine(self.cv.as_deref()?, self.jobs.get(job_id)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn similitud_solo_con_ambos_vectores() {
        let mut v = Vectors::default();
        v.jobs.insert("j1".into(), vec![1.0, 0.0]);
        assert_eq!(v.similarity("j1"), None);
        v.cv = Some(vec![1.0, 0.0]);
        assert_eq!(v.similarity("j1"), Some(1.0));
        assert_eq!(v.similarity("otra"), None);
    }

    /// Con el Ollama local y su modelo de embeddings (`ollama pull bge-m3`). Correr a mano:
    /// `cargo test -p finder-infra semantic -- --ignored --nocapture`
    #[tokio::test]
    #[ignore]
    async fn en_vivo_la_vacante_parecida_puntua_mas() -> Result<(), Box<dyn std::error::Error>> {
        let cfg = crate::llm::LlmConfig::from_env()?;
        let client = LlmClient::new(cfg.local.clone())?;
        let db = Db::open_in_memory().await?;
        let model = cfg.embed_model.as_str();
        let items = vec![
            (cv_owner("p1"), "Editora de video con 4 años. Premiere Pro, After Effects, motion graphics y corrección de color para cursos y redes sociales.".to_string()),
            (job_owner("video"), "Buscamos editor de video senior para cursos en línea: Premiere Pro, After Effects y animación.".to_string()),
            (job_owner("conta"), "Contador público para conciliaciones bancarias, declaraciones fiscales y cierre contable mensual.".to_string()),
        ];
        assert_eq!(ensure(&db, &client, model, items.clone()).await?, 3);
        // Los mismos textos ya están guardados: no se recalculan.
        assert_eq!(ensure(&db, &client, model, items).await?, 0);
        let v = Vectors::load(&db, model, "p1").await?;
        let (video, conta) = (
            v.similarity("video").ok_or("sin video")?,
            v.similarity("conta").ok_or("sin conta")?,
        );
        println!("{model}: video {video:.3} · contabilidad {conta:.3}");
        assert!(
            video > conta + 0.05,
            "la vacante de video debería parecerse más al CV"
        );
        Ok(())
    }

    #[test]
    fn el_hash_cambia_con_el_texto() {
        assert_ne!(text_hash("a"), text_hash("b"));
        assert_eq!(text_hash("a"), text_hash("a"));
    }
}
