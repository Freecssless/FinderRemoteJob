//! Importación manual de vacantes de cualquier portal (LinkedIn, OCC, Computrabajo…).
//!
//! - [`from_text`]: el usuario pega el texto. Siempre funciona.
//! - [`fetch_url`]: el usuario pega un enlace; se hace **una sola** petición (la que haría su
//!   navegador) y se lee el anuncio. Primero se busca el bloque JSON-LD `JobPosting` de schema.org,
//!   que muchos portales incluyen para Google; si no existe, se toma el texto principal de la página.
//!   Si el portal bloquea la petición o la página no trae el anuncio, se pide pegar el texto.

use reqwest::{Client, StatusCode};
use scraper::{Html, Selector};
use serde_json::Value;

use super::{RawJob, Result, SourceError, html_to_text};
use crate::db::{Db, JobInsert};

pub const SOURCE_ID: &str = "manual";

/// Menos texto que esto suele ser una página de inicio de sesión o un aviso, no un anuncio.
const MIN_TEXT_CHARS: usize = 80;

/// Vacante a partir del texto pegado por el usuario.
pub fn from_text(title: &str, company: Option<&str>, url: Option<&str>, text: &str) -> RawJob {
    // El texto pegado es texto plano: se escapa para que `html_to_text` no interprete "<" como etiqueta.
    let escaped = text
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    RawJob {
        url: url.unwrap_or_default().to_string(),
        company: company.map(String::from),
        title: title.to_string(),
        description_html: escaped,
        ..Default::default()
    }
}

/// Descarga la página del enlace y extrae la vacante.
pub async fn fetch_url(client: &Client, url: &str) -> Result<RawJob> {
    let resp = client.get(url).send().await?;
    let status = resp.status();
    if matches!(
        status,
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN | StatusCode::TOO_MANY_REQUESTS
    ) || status.as_u16() == 999
    // LinkedIn responde 999 a clientes que no son navegador.
    {
        return Err(SourceError::Blocked { status });
    }
    if !status.is_success() {
        return Err(SourceError::Status {
            status,
            url: url.to_string(),
        });
    }
    // Si hubo redirección, se guarda la URL final.
    let final_url = resp.url().to_string();
    parse_page(&final_url, &super::read_limited(resp).await?)
}

/// Guarda la vacante importada (sin duplicar si ya existía).
pub async fn save(db: &Db, raw: RawJob) -> Result<JobInsert> {
    let now = db.now().await?;
    Ok(db.insert_job(&raw.into_job(SOURCE_ID, &now)).await?)
}

/// Extrae la vacante del HTML de una página. Función pura (se prueba sin red).
pub fn parse_page(url: &str, html: &str) -> Result<RawJob> {
    let doc = Html::parse_document(html);
    if let Some(job) = json_ld_job(&doc, url) {
        return Ok(job);
    }

    // Sin JSON-LD: título de og:title, <h1> o <title>; texto de <main>, <article> o <body>.
    let title = first_attr(&doc, "meta[property='og:title']", "content")
        .or_else(|| first_text(&doc, "h1"))
        .or_else(|| first_text(&doc, "title"))
        .ok_or(SourceError::NoContent)?;
    let body_html = ["main", "article", "body"]
        .iter()
        .find_map(|sel| select_first(&doc, sel).map(|e| e.inner_html()))
        .ok_or(SourceError::NoContent)?;
    if html_to_text(&body_html).chars().count() < MIN_TEXT_CHARS {
        return Err(SourceError::NoContent);
    }
    Ok(RawJob {
        url: url.to_string(),
        title,
        description_html: body_html,
        ..Default::default()
    })
}

fn select_first<'a>(doc: &'a Html, selector: &str) -> Option<scraper::ElementRef<'a>> {
    Selector::parse(selector)
        .ok()
        .and_then(|s| doc.select(&s).next())
}

fn first_text(doc: &Html, selector: &str) -> Option<String> {
    let text = select_first(doc, selector)?.text().collect::<String>();
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    (!text.is_empty()).then_some(text)
}

fn first_attr(doc: &Html, selector: &str, attr: &str) -> Option<String> {
    let v = select_first(doc, selector)?
        .value()
        .attr(attr)?
        .trim()
        .to_string();
    (!v.is_empty()).then_some(v)
}

/// Busca un objeto `JobPosting` en los bloques JSON-LD (suelto, en lista o dentro de `@graph`).
fn json_ld_job(doc: &Html, url: &str) -> Option<RawJob> {
    let sel = Selector::parse("script[type='application/ld+json']").ok()?;
    doc.select(&sel)
        .filter_map(|s| serde_json::from_str::<Value>(&s.text().collect::<String>()).ok())
        .find_map(|v| find_job_posting(&v).and_then(|p| job_from_ld(p, url)))
}

fn find_job_posting(v: &Value) -> Option<&Value> {
    match v {
        Value::Array(items) => items.iter().find_map(find_job_posting),
        Value::Object(o) => {
            let is_job = match o.get("@type") {
                Some(Value::String(t)) => t == "JobPosting",
                Some(Value::Array(ts)) => ts.iter().any(|t| t == "JobPosting"),
                _ => false,
            };
            if is_job {
                Some(v)
            } else {
                o.get("@graph").and_then(find_job_posting)
            }
        }
        _ => None,
    }
}

fn job_from_ld(p: &Value, url: &str) -> Option<RawJob> {
    let s = |ptr: &str| {
        p.pointer(ptr)
            .and_then(Value::as_str)
            .map(|t| t.trim().to_string())
    };
    let n = |ptr: &str| p.pointer(ptr).and_then(Value::as_f64).map(|f| f as i64);

    let title = s("/title")?;
    let description_html = s("/description")?;
    let remote = s("/jobLocationType").is_some_and(|t| t.eq_ignore_ascii_case("TELECOMMUTE"));
    // Ubicación: requisito de residencia del candidato o dirección de la oficina.
    let location = s("/applicantLocationRequirements/name").or_else(|| {
        let parts: Vec<String> = [
            "/jobLocation/address/addressLocality",
            "/jobLocation/address/addressCountry",
        ]
        .iter()
        .filter_map(|ptr| s(ptr))
        .collect();
        (!parts.is_empty()).then(|| parts.join(", "))
    });
    Some(RawJob {
        external_id: s("/identifier/value"),
        url: url.to_string(),
        company: s("/hiringOrganization/name"),
        title,
        location,
        remote_type: remote.then(|| "remote".into()),
        salary_min: n("/baseSalary/value/minValue").or_else(|| n("/baseSalary/value/value")),
        salary_max: n("/baseSalary/value/maxValue"),
        currency: s("/baseSalary/currency"),
        description_html,
        language: None,
        posted_at: s("/datePosted"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lee_json_ld_job_posting() -> Result<()> {
        let j = parse_page(
            "https://portal.mx/v/1",
            include_str!("../../tests/fixtures/manual_jobposting.html"),
        )?;
        assert_eq!(j.title, "Editor de Video Jr");
        assert_eq!(j.company.as_deref(), Some("Empresa Ejemplo S.A. de C.V."));
        assert_eq!(j.location.as_deref(), Some("México"));
        assert_eq!(j.remote_type.as_deref(), Some("remote"));
        assert_eq!((j.salary_min, j.salary_max), (Some(15_000), Some(20_000)));
        assert_eq!(j.currency.as_deref(), Some("MXN"));
        let text = html_to_text(&j.description_html);
        assert!(text.contains("- Adobe Premiere Pro"));
        assert!(!text.contains("console.log"));
        Ok(())
    }

    #[test]
    fn sin_json_ld_usa_el_texto_principal() -> Result<()> {
        let j = parse_page(
            "https://estudio.mx/empleo",
            include_str!("../../tests/fixtures/manual_plain.html"),
        )?;
        assert_eq!(j.title, "Motion Designer & Editor");
        let text = html_to_text(&j.description_html);
        assert!(text.contains("- Cinema 4D"));
        assert!(!text.contains("Menú del sitio") && !text.contains("tracking"));
        Ok(())
    }

    #[test]
    fn pagina_sin_anuncio_pide_pegar_el_texto() {
        let login = "<html><head><title>Inicia sesión</title></head><body><main>Inicia sesión para continuar</main></body></html>";
        assert!(matches!(
            parse_page("https://x", login),
            Err(SourceError::NoContent)
        ));
    }

    #[test]
    fn json_ld_dentro_de_graph() -> Result<()> {
        let html = r#"<script type="application/ld+json">{"@graph":[{"@type":"WebPage"},
            {"@type":["JobPosting"],"title":"QA","description":"<p>Pruebas</p>"}]}</script>"#;
        assert_eq!(parse_page("https://x", html)?.title, "QA");
        Ok(())
    }

    #[test]
    fn texto_pegado_no_se_interpreta_como_html() {
        let raw = from_text(
            "Editor",
            Some("Agencia"),
            None,
            "Requisitos:\n- Premiere <avanzado>\n- After Effects",
        );
        let job = raw.into_job(SOURCE_ID, "2026-11-05T00:00:00Z");
        assert_eq!(
            job.description,
            "Requisitos:\n- Premiere <avanzado>\n- After Effects"
        );
    }

    #[tokio::test]
    async fn importar_y_guardar_sin_duplicar() -> std::result::Result<(), Box<dyn std::error::Error>>
    {
        let db = Db::open_in_memory().await?;
        let a = save(&db, from_text("Editor", Some("Agencia"), None, "texto")).await?;
        let b = save(
            &db,
            from_text("Editor", Some("agencia"), Some("https://otra"), "otro"),
        )
        .await?;
        assert!(a.created && !b.created);
        assert_eq!(a.id, b.id);
        Ok(())
    }
}
