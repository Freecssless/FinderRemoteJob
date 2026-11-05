//! Remote OK: API JSON pública.
//! Condiciones: enlazar a la URL de Remote OK y mencionar "Remote OK" como fuente
//! (se guarda `url` original y `source_id`). El primer elemento es el aviso legal, no una vacante.

use serde::Deserialize;

use super::{RawJob, Result, epoch_to_iso, format_err, parse_each};

pub const URL: &str = "https://remoteok.com/api";

#[derive(Deserialize)]
struct Item {
    id: serde_json::Value,
    position: String,
    url: String,
    #[serde(default)]
    company: Option<String>,
    #[serde(default)]
    location: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    description: String,
    #[serde(default)]
    salary_min: Option<i64>,
    #[serde(default)]
    salary_max: Option<i64>,
    #[serde(default)]
    epoch: Option<i64>,
}

pub fn parse(body: &str) -> Result<Vec<RawJob>> {
    let items: Vec<serde_json::Value> =
        serde_json::from_str(body).map_err(|e| format_err("remoteok", e))?;
    // El aviso legal no tiene `position`, así que `parse_each` lo descarta solo.
    Ok(parse_each::<Item>(items)
        .into_iter()
        .map(|i| {
            // Las etiquetas ayudan a encontrar habilidades que la descripción no nombra.
            let tags = if i.tags.is_empty() {
                String::new()
            } else {
                format!("<p>Etiquetas: {}</p>", i.tags.join(", "))
            };
            RawJob {
                external_id: Some(i.id.to_string().trim_matches('"').to_string()),
                url: i.url,
                company: i.company,
                title: i.position,
                location: i.location,
                remote_type: Some("remote".into()),
                // 0 significa "sin dato".
                salary_min: i.salary_min.filter(|&s| s > 0),
                salary_max: i.salary_max.filter(|&s| s > 0),
                currency: Some("USD".into()),
                description_html: format!("{}{tags}", i.description),
                language: None,
                posted_at: i.epoch.map(epoch_to_iso),
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lee_la_respuesta_real_y_omite_el_aviso_legal() -> Result<()> {
        let jobs = parse(include_str!("../../tests/fixtures/remoteok.json"))?;
        assert_eq!(jobs.len(), 2);
        let j = &jobs[0];
        assert_eq!(j.title, "AI Content Reviewer Video Annotation");
        assert_eq!(j.company.as_deref(), Some("iMerit"));
        assert_eq!(j.external_id.as_deref(), Some("1137686"));
        assert!(j.url.starts_with("https://remoteOK.com/remote-jobs/"));
        assert_eq!(j.salary_min, Some(80_000));
        assert!(j.description_html.contains("Etiquetas: content writing"));
        assert!(
            j.posted_at
                .as_deref()
                .is_some_and(|d| d.starts_with("2026-11-04"))
        );
        Ok(())
    }

    #[test]
    fn formato_inesperado_da_error_claro() {
        assert!(parse("{\"no\": \"es una lista\"}").is_err());
    }
}
