//! Remotive: API JSON pública.
//! Condiciones: enlazar a la URL de Remotive y mencionarla como fuente; máximo 4 consultas al día
//! (ver `Source::min_interval_minutes`); las vacantes llegan con 24 h de retraso.

use serde::Deserialize;

use super::{RawJob, Result, format_err, parse_each};

pub const URL: &str = "https://remotive.com/api/remote-jobs";

#[derive(Deserialize)]
struct Response {
    jobs: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
struct Item {
    id: i64,
    url: String,
    title: String,
    #[serde(default)]
    company_name: Option<String>,
    #[serde(default)]
    candidate_required_location: Option<String>,
    #[serde(default)]
    salary: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    description: String,
    #[serde(default)]
    publication_date: Option<String>,
}

pub fn parse(body: &str) -> Result<Vec<RawJob>> {
    let r: Response = serde_json::from_str(body).map_err(|e| format_err("remotive", e))?;
    Ok(parse_each::<Item>(r.jobs)
        .into_iter()
        .map(|i| {
            // El salario llega como texto libre ("$20k - $35k"): se muestra, no se interpreta.
            let salary = i
                .salary
                .filter(|s| !s.trim().is_empty())
                .map(|s| format!("<p>Salario: {s}</p>"))
                .unwrap_or_default();
            let tags = if i.tags.is_empty() {
                String::new()
            } else {
                format!("<p>Etiquetas: {}</p>", i.tags.join(", "))
            };
            RawJob {
                external_id: Some(i.id.to_string()),
                url: i.url,
                company: i.company_name.map(|c| c.trim().to_string()),
                title: i.title,
                location: i.candidate_required_location,
                remote_type: Some("remote".into()),
                description_html: format!("{salary}{}{tags}", i.description),
                // Viene sin zona horaria; Remotive publica en UTC.
                posted_at: i.publication_date.map(|d| format!("{d}Z")),
                ..Default::default()
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lee_la_respuesta_real() -> Result<()> {
        let jobs = parse(include_str!("../../tests/fixtures/remotive.json"))?;
        assert_eq!(jobs.len(), 2);
        let j = &jobs[0];
        assert_eq!(j.title, "Freelance Copywriter");
        assert_eq!(
            j.company.as_deref(),
            Some("Coalition Technologies"),
            "sin espacios sobrantes"
        );
        assert_eq!(j.location.as_deref(), Some("Worldwide"));
        assert!(j.description_html.starts_with("<p>Salario: $20k -$35k</p>"));
        assert_eq!(j.posted_at.as_deref(), Some("2026-11-02T20:01:00Z"));
        Ok(())
    }
}
