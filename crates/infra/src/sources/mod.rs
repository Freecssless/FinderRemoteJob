//! Fuentes de vacantes (plan §3).
//!
//! - **Automáticas** ([`Source`]): solo portales con API o RSS públicos. Cada una respeta sus
//!   condiciones de uso: atribución (se guarda el enlace original y `source_id`) y frecuencia
//!   máxima de consulta ([`Source::min_interval_minutes`]).
//! - **Importación manual** ([`manual`]): el usuario pega una URL o el texto de una vacante de
//!   cualquier portal. Con una URL se hace una sola petición, iniciada por él.
//! - **Portales asistidos** ([`ASSISTED_PORTALS`]): LinkedIn, Indeed, OCC… Solo se genera el enlace
//!   de búsqueda para abrirlo en el navegador. Aquí no hay scraping ni postulación automática.

pub mod getonboard;
pub mod jobicy;
pub mod manual;
pub mod remoteok;
pub mod remotive;
pub mod wwr;

use std::time::Duration;

use ego_tree::iter::Edge;
use finder_core::ats::keywords::normalize;
use finder_core::model::{Id, Job, dedupe_key};
use reqwest::{Client, StatusCode, Url};
use scraper::{Html, Node};

use crate::db::{Db, DbError};

#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error("error de red: {0}")]
    Http(#[from] reqwest::Error),
    #[error("{url} respondió {status}")]
    Status { status: StatusCode, url: String },
    #[error(
        "el portal no permite leer esta página ({status}); copia y pega el texto de la vacante"
    )]
    Blocked { status: StatusCode },
    #[error("no se encontró el texto de la vacante en la página; copia y pega el texto")]
    NoContent,
    #[error("respuesta de {source_id} con formato inesperado: {msg}")]
    Format {
        source_id: &'static str,
        msg: String,
    },
    #[error(transparent)]
    Db(#[from] DbError),
}

pub type Result<T> = std::result::Result<T, SourceError>;

/// Vacante tal como llega de una fuente, antes de normalizarla a [`Job`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RawJob {
    pub external_id: Option<String>,
    pub url: String,
    pub company: Option<String>,
    pub title: String,
    pub location: Option<String>,
    pub remote_type: Option<String>,
    pub salary_min: Option<i64>,
    pub salary_max: Option<i64>,
    pub currency: Option<String>,
    /// HTML (las fuentes lo entregan así); se convierte a texto en [`RawJob::into_job`].
    pub description_html: String,
    pub language: Option<String>,
    /// ISO-8601.
    pub posted_at: Option<String>,
}

impl RawJob {
    pub fn into_job(self, source_id: &str, now: &str) -> Job {
        let dedupe_hash = dedupe_key(
            self.company.as_deref(),
            &self.title,
            self.location.as_deref(),
        );
        Job {
            id: Db::new_id(),
            source_id: source_id.to_string(),
            external_id: self.external_id,
            url: self.url,
            company: self.company.filter(|c| !c.trim().is_empty()),
            title: self.title.trim().to_string(),
            location: self.location.filter(|l| !l.trim().is_empty()),
            remote_type: self.remote_type,
            salary_min: self.salary_min,
            salary_max: self.salary_max,
            currency: self.currency,
            description: html_to_text(&self.description_html),
            language: self.language,
            requirements: None,
            posted_at: self.posted_at,
            found_at: now.to_string(),
            dedupe_hash,
        }
    }
}

/// Qué buscar. `keywords` suele venir de los puestos objetivo del perfil ("Editor de video").
#[derive(Debug, Clone, Default)]
pub struct Query {
    pub keywords: Vec<String>,
}

// ---------------------------------------------------------------------------
// Fuentes automáticas
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    RemoteOk,
    Remotive,
    Wwr,
    GetOnBoard,
    Jobicy,
}

impl Source {
    pub const ALL: [Source; 5] = [
        Source::RemoteOk,
        Source::Remotive,
        Source::Wwr,
        Source::GetOnBoard,
        Source::Jobicy,
    ];

    /// Igual que `sources.id` en la base de datos.
    pub fn id(self) -> &'static str {
        match self {
            Source::RemoteOk => "remoteok",
            Source::Remotive => "remotive",
            Source::Wwr => "wwr",
            Source::GetOnBoard => "getonboard",
            Source::Jobicy => "jobicy",
        }
    }

    /// Minutos mínimos entre consultas. Remotive pide máximo 4 al día; para las demás, 1 hora
    /// es más que suficiente (sus datos no cambian más rápido).
    pub fn min_interval_minutes(self) -> u32 {
        match self {
            Source::Remotive => 6 * 60,
            _ => 60,
        }
    }

    /// Descarga y lee las vacantes (sin filtrar ni guardar).
    pub async fn fetch(self, client: &Client, q: &Query) -> Result<Vec<RawJob>> {
        match self {
            Source::RemoteOk => remoteok::parse(&get_text(client, remoteok::URL).await?),
            Source::Remotive => remotive::parse(&get_text(client, remotive::URL).await?),
            Source::Wwr => wwr::parse(&get_text(client, wwr::URL).await?),
            Source::Jobicy => jobicy::parse(&get_text(client, jobicy::URL).await?),
            Source::GetOnBoard => {
                // Su API busca por texto: una consulta por palabra clave (máx. 3).
                let mut all: Vec<RawJob> = Vec::new();
                for kw in q.keywords.iter().take(3) {
                    for job in
                        getonboard::parse(&get_text(client, getonboard::url(kw)?.as_str()).await?)?
                    {
                        if !all.iter().any(|j| j.external_id == job.external_id) {
                            all.push(job);
                        }
                    }
                }
                Ok(all)
            }
        }
    }
}

/// Resultado de sincronizar una fuente.
#[derive(Debug, Clone, PartialEq)]
pub struct SyncReport {
    pub source: &'static str,
    /// `true` si no se consultó porque la última vez fue hace muy poco.
    pub skipped_too_soon: bool,
    pub fetched: usize,
    /// Las que pasaron el filtro de palabras clave.
    pub kept: usize,
    /// Ids de las vacantes nuevas (para calcular su compatibilidad con cada perfil).
    pub new_jobs: Vec<Id>,
}

/// Consulta la fuente (si ya toca), filtra por palabras clave y guarda las vacantes nuevas.
pub async fn sync(db: &Db, client: &Client, source: Source, q: &Query) -> Result<SyncReport> {
    let mut report = SyncReport {
        source: source.id(),
        skipped_too_soon: false,
        fetched: 0,
        kept: 0,
        new_jobs: Vec::new(),
    };
    if !db
        .source_due(source.id(), source.min_interval_minutes())
        .await?
    {
        report.skipped_too_soon = true;
        return Ok(report);
    }
    let raw = source.fetch(client, q).await?;
    // Se marca aunque no haya resultados: la consulta ya se hizo y cuenta para el límite.
    db.mark_source_synced(source.id()).await?;
    report.fetched = raw.len();

    let now = db.now().await?;
    for r in raw.into_iter().filter(|r| matches_keywords(r, &q.keywords)) {
        report.kept += 1;
        let inserted = db.insert_job(&r.into_job(source.id(), &now)).await?;
        if inserted.created {
            report.new_jobs.push(inserted.id);
        }
    }
    Ok(report)
}

/// Palabras que no ayudan a decidir si una vacante es del oficio buscado.
const KEYWORD_STOPWORDS: &[&str] = &[
    "de", "del", "la", "el", "en", "y", "para", "con", "the", "of", "and", "for", "jr", "sr",
    "junior", "senior", "remoto", "remote",
];

/// ¿La vacante menciona todas las palabras significativas de alguno de los puestos buscados?
/// "Editor de video" acepta "Video Editor". Sin palabras clave, se aceptan todas.
pub fn matches_keywords(job: &RawJob, keywords: &[String]) -> bool {
    if keywords.is_empty() {
        return true;
    }
    let text = normalize(&format!("{} {}", job.title, job.description_html));
    let words: Vec<&str> = text.split(|c: char| !c.is_alphanumeric()).collect();
    keywords.iter().any(|kw| {
        let kw = normalize(kw);
        let mut wanted = kw
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() >= 2 && !KEYWORD_STOPWORDS.contains(w))
            .peekable();
        wanted.peek().is_some() && wanted.all(|w| words.contains(&w))
    })
}

// ---------------------------------------------------------------------------
// HTTP
// ---------------------------------------------------------------------------

/// Cliente HTTP de la app: identificación honesta, tiempo de espera y sin cookies.
pub fn http_client() -> Result<Client> {
    Ok(Client::builder()
        .user_agent(concat!(
            "FinderRemoteJob/",
            env!("CARGO_PKG_VERSION"),
            " (app personal de búsqueda de empleo)"
        ))
        .timeout(Duration::from_secs(30))
        .build()?)
}

async fn get_text(client: &Client, url: &str) -> Result<String> {
    let resp = client.get(url).send().await?;
    let status = resp.status();
    if !status.is_success() {
        return Err(SourceError::Status {
            status,
            url: url.to_string(),
        });
    }
    Ok(resp.text().await?)
}

/// Deserializa cada elemento por separado: si un anuncio viene mal formado se omite,
/// en vez de perder toda la respuesta.
fn parse_each<T: serde::de::DeserializeOwned>(items: Vec<serde_json::Value>) -> Vec<T> {
    items
        .into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect()
}

fn format_err(source_id: &'static str, e: impl std::fmt::Display) -> SourceError {
    SourceError::Format {
        source_id,
        msg: e.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Texto y fechas
// ---------------------------------------------------------------------------

const BLOCK_TAGS: &[&str] = &[
    "p",
    "div",
    "section",
    "article",
    "main",
    "ul",
    "ol",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "tr",
    "table",
    "blockquote",
    "pre",
];
/// Su contenido no es parte del anuncio.
const SKIP_TAGS: &[&str] = &[
    "script", "style", "noscript", "nav", "header", "footer", "svg", "form", "template",
];

/// HTML → texto legible: respeta párrafos, viñetas ("- ") y saltos de línea, y descarta
/// scripts, estilos y la navegación del sitio.
pub fn html_to_text(html: &str) -> String {
    let doc = Html::parse_fragment(html);
    let mut out = String::new();
    let mut skipping = 0usize;
    for edge in doc.tree.root().traverse() {
        match edge {
            Edge::Open(node) => match node.value() {
                Node::Element(e) if SKIP_TAGS.contains(&e.name()) => skipping += 1,
                _ if skipping > 0 => {}
                Node::Text(t) => out.push_str(t),
                Node::Element(e) => match e.name() {
                    "br" => out.push('\n'),
                    "li" => out.push_str("\n- "),
                    name if BLOCK_TAGS.contains(&name) => out.push('\n'),
                    _ => {}
                },
                _ => {}
            },
            Edge::Close(node) => {
                if let Node::Element(e) = node.value() {
                    if SKIP_TAGS.contains(&e.name()) {
                        skipping = skipping.saturating_sub(1);
                    } else if skipping == 0 && BLOCK_TAGS.contains(&e.name()) {
                        out.push('\n');
                    }
                }
            }
        }
    }
    tidy(&out)
}

/// Espacios simples por línea, sin más de una línea vacía seguida.
fn tidy(text: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    for line in text.lines() {
        let line = line.split_whitespace().collect::<Vec<_>>().join(" ");
        if line.is_empty() && out.last().is_none_or(|l| l.is_empty()) {
            continue;
        }
        out.push(line);
    }
    while out.last().is_some_and(|l| l.is_empty()) {
        out.pop();
    }
    out.join("\n")
}

/// Segundos Unix → "AAAA-MM-DDTHH:MM:SSZ" (algoritmo de días civiles de H. Hinnant).
pub fn epoch_to_iso(secs: i64) -> String {
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

/// Fecha RFC 2822 de RSS ("Fri, 16 Oct 2026 14:21:11 +0000") → ISO.
/// shortcut: ignora la zona horaria (WWR siempre usa +0000); convertir si aparece otra fuente RSS con otra zona.
pub fn rfc2822_to_iso(s: &str) -> Option<String> {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let parts: Vec<&str> = s.split_whitespace().collect();
    let [_, day, mon, year, time, ..] = parts.as_slice() else {
        return None;
    };
    let month = MONTHS.iter().position(|m| m == mon)? + 1;
    let day: u32 = day.parse().ok()?;
    let year: u32 = year.parse().ok()?;
    (time.len() == 8).then(|| format!("{year:04}-{month:02}-{day:02}T{time}Z"))
}

// ---------------------------------------------------------------------------
// Portales asistidos (sin API): solo enlaces para abrir en el navegador
// ---------------------------------------------------------------------------

/// Portal sin API oficial. La app solo arma el enlace de búsqueda; el usuario navega con su
/// propia cuenta y, si le interesa una vacante, la importa con [`manual`].
/// Fase 2: aquí se engancha la postulación asistida (abrir, copiar carta, confirmar "Ya me postulé").
#[derive(Debug, Clone, Copy)]
pub struct AssistedPortal {
    pub id: &'static str,
    pub name: &'static str,
    search: fn(&str) -> String,
}

impl AssistedPortal {
    pub fn search_url(&self, keywords: &str) -> String {
        (self.search)(keywords)
    }
}

fn with_query(base: &str, params: &[(&str, &str)]) -> String {
    Url::parse_with_params(base, params)
        .map(String::from)
        .unwrap_or_else(|_| base.to_string())
}

/// "Editor de Vídeo" → "editor-de-video" (para portales que buscan por ruta).
fn slug(keywords: &str) -> String {
    normalize(keywords)
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

pub const ASSISTED_PORTALS: &[AssistedPortal] = &[
    AssistedPortal {
        id: "linkedin",
        name: "LinkedIn",
        // f_WT=2: filtro "Remoto" de LinkedIn.
        search: |k| {
            with_query(
                "https://www.linkedin.com/jobs/search/",
                &[("keywords", k), ("f_WT", "2")],
            )
        },
    },
    AssistedPortal {
        id: "indeed",
        name: "Indeed México",
        search: |k| with_query("https://mx.indeed.com/jobs", &[("q", k), ("l", "Remoto")]),
    },
    AssistedPortal {
        id: "occ",
        name: "OCC Mundial",
        search: |k| format!("https://www.occ.com.mx/empleos/de-{}/", slug(k)),
    },
    AssistedPortal {
        id: "computrabajo",
        name: "Computrabajo México",
        search: |k| format!("https://mx.computrabajo.com/trabajo-de-{}", slug(k)),
    },
    AssistedPortal {
        id: "bumeran",
        name: "Bumeran México",
        search: |k| {
            format!(
                "https://www.bumeran.com.mx/empleos-busqueda-{}.html",
                slug(k)
            )
        },
    },
    AssistedPortal {
        id: "glassdoor",
        name: "Glassdoor México",
        search: |k| {
            with_query(
                "https://www.glassdoor.com.mx/Empleo/empleos.htm",
                &[("sc.keyword", k)],
            )
        },
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_a_texto_respeta_parrafos_y_vinetas() {
        let html = "<p>Buscamos <strong>editor</strong>.</p><h3>Requisitos</h3><ul><li>Premiere</li><li>After&nbsp;Effects</li></ul>\
                    <script>alert(1)</script><style>p{}</style><p>Línea 1<br>Línea 2</p>";
        assert_eq!(
            html_to_text(html),
            "Buscamos editor.\n\nRequisitos\n\n- Premiere\n- After Effects\n\nLínea 1\nLínea 2"
        );
        assert_eq!(html_to_text("texto plano"), "texto plano");
    }

    #[test]
    fn fechas_unix_y_rss_a_iso() {
        assert_eq!(epoch_to_iso(0), "1970-01-01T00:00:00Z");
        assert_eq!(epoch_to_iso(1_793_583_808), "2026-11-02T01:43:28Z");
        assert_eq!(
            epoch_to_iso(951_782_400),
            "2000-02-29T00:00:00Z",
            "año bisiesto"
        );
        assert_eq!(
            rfc2822_to_iso("Fri, 16 Oct 2026 14:21:11 +0000").as_deref(),
            Some("2026-10-16T14:21:11Z")
        );
        assert_eq!(rfc2822_to_iso("ayer"), None);
    }

    #[test]
    fn filtro_por_palabras_clave() {
        let job = |title: &str| RawJob {
            title: title.into(),
            ..Default::default()
        };
        let kw = vec!["Editor de video".to_string()];
        assert!(matches_keywords(&job("Video Editor (Remote)"), &kw));
        assert!(matches_keywords(&job("Editor de Vídeo Jr"), &kw));
        assert!(!matches_keywords(&job("Software Engineer"), &kw));
        assert!(
            !matches_keywords(&job("Editor de textos"), &kw),
            "faltan palabras"
        );
        assert!(
            matches_keywords(&job("Lo que sea"), &[]),
            "sin palabras clave pasa todo"
        );
        assert!(
            !matches_keywords(&job("Remote role"), &["de la".to_string()]),
            "solo conectores no acepta nada"
        );
    }

    #[test]
    fn raw_job_se_normaliza_a_job() {
        let raw = RawJob {
            url: "https://x".into(),
            company: Some("  ".into()),
            title: " Editor ".into(),
            description_html: "<p>Hola</p>".into(),
            ..Default::default()
        };
        let job = raw.into_job("manual", "2026-11-05T00:00:00Z");
        assert_eq!(job.title, "Editor");
        assert_eq!(job.company, None, "empresa vacía = desconocida");
        assert_eq!(job.description, "Hola");
        assert_eq!(job.dedupe_hash, dedupe_key(None, "Editor", None));
    }

    #[test]
    fn enlaces_de_portales_asistidos() {
        let url = |id: &str| {
            ASSISTED_PORTALS
                .iter()
                .find(|p| p.id == id)
                .map(|p| p.search_url("Editor de Vídeo"))
        };
        assert_eq!(
            url("linkedin").as_deref(),
            Some("https://www.linkedin.com/jobs/search/?keywords=Editor+de+V%C3%ADdeo&f_WT=2")
        );
        assert_eq!(
            url("occ").as_deref(),
            Some("https://www.occ.com.mx/empleos/de-editor-de-video/")
        );
        assert_eq!(
            url("computrabajo").as_deref(),
            Some("https://mx.computrabajo.com/trabajo-de-editor-de-video")
        );
    }

    #[test]
    fn ids_de_fuentes_coinciden_con_la_migracion() {
        let sql = include_str!("../../migrations/0001_init.sql");
        for s in Source::ALL {
            assert!(
                sql.contains(&format!("('{}',", s.id())),
                "falta {} en la migración",
                s.id()
            );
        }
    }

    /// Consulta real a internet. Correr a mano: `cargo test -p finder-infra -- --ignored --nocapture`
    #[tokio::test]
    #[ignore]
    async fn en_vivo_todas_las_fuentes_responden()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let client = http_client()?;
        let q = Query {
            keywords: vec!["editor".into()],
        };
        for s in Source::ALL {
            let jobs = s.fetch(&client, &q).await?;
            println!("{:<11} {:>4} vacantes", s.id(), jobs.len());
            assert!(!jobs.is_empty(), "{} no devolvió vacantes", s.id());
        }
        Ok(())
    }
}
