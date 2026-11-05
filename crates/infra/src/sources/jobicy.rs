//! Jobicy: API JSON pública, filtrada a LATAM.
//! Condiciones: acreditar a Jobicy con enlace directo y que "postular" lleve a la URL original.

use serde::Deserialize;

use super::{RawJob, Result, format_err, parse_each};

pub const URL: &str = "https://jobicy.com/api/v2/remote-jobs?count=100&geo=latam";

#[derive(Deserialize)]
struct Response {
    #[serde(default)]
    jobs: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Item {
    id: i64,
    url: String,
    job_title: String,
    #[serde(default)]
    company_name: Option<String>,
    #[serde(default)]
    job_geo: Option<String>,
    #[serde(default)]
    job_level: Option<String>,
    #[serde(default)]
    job_description: String,
    #[serde(default)]
    pub_date: Option<String>,
    // Opcionales y de tipo variable según el anuncio: se leen como valor genérico.
    #[serde(default)]
    annual_salary_min: Option<serde_json::Value>,
    #[serde(default)]
    annual_salary_max: Option<serde_json::Value>,
    #[serde(default)]
    salary_currency: Option<String>,
}

fn as_i64(v: Option<serde_json::Value>) -> Option<i64> {
    match v? {
        serde_json::Value::Number(n) => n.as_f64().map(|f| f as i64),
        serde_json::Value::String(s) => s.trim().parse::<f64>().ok().map(|f| f as i64),
        _ => None,
    }
    .filter(|&n| n > 0)
}

pub fn parse(body: &str) -> Result<Vec<RawJob>> {
    let r: Response = serde_json::from_str(body).map_err(|e| format_err("jobicy", e))?;
    Ok(parse_each::<Item>(r.jobs)
        .into_iter()
        .map(|i| {
            let level = i
                .job_level
                .map(|l| format!("<p>Nivel: {l}</p>"))
                .unwrap_or_default();
            RawJob {
                external_id: Some(i.id.to_string()),
                url: i.url,
                company: i.company_name,
                title: i.job_title,
                location: i.job_geo,
                remote_type: Some("remote".into()),
                salary_min: as_i64(i.annual_salary_min),
                salary_max: as_i64(i.annual_salary_max),
                currency: i.salary_currency,
                description_html: format!("{level}{}", i.job_description),
                posted_at: i.pub_date,
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
        let jobs = parse(include_str!("../../tests/fixtures/jobicy.json"))?;
        assert_eq!(jobs.len(), 2);
        let j = &jobs[0];
        assert_eq!(j.company.as_deref(), Some("Welo Global"));
        assert!(j.url.starts_with("https://jobicy.com/jobs/"));
        assert!(
            j.location
                .as_deref()
                .is_some_and(|l| l.contains("Argentina"))
        );
        assert!(j.description_html.starts_with("<p>Nivel: Senior</p>"));
        Ok(())
    }

    #[test]
    fn salario_acepta_numero_o_texto() {
        assert_eq!(as_i64(Some(serde_json::json!(50000))), Some(50_000));
        assert_eq!(as_i64(Some(serde_json::json!("60000.5"))), Some(60_000));
        assert_eq!(as_i64(Some(serde_json::json!(0))), None);
        assert_eq!(as_i64(None), None);
    }
}
