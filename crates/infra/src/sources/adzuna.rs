//! Adzuna: API de búsqueda con clave gratuita (developer.adzuna.com). Uso permitido: búsqueda
//! personal de empleo. Cada anuncio se muestra con la fuente "Adzuna" y enlaza a su `redirect_url`.
//!
//! Nota: el ejemplo de `tests/fixtures/adzuna.json` está armado con el formato de su documentación
//! (sin clave no se pudo grabar una respuesta real).

use reqwest::Url;
use serde::Deserialize;

use super::{RawJob, Result, format_err, parse_each};

/// Países que atiende Adzuna (código de su URL).
pub const COUNTRIES: [&str; 19] = [
    "mx", "us", "ca", "br", "es", "gb", "de", "fr", "it", "nl", "be", "at", "ch", "pl", "au", "nz",
    "sg", "in", "za",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Credentials {
    pub app_id: String,
    pub app_key: String,
    pub country: String,
}

pub fn url(c: &Credentials, keyword: &str) -> Result<Url> {
    if !COUNTRIES.contains(&c.country.as_str()) {
        return Err(format_err(
            "adzuna",
            format!("país no admitido: {}", c.country),
        ));
    }
    Url::parse_with_params(
        &format!("https://api.adzuna.com/v1/api/jobs/{}/search/1", c.country),
        &[
            ("app_id", c.app_id.as_str()),
            ("app_key", c.app_key.as_str()),
            ("what", keyword),
            ("results_per_page", "50"),
            ("content-type", "application/json"),
        ],
    )
    .map_err(|e| format_err("adzuna", e))
}

fn currency(country: &str) -> Option<&'static str> {
    Some(match country {
        "mx" => "MXN",
        "us" => "USD",
        "ca" => "CAD",
        "br" => "BRL",
        "gb" => "GBP",
        "es" | "de" | "fr" | "it" | "nl" | "be" | "at" => "EUR",
        "ch" => "CHF",
        "pl" => "PLN",
        "au" => "AUD",
        "nz" => "NZD",
        "sg" => "SGD",
        "in" => "INR",
        "za" => "ZAR",
        _ => return None,
    })
}

#[derive(Deserialize)]
struct Response {
    #[serde(default)]
    results: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
struct Item {
    id: serde_json::Value,
    title: String,
    redirect_url: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    created: Option<String>,
    #[serde(default)]
    company: Option<Named>,
    #[serde(default)]
    location: Option<Named>,
    #[serde(default)]
    salary_min: Option<f64>,
    #[serde(default)]
    salary_max: Option<f64>,
    /// 0/1, a veces como texto.
    #[serde(default)]
    salary_is_predicted: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct Named {
    display_name: String,
}

pub fn parse(body: &str, country: &str) -> Result<Vec<RawJob>> {
    let r: Response = serde_json::from_str(body).map_err(|e| format_err("adzuna", e))?;
    Ok(parse_each::<Item>(r.results)
        .into_iter()
        .map(|i| {
            // Un salario "predicho" lo inventa Adzuna, no el anuncio: no se guarda.
            let predicted = matches!(
                i.salary_is_predicted.as_ref().map(|v| v.to_string().trim_matches('"').to_string()),
                Some(s) if s == "1"
            );
            let pay = |v: Option<f64>| v.filter(|n| *n > 0.0 && !predicted).map(|n| n as i64);
            RawJob {
                external_id: Some(i.id.to_string().trim_matches('"').to_string()),
                url: i.redirect_url,
                company: i.company.map(|c| c.display_name),
                title: i.title,
                location: i.location.map(|l| l.display_name),
                salary_min: pay(i.salary_min),
                salary_max: pay(i.salary_max),
                currency: (!predicted && i.salary_min.is_some())
                    .then(|| currency(country))
                    .flatten()
                    .map(String::from),
                description_html: i.description,
                posted_at: i.created,
                ..Default::default()
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lee_la_respuesta() -> Result<()> {
        let jobs = parse(include_str!("../../tests/fixtures/adzuna.json"), "mx")?;
        assert_eq!(jobs.len(), 2);
        let j = &jobs[0];
        assert_eq!(j.external_id.as_deref(), Some("4912345678"));
        assert_eq!(j.company.as_deref(), Some("Productora Ejemplo"));
        assert_eq!(
            (j.salary_min, j.currency.as_deref()),
            (Some(180_000), Some("MXN"))
        );
        // Salario predicho por Adzuna: fuera.
        assert_eq!(
            (jobs[1].salary_min, jobs[1].currency.as_deref()),
            (None, None)
        );
        Ok(())
    }

    #[test]
    fn url_con_pais_valido() -> Result<()> {
        let c = Credentials {
            app_id: "id".into(),
            app_key: "k".into(),
            country: "mx".into(),
        };
        let u = url(&c, "editor de video")?;
        assert!(
            u.as_str()
                .starts_with("https://api.adzuna.com/v1/api/jobs/mx/search/1?")
        );
        assert!(u.as_str().contains("what=editor+de+video"));
        let mut bad = c.clone();
        bad.country = "../x".into();
        assert!(url(&bad, "a").is_err());
        Ok(())
    }
}
