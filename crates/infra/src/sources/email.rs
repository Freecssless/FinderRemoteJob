//! Lector de alertas por correo (plan §3): LinkedIn, Indeed, OCC, Computrabajo y Bumeran te
//! mandan avisos de vacantes a tu correo; aquí se leen por IMAP y se sacan los enlaces a las
//! vacantes. La app nunca entra a esos portales.
//!
//! - El buzón se abre en **solo lectura** (`EXAMINE`) y los correos se bajan con `BODY.PEEK[]`:
//!   no se marcan como leídos ni se mueven.
//! - Solo se buscan correos de los remitentes de [`PORTALS`] de los últimos [`DAYS`] días.
//! - La contraseña (de aplicación, en Gmail/Outlook) vive en el llavero del sistema.
//!
//! shortcut: la lectura de cada alerta es heurística (enlaces que parecen de una vacante + el
//! texto alrededor) y se armó sin correos reales; si un portal cambia su formato o envuelve los
//! enlaces en un redirector, sus vacantes no aparecen. Mejorar con correos reales de ejemplo.

use std::collections::HashSet;
use std::sync::Arc;

use futures_util::StreamExt;
use mail_parser::MessageParser;
use reqwest::Url;
use rustls::ClientConfig;
use rustls_platform_verifier::BuilderVerifierExt;
use scraper::{ElementRef, Html, Selector};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tokio_rustls::rustls::pki_types::ServerName;

use super::{RawJob, Result, SourceError};

/// Correos de los últimos días que se revisan en cada consulta (las repetidas se descartan al guardar).
pub const DAYS: u32 = 14;
/// Tope de correos por consulta.
const MAX_MESSAGES: usize = 200;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImapConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Portal {
    LinkedIn,
    Indeed,
    Occ,
    Computrabajo,
    Bumeran,
}

/// Remitente (parte del dominio) → portal.
const PORTALS: [(&str, Portal); 5] = [
    ("linkedin.com", Portal::LinkedIn),
    ("indeed.com", Portal::Indeed),
    ("occ.com.mx", Portal::Occ),
    ("computrabajo.com", Portal::Computrabajo),
    ("bumeran.com", Portal::Bumeran),
];

fn imap_err(e: impl std::fmt::Display) -> SourceError {
    SourceError::Format {
        source_id: "email",
        msg: format!("correo: {e}"),
    }
}

/// "2026-11-06…" → "06-Nov-2026" (formato de fecha de IMAP).
fn imap_date(iso: &str) -> Option<String> {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let (y, rest) = iso.split_once('-')?;
    let (m, rest) = rest.split_once('-')?;
    let d = rest.get(..2)?;
    let month = MONTHS.get(m.parse::<usize>().ok()?.checked_sub(1)?)?;
    Some(format!("{d}-{month}-{y}"))
}

/// Baja los correos de alertas de los últimos [`DAYS`] días (sin marcarlos como leídos).
/// `since_iso` es la fecha desde la que se busca ("AAAA-MM-DD…").
pub async fn fetch(cfg: &ImapConfig, since_iso: &str) -> Result<Vec<Vec<u8>>> {
    let since = imap_date(since_iso).ok_or_else(|| imap_err("fecha inválida"))?;
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let tls = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .and_then(|b| b.with_platform_verifier())
        .map_err(imap_err)?
        .with_no_client_auth();
    let server = ServerName::try_from(cfg.host.clone()).map_err(imap_err)?;
    let tcp = TcpStream::connect((cfg.host.as_str(), cfg.port))
        .await
        .map_err(|e| {
            imap_err(format!(
                "no se pudo conectar a {}:{} ({e})",
                cfg.host, cfg.port
            ))
        })?;
    let stream = TlsConnector::from(Arc::new(tls))
        .connect(server, tcp)
        .await
        .map_err(imap_err)?;
    let mut client = async_imap::Client::new(stream);
    client
        .read_response()
        .await
        .map_err(imap_err)?
        .ok_or_else(|| imap_err("el servidor cerró la conexión"))?;
    let mut session = client
        .login(&cfg.user, &cfg.password)
        .await
        .map_err(|(e, _)| imap_err(format!("inicio de sesión rechazado ({e})")))?;
    session.examine("INBOX").await.map_err(imap_err)?;

    let mut uids: Vec<u32> = Vec::new();
    for (sender, _) in PORTALS {
        // Remitente y fecha son constantes de la app: no entra texto del usuario en la búsqueda.
        let found = session
            .uid_search(format!("SINCE {since} FROM \"{sender}\""))
            .await
            .map_err(imap_err)?;
        uids.extend(found);
    }
    uids.sort_unstable();
    uids.dedup();
    let newest: Vec<String> = uids
        .iter()
        .rev()
        .take(MAX_MESSAGES)
        .map(u32::to_string)
        .collect();

    let mut out = Vec::new();
    if !newest.is_empty() {
        let mut stream = session
            .uid_fetch(newest.join(","), "BODY.PEEK[]")
            .await
            .map_err(imap_err)?;
        while let Some(f) = stream.next().await {
            if let Some(body) = f.map_err(imap_err)?.body() {
                out.push(body.to_vec());
            }
        }
    }
    session.logout().await.ok();
    Ok(out)
}

/// Vacantes de un correo de alerta. Un correo que no es de un portal conocido da lista vacía.
pub fn parse_alert(raw: &[u8]) -> Vec<RawJob> {
    let Some(msg) = MessageParser::default().parse(raw) else {
        return vec![];
    };
    let from = msg
        .from()
        .and_then(|a| a.first())
        .and_then(|a| a.address())
        .unwrap_or_default()
        .to_lowercase();
    let Some(portal) = PORTALS
        .iter()
        .find(|(domain, _)| from.ends_with(domain) || from.contains(&format!("{domain}.")))
        .map(|(_, p)| *p)
    else {
        return vec![];
    };
    let Some(html) = msg.body_html(0) else {
        return vec![];
    };
    jobs_in_html(portal, &html)
}

fn jobs_in_html(portal: Portal, html: &str) -> Vec<RawJob> {
    let doc = Html::parse_document(html);
    let Ok(links) = Selector::parse("a[href]") else {
        return vec![];
    };
    let mut seen = HashSet::new();
    let mut out: Vec<RawJob> = Vec::new();
    for a in doc.select(&links) {
        let Some((id, url)) = a.value().attr("href").and_then(|h| job_link(portal, h)) else {
            continue;
        };
        let title = squash(&a.text().collect::<String>());
        if !looks_like_title(&title) {
            continue;
        }
        if !seen.insert(id.clone()) {
            continue;
        }
        let context = block_lines(a, portal, &id, &title, &links);
        out.push(RawJob {
            external_id: Some(id),
            url,
            company: context.first().cloned(),
            location: context.get(1).cloned(),
            description_html: context
                .iter()
                .map(|l| format!("<p>{}</p>", escape(l)))
                .collect(),
            title,
            ..Default::default()
        });
    }
    out
}

/// (id, URL limpia) si el enlace lleva a una vacante del portal.
fn job_link(portal: Portal, href: &str) -> Option<(String, String)> {
    let url = Url::parse(href).ok()?;
    let host = url.host_str()?.to_lowercase();
    let path = url.path();
    match portal {
        Portal::LinkedIn if host.ends_with("linkedin.com") => {
            // /jobs/view/123/ o /comm/jobs/view/123/
            let rest = path.split("/jobs/view/").nth(1)?;
            let id: String = rest.chars().take_while(char::is_ascii_digit).collect();
            (!id.is_empty()).then(|| {
                (
                    id.clone(),
                    format!("https://www.linkedin.com/jobs/view/{id}/"),
                )
            })
        }
        Portal::Indeed if host.contains("indeed.") => {
            let jk = url.query_pairs().find(|(k, _)| k == "jk")?.1.to_string();
            let ok = !jk.is_empty() && jk.chars().all(|c| c.is_ascii_alphanumeric());
            ok.then(|| (jk.clone(), format!("https://{host}/viewjob?jk={jk}")))
        }
        Portal::Occ if host.ends_with("occ.com.mx") && path.contains("/empleo/oferta/") => {
            let id: String = path
                .split("/empleo/oferta/")
                .nth(1)?
                .chars()
                .take_while(char::is_ascii_digit)
                .collect();
            (!id.is_empty()).then(|| (id, format!("https://{host}{path}")))
        }
        Portal::Computrabajo
            if host.contains("computrabajo.com") && path.contains("/oferta-de-trabajo") =>
        {
            Some((path.to_string(), format!("https://{host}{path}")))
        }
        Portal::Bumeran
            if host.contains("bumeran.com")
                && path.starts_with("/empleos/")
                && path.ends_with(".html") =>
        {
            Some((path.to_string(), format!("https://{host}{path}")))
        }
        _ => None,
    }
}

fn squash(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Descarta botones ("Ver empleo", "See all jobs") y enlaces de imagen.
fn looks_like_title(t: &str) -> bool {
    const GENERIC: [&str; 10] = [
        "ver empleo",
        "ver vacante",
        "ver oferta",
        "ver más",
        "postúlate",
        "postularme",
        "view job",
        "see all jobs",
        "apply",
        "ver todas",
    ];
    let lower = t.to_lowercase();
    t.chars().count() >= 4
        && t.chars().count() <= 150
        && !GENERIC.iter().any(|g| lower.starts_with(g))
}

/// Líneas de texto del bloque que contiene el enlace (empresa, ubicación…), sin el título.
/// Sube por los ancestros hasta antes del que ya incluye el enlace de otra vacante.
fn block_lines(
    a: ElementRef<'_>,
    portal: Portal,
    id: &str,
    title: &str,
    links: &Selector,
) -> Vec<String> {
    let mut best: Vec<String> = Vec::new();
    for node in a.ancestors().take(8) {
        let Some(el) = ElementRef::wrap(node) else {
            break;
        };
        let other_job = el.select(links).any(|l| {
            l.value()
                .attr("href")
                .and_then(|h| job_link(portal, h))
                .is_some_and(|(other, _)| other != id)
        });
        if other_job {
            break;
        }
        best = el
            .text()
            .map(squash)
            .filter(|l| !l.is_empty() && l != title)
            .collect();
    }
    best.truncate(6);
    best
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALERTA: &str = "From: LinkedIn <jobalerts-noreply@linkedin.com>\r
To: yo@example.com\r
Subject: Editor de video: 2 empleos nuevos\r
MIME-Version: 1.0\r
Content-Type: text/html; charset=utf-8\r
\r
<html><body><table>\r
<tr><td><a href=\"https://www.linkedin.com/comm/jobs/view/4012345678/?trackingId=abc\">Editor de video</a>\r
<p>Productora Ejemplo</p><p>Ciudad de México (remoto)</p></td></tr>\r
<tr><td><a href=\"https://www.linkedin.com/comm/jobs/view/4012345678/?trackingId=abc\"><img src=\"logo.png\"></a></td></tr>\r
<tr><td><a href=\"https://www.linkedin.com/comm/jobs/view/4099999999/\">Video Editor Sr.</a>\r
<p>Otra Empresa</p><p>Latinoamérica</p></td></tr>\r
<tr><td><a href=\"https://www.linkedin.com/comm/jobs/search/?keywords=editor\">Ver todas las vacantes</a></td></tr>\r
</table></body></html>\r
";

    #[test]
    fn lee_una_alerta_de_linkedin() {
        let jobs = parse_alert(ALERTA.as_bytes());
        assert_eq!(jobs.len(), 2, "{jobs:#?}");
        let j = &jobs[0];
        assert_eq!(j.title, "Editor de video");
        assert_eq!(j.url, "https://www.linkedin.com/jobs/view/4012345678/");
        assert_eq!(j.company.as_deref(), Some("Productora Ejemplo"));
        assert_eq!(j.location.as_deref(), Some("Ciudad de México (remoto)"));
        assert_eq!(jobs[1].company.as_deref(), Some("Otra Empresa"));
    }

    #[test]
    fn ignora_remitentes_desconocidos() {
        let otro = ALERTA.replace("jobalerts-noreply@linkedin.com", "promo@tienda.com");
        assert!(parse_alert(otro.as_bytes()).is_empty());
    }

    #[test]
    fn enlaces_de_cada_portal() {
        let id = |p, h| job_link(p, h).map(|(id, _)| id);
        assert_eq!(
            id(
                Portal::Indeed,
                "https://mx.indeed.com/rc/clk?jk=a1b2c3d4&from=ja"
            ),
            Some("a1b2c3d4".into())
        );
        assert_eq!(
            id(
                Portal::Occ,
                "https://www.occ.com.mx/empleo/oferta/20123456-editor-de-video/?utm=x"
            ),
            Some("20123456".into())
        );
        assert!(
            id(
                Portal::Computrabajo,
                "https://mx.computrabajo.com/ofertas-de-trabajo/oferta-de-trabajo-de-editor-abc123"
            )
            .is_some()
        );
        assert!(
            id(
                Portal::Bumeran,
                "https://www.bumeran.com.mx/empleos/editor-de-video-acme-1115.html"
            )
            .is_some()
        );
        assert_eq!(id(Portal::LinkedIn, "https://evil.com/jobs/view/123"), None);
        assert_eq!(
            id(Portal::Indeed, "https://mx.indeed.com/viewjob?jk=a'b"),
            None
        );
    }

    #[test]
    fn fecha_imap() {
        assert_eq!(
            imap_date("2026-11-06T10:00:00Z").as_deref(),
            Some("06-Nov-2026")
        );
        assert_eq!(imap_date("2026-13-01"), None);
    }
}
