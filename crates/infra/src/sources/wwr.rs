//! We Work Remotely: feed RSS público de todas las categorías.
//! Se lee con `quick-xml` (no `feed-rs`) para conservar `<region>` ("USA Only",
//! "Anywhere in the World"), que alimenta la detección de bloqueantes.

use serde::Deserialize;

use super::{RawJob, Result, format_err, rfc2822_to_iso};

pub const URL: &str = "https://weworkremotely.com/remote-jobs.rss";

#[derive(Deserialize)]
struct Rss {
    channel: Channel,
}

#[derive(Deserialize)]
struct Channel {
    #[serde(rename = "item", default)]
    items: Vec<Item>,
}

#[derive(Deserialize)]
struct Item {
    title: String,
    link: String,
    #[serde(default)]
    guid: Option<String>,
    #[serde(default)]
    region: Option<String>,
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    description: String,
    #[serde(rename = "pubDate", default)]
    pub_date: Option<String>,
}

pub fn parse(body: &str) -> Result<Vec<RawJob>> {
    let rss: Rss = quick_xml::de::from_str(body).map_err(|e| format_err("wwr", e))?;
    Ok(rss
        .channel
        .items
        .into_iter()
        .map(|i| {
            // El título viene como "Empresa: Puesto".
            let (company, title) = match i.title.split_once(": ") {
                Some((c, t)) => (Some(c.trim().to_string()), t.trim().to_string()),
                None => (None, i.title.trim().to_string()),
            };
            let category = i
                .category
                .map(|c| format!("<p>Categoría: {c}</p>"))
                .unwrap_or_default();
            RawJob {
                external_id: i.guid,
                url: i.link,
                company,
                title,
                location: i.region,
                remote_type: Some("remote".into()),
                description_html: format!("{category}{}", i.description),
                posted_at: i.pub_date.as_deref().and_then(rfc2822_to_iso),
                ..Default::default()
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lee_el_feed_real_con_region() -> Result<()> {
        let jobs = parse(include_str!("../../tests/fixtures/wwr.rss"))?;
        assert_eq!(jobs.len(), 3);
        let j = &jobs[0];
        assert_eq!(j.company.as_deref(), Some("Sanctuary Computer"));
        assert_eq!(j.title, "Head of Technology");
        assert_eq!(j.location.as_deref(), Some("Anywhere in the World"));
        assert_eq!(j.posted_at.as_deref(), Some("2026-10-16T14:21:11Z"));
        assert!(
            j.description_html.contains("<p>"),
            "la descripción llega como HTML ya decodificado"
        );
        assert_eq!(jobs[2].location.as_deref(), Some("USA Only"));
        Ok(())
    }
}
