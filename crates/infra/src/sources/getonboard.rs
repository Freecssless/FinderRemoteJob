//! Get on Board: API JSON pública (empleo tecnológico y creativo en LATAM).
//! Su documentación (getonbrd.com/api-doc) se genera con JavaScript y no se pudieron leer sus
//! términos; se usa con volumen bajo (máx. 3 búsquedas por sincronización, 1 por hora).

use reqwest::Url;
use serde::Deserialize;

use super::{RawJob, Result, epoch_to_iso, format_err, parse_each};

const BASE: &str = "https://www.getonbrd.com/api/v0/search/jobs";

/// Búsqueda por texto, pidiendo los datos de la empresa en la misma respuesta.
pub fn url(keyword: &str) -> Result<Url> {
    Url::parse_with_params(
        BASE,
        &[
            ("query", keyword),
            ("per_page", "50"),
            ("expand", "[\"company\"]"),
        ],
    )
    .map_err(|e| format_err("getonboard", e))
}

#[derive(Deserialize)]
struct Response {
    data: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
struct Item {
    id: String,
    attributes: Attributes,
    links: Links,
}

#[derive(Deserialize)]
struct Links {
    public_url: String,
}

#[derive(Deserialize)]
struct Attributes {
    title: String,
    #[serde(default)]
    description_headline: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    functions_headline: Option<String>,
    #[serde(default)]
    functions: Option<String>,
    #[serde(default)]
    desirable_headline: Option<String>,
    #[serde(default)]
    desirable: Option<String>,
    #[serde(default)]
    benefits_headline: Option<String>,
    #[serde(default)]
    benefits: Option<String>,
    #[serde(default)]
    remote_modality: Option<String>,
    #[serde(default)]
    countries: Vec<String>,
    #[serde(default)]
    lang: Option<String>,
    #[serde(default)]
    min_salary: Option<i64>,
    #[serde(default)]
    max_salary: Option<i64>,
    #[serde(default)]
    published_at: Option<i64>,
    #[serde(default)]
    company: Option<serde_json::Value>,
}

pub fn parse(body: &str) -> Result<Vec<RawJob>> {
    let r: Response = serde_json::from_str(body).map_err(|e| format_err("getonboard", e))?;
    Ok(parse_each::<Item>(r.data)
        .into_iter()
        .map(|i| {
            let a = i.attributes;
            // Secciones con sus encabezados ("Requisitos", "Deseable"...) para que la extracción
            // de palabras clave distinga lo requerido de lo deseable.
            let section = |h: Option<String>, body: Option<String>| match body {
                Some(b) if !b.trim().is_empty() => format!("<h3>{}</h3>{b}", h.unwrap_or_default()),
                _ => String::new(),
            };
            let description_html = [
                section(a.functions_headline, a.functions),
                section(a.description_headline, a.description),
                section(a.desirable_headline, a.desirable),
                section(a.benefits_headline, a.benefits),
            ]
            .concat();
            let company = a
                .company
                .as_ref()
                .and_then(|c| c.pointer("/data/attributes/name"))
                .and_then(|n| n.as_str())
                .map(String::from);
            RawJob {
                external_id: Some(i.id),
                url: i.links.public_url,
                company,
                title: a.title,
                location: (!a.countries.is_empty()).then(|| a.countries.join(", ")),
                remote_type: a.remote_modality,
                salary_min: a.min_salary,
                salary_max: a.max_salary,
                // Get on Board publica rangos mensuales en USD.
                currency: (a.min_salary.is_some() || a.max_salary.is_some())
                    .then(|| "USD/mes".into()),
                description_html,
                language: a.lang,
                posted_at: a.published_at.map(epoch_to_iso),
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lee_la_respuesta_real_con_empresa_expandida() -> Result<()> {
        let jobs = parse(include_str!("../../tests/fixtures/getonboard.json"))?;
        assert_eq!(jobs.len(), 2);
        let j = &jobs[0];
        assert_eq!(j.company.as_deref(), Some("Cute Digital Media"));
        assert_eq!(j.remote_type.as_deref(), Some("fully_remote"));
        assert_eq!(j.location.as_deref(), Some("Remote"));
        assert!(j.url.starts_with("https://www.getonbrd.com/jobs/"));
        assert!(j.description_html.contains("<h3>"));
        Ok(())
    }

    #[test]
    fn url_codifica_la_busqueda() -> Result<()> {
        let u = url("editor de video")?;
        assert!(u.as_str().contains("query=editor+de+video"));
        assert!(u.as_str().contains("expand=%5B%22company%22%5D"));
        Ok(())
    }
}
