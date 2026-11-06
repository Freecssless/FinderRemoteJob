//! Jooble: API REST con clave gratuita por país (jooble.org/api/about; la de jooble.org solo da
//! vacantes de EE. UU., la de mx.jooble.org las de México). **La clave gratuita tiene 500
//! consultas en total**, así que se consulta una vez al día con todos los puestos juntos.
//!
//! Nota: el ejemplo de `tests/fixtures/jooble.json` está armado con el formato de su documentación
//! (sin clave no se pudo grabar una respuesta real).

use serde::Deserialize;
use serde_json::{Value, json};

use super::{RawJob, Result, format_err, parse_each};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Credentials {
    pub key: String,
    /// "jooble.org", "mx.jooble.org"…
    pub domain: String,
}

/// Solo `jooble.org` o `xx.jooble.org`: el dominio forma parte de la URL a la que va la clave.
pub fn valid_domain(d: &str) -> bool {
    d == "jooble.org"
        || d.strip_suffix(".jooble.org").is_some_and(|sub| {
            (2..=3).contains(&sub.len()) && sub.chars().all(|c| c.is_ascii_lowercase())
        })
}

pub fn url(c: &Credentials) -> Result<String> {
    if !valid_domain(&c.domain)
        || !c
            .key
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
    {
        return Err(format_err("jooble", "dominio o clave con formato inválido"));
    }
    Ok(format!("https://{}/api/{}", c.domain, c.key))
}

pub fn body(keywords: &[String]) -> Value {
    json!({ "keywords": keywords.join(", "), "location": "", "page": 1, "ResultOnPage": 100 })
}

#[derive(Deserialize)]
struct Response {
    #[serde(default)]
    jobs: Vec<Value>,
}

#[derive(Deserialize)]
struct Item {
    id: Value,
    title: String,
    link: String,
    #[serde(default)]
    location: String,
    #[serde(default)]
    snippet: String,
    #[serde(default)]
    salary: String,
    #[serde(default)]
    company: String,
    #[serde(default, rename = "type")]
    kind: String,
    #[serde(default)]
    updated: Option<String>,
}

pub fn parse(body: &str) -> Result<Vec<RawJob>> {
    let r: Response = serde_json::from_str(body).map_err(|e| format_err("jooble", e))?;
    let some = |s: String| Some(s).filter(|s| !s.trim().is_empty());
    Ok(parse_each::<Item>(r.jobs)
        .into_iter()
        .map(|i| {
            // El salario llega como texto libre ("15,000 - 20,000 MXN"): va a la descripción.
            let extra: String = [("Salario", &i.salary), ("Jornada", &i.kind)]
                .iter()
                .filter(|(_, v)| !v.trim().is_empty())
                .map(|(k, v)| format!("<p>{k}: {v}</p>"))
                .collect();
            RawJob {
                external_id: Some(i.id.to_string()),
                url: i.link,
                company: some(i.company),
                title: i.title,
                location: some(i.location),
                description_html: format!("{extra}{}", i.snippet),
                posted_at: i
                    .updated
                    .map(|u| format!("{}Z", u.chars().take(19).collect::<String>())),
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
        let jobs = parse(include_str!("../../tests/fixtures/jooble.json"))?;
        assert_eq!(jobs.len(), 2);
        let j = &jobs[0];
        assert_eq!(j.external_id.as_deref(), Some("1234567890123"));
        assert_eq!(j.posted_at.as_deref(), Some("2026-11-03T12:55:35Z"));
        assert!(
            j.description_html
                .starts_with("<p>Salario: 15,000 - 20,000 MXN</p>")
        );
        assert_eq!(jobs[1].company, None);
        Ok(())
    }

    #[test]
    fn dominio_y_url() {
        assert!(valid_domain("jooble.org") && valid_domain("mx.jooble.org"));
        assert!(
            !valid_domain("evil.com")
                && !valid_domain("a.b.jooble.org")
                && !valid_domain("x.jooble.org.evil.com")
        );
        let c = Credentials {
            key: "abc-123".into(),
            domain: "mx.jooble.org".into(),
        };
        assert_eq!(
            url(&c).ok().as_deref(),
            Some("https://mx.jooble.org/api/abc-123")
        );
        assert!(
            url(&Credentials {
                key: "a/b".into(),
                ..c
            })
            .is_err()
        );
        assert_eq!(
            body(&["Editor de video".into(), "Video editor".into()])["keywords"],
            "Editor de video, Video editor"
        );
    }
}
