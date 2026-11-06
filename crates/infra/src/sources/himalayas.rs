//! Himalayas: API JSON pública de empleos remotos, con búsqueda por texto (20 por página).
//! Condiciones (himalayas.app/terms#api): acreditar a Himalayas con un enlace visible y que
//! "postular" use su `applicationLink` (por eso es la URL que se guarda).

use reqwest::Url;
use serde::Deserialize;

use super::{RawJob, Result, epoch_to_iso, format_err, parse_each};

const BASE: &str = "https://himalayas.app/jobs/api/search";

pub fn url(keyword: &str) -> Result<Url> {
    Url::parse_with_params(BASE, &[("q", keyword)]).map_err(|e| format_err("himalayas", e))
}

#[derive(Deserialize)]
struct Response {
    #[serde(default)]
    jobs: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Item {
    guid: String,
    application_link: String,
    title: String,
    #[serde(default)]
    company_name: Option<String>,
    #[serde(default)]
    location_restrictions: Vec<String>,
    #[serde(default)]
    min_salary: Option<f64>,
    #[serde(default)]
    max_salary: Option<f64>,
    #[serde(default)]
    salary_period: Option<String>,
    #[serde(default)]
    currency: Option<String>,
    #[serde(default)]
    seniority: Vec<String>,
    #[serde(default)]
    description: String,
    #[serde(default)]
    pub_date: Option<i64>,
}

/// Las demás fuentes dan salario anual: el mensual se multiplica por 12; otros periodos se omiten.
fn annual(v: Option<f64>, period: Option<&str>) -> Option<i64> {
    let factor = match period {
        None | Some("annual") => 1.0,
        Some("monthly") => 12.0,
        _ => return None,
    };
    v.filter(|n| *n > 0.0).map(|n| (n * factor) as i64)
}

pub fn parse(body: &str) -> Result<Vec<RawJob>> {
    let r: Response = serde_json::from_str(body).map_err(|e| format_err("himalayas", e))?;
    Ok(parse_each::<Item>(r.jobs)
        .into_iter()
        .map(|i| {
            let period = i.salary_period.as_deref();
            let level = match i.seniority.is_empty() {
                true => String::new(),
                false => format!("<p>Nivel: {}</p>", i.seniority.join(", ")),
            };
            RawJob {
                external_id: Some(i.guid),
                url: i.application_link,
                company: i.company_name,
                title: i.title,
                location: Some(if i.location_restrictions.is_empty() {
                    "Worldwide".into()
                } else {
                    i.location_restrictions.join(", ")
                }),
                remote_type: Some("remote".into()),
                salary_min: annual(i.min_salary, period),
                salary_max: annual(i.max_salary, period),
                currency: i.currency,
                description_html: format!("{level}{}", i.description),
                posted_at: i.pub_date.map(epoch_to_iso),
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
        let jobs = parse(include_str!("../../tests/fixtures/himalayas.json"))?;
        assert_eq!(jobs.len(), 2);
        let j = &jobs[0];
        assert_eq!(j.company.as_deref(), Some("Soar With Us"));
        assert!(j.url.starts_with("https://himalayas.app/companies/"));
        assert_eq!(j.location.as_deref(), Some("South Africa"));
        // 2000 USD mensuales.
        assert_eq!(j.salary_min, Some(24_000));
        assert_eq!(j.posted_at.as_deref(), Some("2026-10-29T01:38:37Z"));
        assert_eq!(jobs[1].salary_min, None);
        Ok(())
    }

    #[test]
    fn salario_anual() {
        assert_eq!(annual(Some(1000.0), Some("monthly")), Some(12_000));
        assert_eq!(annual(Some(50.0), Some("hourly")), None);
        assert_eq!(annual(Some(0.0), None), None);
    }

    #[test]
    fn url_codifica_la_busqueda() -> Result<()> {
        assert!(
            url("editor de video")?
                .as_str()
                .ends_with("?q=editor+de+video")
        );
        Ok(())
    }
}
