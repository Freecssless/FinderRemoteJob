//! Páginas de empleo de empresas concretas en Greenhouse, Lever y Ashby (plan §3). Sus APIs de
//! publicaciones son públicas y sin clave; el usuario elige qué empresas seguir pegando la URL de
//! su página de empleos (o `greenhouse:slug`, `lever:slug`, `ashby:slug`).

use reqwest::Url;
use scraper::Html;
use serde::Deserialize;

use super::{RawJob, Result, epoch_to_iso, format_err, parse_each};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ats {
    Greenhouse,
    Lever,
    Ashby,
}

impl Ats {
    pub fn id(self) -> &'static str {
        match self {
            Ats::Greenhouse => "greenhouse",
            Ats::Lever => "lever",
            Ats::Ashby => "ashby",
        }
    }
}

/// Una empresa seguida: su sistema y su identificador ("gitlab" en boards.greenhouse.io/gitlab).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    pub ats: Ats,
    pub slug: String,
}

impl Board {
    /// Acepta la URL de la página de empleos o `sistema:slug`. `None` si no se reconoce.
    pub fn parse(line: &str) -> Option<Board> {
        let line = line.trim();
        let (ats, slug) = match line.split_once(':') {
            Some((p, s)) if !s.starts_with("//") => (by_name(p)?, s.trim()),
            _ => {
                let with_scheme = if line.contains("://") {
                    line.to_string()
                } else {
                    format!("https://{line}")
                };
                let url = Url::parse(&with_scheme).ok()?;
                let ats = match url.host_str()? {
                    "boards.greenhouse.io" | "job-boards.greenhouse.io" => Ats::Greenhouse,
                    "jobs.lever.co" => Ats::Lever,
                    "jobs.ashbyhq.com" => Ats::Ashby,
                    _ => return None,
                };
                let slug = url.path_segments()?.find(|s| !s.is_empty())?.to_string();
                return valid(&slug).then_some(Board { ats, slug });
            }
        };
        valid(slug).then(|| Board {
            ats,
            slug: slug.to_string(),
        })
    }

    pub fn api_url(&self) -> String {
        match self.ats {
            Ats::Greenhouse => format!(
                "https://boards-api.greenhouse.io/v1/boards/{}/jobs?content=true",
                self.slug
            ),
            Ats::Lever => format!("https://api.lever.co/v0/postings/{}?mode=json", self.slug),
            Ats::Ashby => format!(
                "https://api.ashbyhq.com/posting-api/job-board/{}?includeCompensation=true",
                self.slug
            ),
        }
    }

    /// Lever y Ashby no dan el nombre de la empresa: se deduce del slug ("acme-labs" → "Acme Labs").
    fn company(&self) -> String {
        self.slug
            .split(['-', '_'])
            .filter(|w| !w.is_empty())
            .map(|w| {
                let mut c = w.chars();
                c.next()
                    .map(|f| f.to_uppercase().chain(c).collect::<String>())
                    .unwrap_or_default()
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn parse_jobs(&self, body: &str) -> Result<Vec<RawJob>> {
        match self.ats {
            Ats::Greenhouse => greenhouse(body),
            Ats::Lever => lever(body, &self.company()),
            Ats::Ashby => ashby(body, &self.company()),
        }
    }
}

fn by_name(name: &str) -> Option<Ats> {
    [Ats::Greenhouse, Ats::Lever, Ats::Ashby]
        .into_iter()
        .find(|a| a.id().eq_ignore_ascii_case(name.trim()))
}

/// El slug va dentro de la URL de la API: solo letras, números, `-`, `_` y `.`.
fn valid(slug: &str) -> bool {
    !slug.is_empty()
        && slug.len() <= 100
        && slug
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        && slug != "."
        && slug != ".."
}

// ---------------------------------------------------------------------------
// Greenhouse
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct GhResponse {
    #[serde(default)]
    jobs: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
struct GhJob {
    id: i64,
    title: String,
    absolute_url: String,
    #[serde(default)]
    company_name: Option<String>,
    #[serde(default)]
    location: Option<GhLocation>,
    /// HTML con las entidades escapadas (`&lt;p&gt;`).
    #[serde(default)]
    content: String,
    #[serde(default)]
    first_published: Option<String>,
    #[serde(default)]
    language: Option<String>,
}

#[derive(Deserialize)]
struct GhLocation {
    name: String,
}

fn greenhouse(body: &str) -> Result<Vec<RawJob>> {
    let r: GhResponse = serde_json::from_str(body).map_err(|e| format_err("greenhouse", e))?;
    Ok(parse_each::<GhJob>(r.jobs)
        .into_iter()
        .map(|j| RawJob {
            external_id: Some(j.id.to_string()),
            url: j.absolute_url,
            company: j.company_name,
            title: j.title,
            location: j.location.map(|l| l.name),
            // Quitar el escape da el HTML real.
            description_html: Html::parse_fragment(&j.content)
                .root_element()
                .text()
                .collect(),
            language: j.language,
            posted_at: j.first_published,
            ..Default::default()
        })
        .collect())
}

// ---------------------------------------------------------------------------
// Lever
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LeverJob {
    id: String,
    text: String,
    hosted_url: String,
    #[serde(default)]
    categories: LeverCategories,
    #[serde(default)]
    workplace_type: Option<String>,
    #[serde(default)]
    description: String,
    #[serde(default)]
    lists: Vec<LeverList>,
    #[serde(default)]
    additional: String,
    /// Milisegundos desde 1970.
    #[serde(default)]
    created_at: Option<i64>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct LeverCategories {
    #[serde(default)]
    location: Option<String>,
    #[serde(default)]
    all_locations: Vec<String>,
}

#[derive(Deserialize)]
struct LeverList {
    text: String,
    content: String,
}

fn lever(body: &str, company: &str) -> Result<Vec<RawJob>> {
    let items: Vec<serde_json::Value> =
        serde_json::from_str(body).map_err(|e| format_err("lever", e))?;
    Ok(parse_each::<LeverJob>(items)
        .into_iter()
        .map(|j| {
            let lists: String = j
                .lists
                .iter()
                .map(|l| format!("<h3>{}</h3><ul>{}</ul>", l.text, l.content))
                .collect();
            let location = match j.categories.all_locations.is_empty() {
                true => j.categories.location,
                false => Some(j.categories.all_locations.join(", ")),
            };
            RawJob {
                external_id: Some(j.id),
                url: j.hosted_url,
                company: Some(company.to_string()),
                title: j.text,
                location,
                remote_type: j.workplace_type.filter(|w| w != "unspecified"),
                description_html: format!("{}{lists}{}", j.description, j.additional),
                posted_at: j.created_at.map(|ms| epoch_to_iso(ms / 1000)),
                ..Default::default()
            }
        })
        .collect())
}

// ---------------------------------------------------------------------------
// Ashby
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct AshbyResponse {
    #[serde(default)]
    jobs: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AshbyJob {
    id: String,
    title: String,
    job_url: String,
    #[serde(default)]
    location: Option<String>,
    #[serde(default)]
    secondary_locations: Vec<AshbyLocation>,
    #[serde(default)]
    is_remote: bool,
    #[serde(default)]
    workplace_type: Option<String>,
    #[serde(default)]
    description_html: String,
    #[serde(default)]
    published_at: Option<String>,
    #[serde(default = "listed")]
    is_listed: bool,
    #[serde(default)]
    compensation: Option<AshbyCompensation>,
}

fn listed() -> bool {
    true
}

#[derive(Deserialize)]
struct AshbyLocation {
    location: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AshbyCompensation {
    #[serde(default)]
    compensation_tier_summary: Option<String>,
}

fn ashby(body: &str, company: &str) -> Result<Vec<RawJob>> {
    let r: AshbyResponse = serde_json::from_str(body).map_err(|e| format_err("ashby", e))?;
    Ok(parse_each::<AshbyJob>(r.jobs)
        .into_iter()
        .filter(|j| j.is_listed)
        .map(|j| {
            let location = j
                .location
                .into_iter()
                .chain(j.secondary_locations.into_iter().map(|l| l.location))
                .collect::<Vec<_>>()
                .join(", ");
            let pay = j
                .compensation
                .and_then(|c| c.compensation_tier_summary)
                .map(|s| format!("<p>Compensación: {s}</p>"))
                .unwrap_or_default();
            RawJob {
                external_id: Some(j.id),
                url: j.job_url,
                company: Some(company.to_string()),
                title: j.title,
                location: Some(location),
                remote_type: if j.is_remote {
                    Some("remote".into())
                } else {
                    j.workplace_type.map(|w| w.to_lowercase())
                },
                description_html: format!("{pay}{}", j.description_html),
                posted_at: j.published_at,
                ..Default::default()
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(ats: Ats, slug: &str) -> Option<Board> {
        Some(Board {
            ats,
            slug: slug.into(),
        })
    }

    #[test]
    fn reconoce_urls_y_atajos() {
        assert_eq!(
            Board::parse("https://boards.greenhouse.io/gitlab"),
            b(Ats::Greenhouse, "gitlab")
        );
        assert_eq!(
            Board::parse("job-boards.greenhouse.io/gitlab/jobs/123"),
            b(Ats::Greenhouse, "gitlab")
        );
        assert_eq!(
            Board::parse("https://jobs.lever.co/spotify/"),
            b(Ats::Lever, "spotify")
        );
        assert_eq!(
            Board::parse("https://jobs.ashbyhq.com/ramp?utm=x"),
            b(Ats::Ashby, "ramp")
        );
        assert_eq!(Board::parse(" Lever : netflix "), b(Ats::Lever, "netflix"));
        assert_eq!(Board::parse("https://www.linkedin.com/company/x"), None);
        assert_eq!(Board::parse("greenhouse:a/b"), None);
        assert_eq!(Board::parse("workday:acme"), None);
    }

    #[test]
    fn empresa_desde_el_slug() {
        assert_eq!(
            Board::parse("lever:acme-labs").map(|b| b.company()),
            Some("Acme Labs".into())
        );
    }

    #[test]
    fn lee_greenhouse() -> Result<()> {
        let jobs = greenhouse(include_str!("../../tests/fixtures/greenhouse.json"))?;
        assert_eq!(jobs.len(), 2);
        let j = &jobs[0];
        assert_eq!(j.company.as_deref(), Some("GitLab"));
        assert_eq!(j.location.as_deref(), Some("Remote, United States"));
        assert!(
            j.description_html.starts_with("<div"),
            "{}",
            j.description_html
        );
        Ok(())
    }

    #[test]
    fn lee_lever() -> Result<()> {
        let jobs = lever(include_str!("../../tests/fixtures/lever.json"), "Spotify")?;
        assert_eq!(jobs.len(), 2);
        assert_eq!(jobs[0].remote_type.as_deref(), Some("remote"));
        assert_eq!(jobs[0].location.as_deref(), Some("London, Stockholm"));
        assert_eq!(jobs[0].company.as_deref(), Some("Spotify"));
        assert!(jobs[0].url.starts_with("https://jobs.lever.co/spotify/"));
        assert!(
            jobs[0]
                .posted_at
                .as_deref()
                .is_some_and(|d| d.starts_with("2026-"))
        );
        Ok(())
    }

    #[test]
    fn lee_ashby() -> Result<()> {
        let jobs = ashby(include_str!("../../tests/fixtures/ashby.json"), "Ramp")?;
        assert_eq!(jobs.len(), 2);
        let j = &jobs[0];
        assert_eq!(j.remote_type.as_deref(), Some("remote"));
        assert!(
            j.location
                .as_deref()
                .is_some_and(|l| l.contains("Remote (Canada)"))
        );
        assert!(j.description_html.starts_with("<p>Compensación: $211.4K"));
        Ok(())
    }
}
