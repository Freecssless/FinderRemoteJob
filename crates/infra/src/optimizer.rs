//! Optimizador de CV para una vacante (plan §4.2). Une todos los módulos:
//!
//! 1. [`prepare`]: requisitos (diccionario + IA local, o solo diccionario si la IA no está),
//!    puntaje ATS **antes**, huecos e idioma.
//! 2. [`optimize`]: el redactor propone un [`Draft`] y se aplica con [`apply_draft`].
//! 3. [`apply_draft`]: guardián → CV final → PDF → ida y vuelta → puntaje ATS **después**.
//! 4. [`save`]: con las decisiones del usuario en el diff, genera los archivos, guarda la
//!    versión vinculada a la postulación y la pasa a "CV adaptado".
//!
//! `prepare` y `apply_draft` no llaman a la IA: se prueban con borradores escritos a mano.

use std::path::{Path, PathBuf};

use finder_core::ats::keywords::{SkillDict, extract_terms};
use finder_core::ats::score::{AtsReport, Gaps, ats_score, classify_gaps, with_roundtrip};
use finder_core::document::{Choices, CvDocument, DocLang};
use finder_core::guard::{Draft, GuardReport, validate};
use finder_core::matching::years_of_experience;
use finder_core::model::{Basics, CvMaster, CvVersion, Job, JobRequirements, Status};
use serde::Serialize;
use serde_json::{Value, json};

use crate::cvfile::CvFileKind;
use crate::db::{Db, DbError};
use crate::docgen::{self, DocError, RoundTrip};
use crate::llm::tasks::{self, TaskError};
use crate::llm::{LlmClient, LlmError};

#[derive(Debug, thiserror::Error)]
pub enum OptimizeError {
    #[error(transparent)]
    Llm(#[from] LlmError),
    #[error(transparent)]
    Doc(#[from] DocError),
    #[error(transparent)]
    Db(#[from] DbError),
    #[error("no se pudo guardar el archivo: {0}")]
    Io(#[from] std::io::Error),
    #[error("el redactor devolvió datos con otra forma: {0}")]
    Shape(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, OptimizeError>;

/// Datos de entrada para optimizar un CV.
pub struct OptimizeInput<'a> {
    pub master: &'a CvMaster,
    /// Texto del archivo importado (CV real del usuario), para el puntaje "antes".
    pub original_text: Option<&'a str>,
    pub job: &'a Job,
    /// Mes actual "AAAA-MM" (años de experiencia que puede citar el resumen).
    pub today: &'a str,
}

/// Paso 1: todo lo que se sabe antes de pedirle nada al redactor.
#[derive(Debug, Clone, Serialize)]
pub struct Prepared {
    pub requirements: JobRequirements,
    /// Avisos para el usuario (p. ej. "la IA local no respondió, se usó solo el diccionario").
    pub notes: Vec<String>,
    pub lang: DocLang,
    pub ats_before: AtsReport,
    pub gaps: Gaps,
}

/// Resultado completo, listo para la pantalla de revisión (diff).
#[derive(Debug, Clone, Serialize)]
pub struct Optimization {
    pub prepared: Prepared,
    pub draft: Draft,
    /// Cada propuesta con su estado (aceptada o rechazada con motivo) y su texto original.
    pub review: GuardReport,
    /// Decisiones por defecto: todo lo que aprobó el guardián.
    pub choices: Choices,
    pub document: CvDocument,
    pub ats_after: AtsReport,
    pub roundtrip: RoundTrip,
}

pub async fn prepare(
    local: Option<&LlmClient>,
    dict: &SkillDict,
    input: &OptimizeInput<'_>,
) -> Prepared {
    let job = input.job;
    let mut notes = Vec::new();
    let requirements = match (&job.requirements, local) {
        (Some(r), _) => r.clone(),
        (None, Some(llm)) => {
            match tasks::extract_requirements(llm, dict, &job.title, &job.description).await {
                Ok(r) => r,
                Err(e) => {
                    notes.push(format!(
                        "La IA local no pudo analizar la vacante ({e}); se usó solo el diccionario."
                    ));
                    dictionary_only(dict, job)
                }
            }
        }
        (None, None) => dictionary_only(dict, job),
    };
    prepare_with(dict, input, requirements, notes)
}

fn dictionary_only(dict: &SkillDict, job: &Job) -> JobRequirements {
    JobRequirements {
        terms: extract_terms(dict, &job.title, &job.description),
        ..Default::default()
    }
}

/// Parte sin IA de [`prepare`].
pub fn prepare_with(
    dict: &SkillDict,
    input: &OptimizeInput<'_>,
    requirements: JobRequirements,
    notes: Vec<String>,
) -> Prepared {
    let job = input.job;
    let lang = match job.language.as_deref() {
        Some("en") => DocLang::En,
        Some("es") => DocLang::Es,
        _ => DocLang::detect(&job.description),
    };
    // "Antes" = tu CV real si lo importaste; si no, el maestro con la plantilla base.
    let before_text = input
        .original_text
        .map(String::from)
        .unwrap_or_else(|| CvDocument::from_master(input.master, lang).to_plain_text());
    let ats_before = ats_score(dict, &before_text, &job.title, &requirements.terms);
    let gaps = classify_gaps(dict, &ats_before.missing, input.master);
    Prepared {
        requirements,
        notes,
        lang,
        ats_before,
        gaps,
    }
}

/// Pasos 2 y 3: pide el borrador al redactor y lo aplica.
pub async fn optimize(
    writer: &LlmClient,
    dict: &SkillDict,
    input: &OptimizeInput<'_>,
    prepared: Prepared,
) -> Result<Optimization> {
    let years = years_of_experience(input.master, input.today).floor() as u32;
    let user = writer_request(input.master, input.job, &prepared, years)?;
    let value = writer
        .complete_json(&writer_system(prepared.lang), &user, &draft_schema())
        .await?;
    let draft: Draft = serde_json::from_value(value)?;
    apply_draft(dict, input, prepared, draft)
}

/// Paso 3: guardián → documento → PDF → ida y vuelta → puntaje "después".
pub fn apply_draft(
    dict: &SkillDict,
    input: &OptimizeInput<'_>,
    prepared: Prepared,
    draft: Draft,
) -> Result<Optimization> {
    let review = validate(dict, input.master, &draft, input.today);
    let choices = Choices::accept_all(&review);
    let document = CvDocument::from_report(input.master, &review, &choices, prepared.lang);
    let (ats_after, roundtrip) =
        score_rendered(dict, &document, input.job, &prepared.requirements)?;
    Ok(Optimization {
        prepared,
        draft,
        review,
        choices,
        document,
        ats_after,
        roundtrip,
    })
}

fn score_rendered(
    dict: &SkillDict,
    doc: &CvDocument,
    job: &Job,
    req: &JobRequirements,
) -> Result<(AtsReport, RoundTrip)> {
    let pdf = docgen::render_pdf(doc)?;
    let rt = docgen::verify(doc, &pdf, CvFileKind::Pdf)?;
    let ats = with_roundtrip(
        ats_score(dict, &rt.text, &job.title, &req.terms),
        rt.missing_ratio,
        rt.sections_in_order,
    );
    Ok((ats, rt))
}

/// Dónde se guarda el CV adaptado.
pub struct SaveTarget<'a> {
    pub profile_id: &'a str,
    pub application_id: &'a str,
    /// Carpeta de los archivos (la app usa `<datos de la app>/cvs/`).
    pub out_dir: &'a Path,
}

/// Paso 4: aplica las decisiones del usuario, escribe PDF y DOCX en `out_dir`, guarda la versión,
/// la vincula a la postulación y la pasa a "CV adaptado" (sin retroceder si ya estaba más adelante).
pub async fn save(
    db: &Db,
    dict: &SkillDict,
    input: &OptimizeInput<'_>,
    opt: &Optimization,
    choices: &Choices,
    target: &SaveTarget<'_>,
) -> Result<CvVersion> {
    let doc = CvDocument::from_report(input.master, &opt.review, choices, opt.prepared.lang);
    let pdf = docgen::render_pdf(&doc)?;
    let docx = docgen::render_docx(&doc)?;
    let rt = docgen::verify(&doc, &pdf, CvFileKind::Pdf)?;
    let ats_after = with_roundtrip(
        ats_score(
            dict,
            &rt.text,
            &input.job.title,
            &opt.prepared.requirements.terms,
        ),
        rt.missing_ratio,
        rt.sections_in_order,
    );

    let id = Db::new_id();
    // Sufijo con el id para no sobrescribir versiones anteriores para la misma vacante.
    let stem = format!(
        "{}_{}",
        doc.file_stem(&input.job.title, input.job.company.as_deref()),
        id.get(..8).unwrap_or(&id)
    );
    std::fs::create_dir_all(target.out_dir)?;
    let pdf_path = write(target.out_dir, &stem, "pdf", &pdf)?;
    let docx_path = write(target.out_dir, &stem, "docx", &docx)?;

    let version = CvVersion {
        id,
        profile_id: target.profile_id.to_string(),
        job_id: Some(input.job.id.clone()),
        variant: "clasica".into(),
        content_json: serde_json::to_string(&doc)?,
        master_hash: master_hash(input.master)?,
        ats_before: Some(opt.prepared.ats_before.total),
        ats_after: Some(ats_after.total),
        missing: ats_after
            .missing
            .iter()
            .map(|t| t.surface.clone())
            .collect(),
        pdf_path: Some(pdf_path.display().to_string()),
        docx_path: Some(docx_path.display().to_string()),
        created_at: db.now().await?,
    };
    db.insert_cv_version(&version).await?;
    db.attach_cv_version(target.application_id, &version.id, None)
        .await?;
    if let Some(app) = db.get_application(target.application_id).await?
        && matches!(app.status, Status::Encontrada | Status::Evaluada)
    {
        db.set_status(target.application_id, Status::CvAdaptado)
            .await?;
    }
    Ok(version)
}

fn write(dir: &Path, stem: &str, ext: &str, bytes: &[u8]) -> Result<PathBuf> {
    let path = dir.join(format!("{stem}.{ext}"));
    std::fs::write(&path, bytes)?;
    Ok(path)
}

/// Huella del CV maestro (FNV-1a de 64 bits sobre su JSON): identifica de qué versión salió el CV.
pub fn master_hash(m: &CvMaster) -> Result<String> {
    let json = serde_json::to_string(m)?;
    let hash = json.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    });
    Ok(format!("{hash:016x}"))
}

// ---------------------------------------------------------------------------
// Petición al redactor
// ---------------------------------------------------------------------------

fn writer_system(lang: DocLang) -> String {
    let idioma = match lang {
        DocLang::Es => "español",
        DocLang::En => "inglés",
    };
    format!(
        "Eres experto en adaptar CVs para sistemas ATS. Reescribes los logros de un CV maestro para una vacante concreta.
Reglas obligatorias:
1. Solo puedes reformular, reordenar y priorizar lo que dice el CV maestro. Nunca agregues experiencia, habilidades, herramientas, cifras, empresas, títulos ni logros que no estén en él.
2. Cada bullet indica su experiencia en \"exp_id\" y cita en \"source_ids\" los logros (b_xx) de esa misma experiencia de los que sale. Puedes combinar logros de la misma experiencia.
3. Conserva cada cifra exactamente como aparece en sus fuentes.
4. Cuando el CV respalde un término de la vacante, escríbelo con la forma exacta del anuncio (por ejemplo \"Adobe Premiere Pro\" en lugar de \"Premiere\").
5. Empieza cada bullet con un verbo de acción en pasado y destaca el resultado.
6. Máximo 5 bullets por experiencia, del más al menos relevante para la vacante.
7. \"summary\": 2 o 3 frases que presenten al candidato para este puesto, citando en \"source_ids\" los ids que lo respaldan (b_xx, sk_xx o exp_xx). Si mencionas años de experiencia, usa exactamente los años indicados.
8. \"skills_order\": ids sk_xx del CV maestro, del más al menos relevante para la vacante.
9. Escribe todo en {idioma}."
    )
}

/// Lo que ve el redactor. Sin nombre, correo, teléfono, ubicación ni enlaces: no los necesita,
/// y así no salen del equipo aunque el redactor sea una API externa.
fn writer_request(master: &CvMaster, job: &Job, prepared: &Prepared, years: u32) -> Result<String> {
    let mut cv = master.clone();
    cv.basics = Basics {
        headline: master.basics.headline.clone(),
        ..Default::default()
    };
    let terms: Vec<Value> = prepared
        .requirements
        .terms
        .iter()
        .map(|t| json!({ "termino": t.surface, "requerido": t.required }))
        .collect();
    let respaldados: Vec<&str> = prepared
        .ats_before
        .synonym_only
        .iter()
        .map(|s| s.job_surface.as_str())
        .chain(
            prepared
                .gaps
                .with_evidence
                .iter()
                .map(|t| t.surface.as_str()),
        )
        .collect();
    Ok(serde_json::to_string_pretty(&json!({
        "vacante": { "puesto": job.title, "empresa": job.company, "descripcion": job.description },
        "terminos_de_la_vacante": terms,
        "terminos_que_el_cv_respalda_pero_no_muestra_con_esa_forma": respaldados,
        "anos_de_experiencia": years,
        "cv_maestro": cv,
    }))?)
}

/// Esquema del [`Draft`] (estricto: todas las propiedades requeridas, sin propiedades extra).
pub fn draft_schema() -> Value {
    let ids = json!({"type": "array", "items": {"type": "string"}});
    json!({
        "type": "object", "additionalProperties": false,
        "required": ["summary", "bullets", "skills_order"],
        "properties": {
            "summary": {"anyOf": [
                {"type": "object", "additionalProperties": false, "required": ["text", "source_ids"],
                 "properties": {"text": {"type": "string"}, "source_ids": ids}},
                {"type": "null"}
            ]},
            "bullets": {"type": "array", "items": {
                "type": "object", "additionalProperties": false, "required": ["exp_id", "text", "source_ids"],
                "properties": {"exp_id": {"type": "string"}, "text": {"type": "string"}, "source_ids": ids}
            }},
            "skills_order": ids
        }
    })
}

/// Ejecuta todo con los clientes de la configuración (atajo para la app).
pub async fn run(
    local: Option<&LlmClient>,
    writer: &LlmClient,
    dict: &SkillDict,
    input: &OptimizeInput<'_>,
) -> std::result::Result<Optimization, OptimizeError> {
    let prepared = prepare(local, dict, input).await;
    optimize(writer, dict, input, prepared).await
}

impl From<TaskError> for OptimizeError {
    fn from(e: TaskError) -> Self {
        match e {
            TaskError::Llm(e) => OptimizeError::Llm(e),
            TaskError::Shape(e) => OptimizeError::Shape(e),
            TaskError::File(e) => OptimizeError::Doc(DocError::Read(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use finder_core::guard::{DraftBullet, DraftText, Issue};
    use finder_core::model::{RemotePref, dedupe_key};

    type R = std::result::Result<(), Box<dyn std::error::Error>>;
    const TODAY: &str = "2026-11";
    const JD: &str = include_str!("../../core/tests/fixtures/jd_editor_video_es.txt");
    const CV_TXT: &str = include_str!("../../core/tests/fixtures/cv_editor_video.txt");

    fn master() -> std::result::Result<CvMaster, serde_json::Error> {
        serde_json::from_str(include_str!(
            "../../core/tests/fixtures/cv_editor_video.json"
        ))
    }

    fn job() -> Job {
        let title = "Editor de Video (Remoto - LATAM)";
        Job {
            id: Db::new_id(),
            source_id: "manual".into(),
            external_id: None,
            url: "https://example.com".into(),
            company: Some("Agencia X".into()),
            title: title.into(),
            location: None,
            remote_type: Some("remote".into()),
            salary_min: None,
            salary_max: None,
            currency: None,
            description: JD.into(),
            language: None,
            requirements: None,
            posted_at: None,
            found_at: "2026-11-05T00:00:00Z".into(),
            dedupe_hash: dedupe_key(Some("Agencia X"), title, None),
        }
    }

    fn b(exp: &str, text: &str, src: &[&str]) -> DraftBullet {
        DraftBullet {
            exp_id: exp.into(),
            text: text.into(),
            source_ids: src.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Lo que devolvería un buen redactor: reformula, usa términos exactos y cita fuentes.
    fn honest_draft() -> Draft {
        Draft {
            summary: Some(DraftText {
                text: "Editor de video con 6 años de experiencia en edición de video para YouTube con Adobe Premiere Pro y After Effects.".into(),
                source_ids: vec!["exp_01".into(), "b_01".into(), "sk_01".into(), "sk_02".into()],
            }),
            bullets: vec![
                b("exp_01", "Edité 120 videos para YouTube con Adobe Premiere Pro y After Effects", &["b_01"]),
                b("exp_01", "Reduje 30% el tiempo de entrega creando plantillas de edición reutilizables", &["b_02"]),
                b("exp_02", "Realicé corrección de color y sonido para comerciales de TV", &["b_03"]),
            ],
            skills_order: vec!["sk_01".into(), "sk_02".into(), "sk_03".into()],
        }
    }

    #[tokio::test]
    async fn flujo_completo_sube_el_puntaje_sin_inventar() -> R {
        let dict = SkillDict::builtin()?;
        let (m, j) = (master()?, job());
        let input = OptimizeInput {
            master: &m,
            original_text: Some(CV_TXT),
            job: &j,
            today: TODAY,
        };

        let prepared = prepare(None, &dict, &input).await;
        assert_eq!(prepared.lang, DocLang::Es);
        assert!(
            prepared
                .gaps
                .without_evidence
                .iter()
                .any(|t| t.canonical == "TikTok")
        );

        let opt = apply_draft(&dict, &input, prepared, honest_draft())?;
        println!("ATS antes {} → después {}", opt.prepared.ats_before.total, opt.ats_after.total);
        assert!(opt.review.all_accepted(), "{:?}", opt.review);
        assert!(opt.roundtrip.ok(), "{:?}", opt.roundtrip.missing_words);
        assert!(
            opt.ats_after.total > opt.prepared.ats_before.total,
            "antes {} → después {}",
            opt.prepared.ats_before.total,
            opt.ats_after.total
        );
        // TikTok no está en el maestro: sigue faltando, nadie lo inventó.
        assert!(
            opt.ats_after
                .missing
                .iter()
                .any(|t| t.canonical == "TikTok")
        );
        Ok(())
    }

    #[test]
    fn borrador_tramposo_vuelve_al_original() -> R {
        let dict = SkillDict::builtin()?;
        let (m, j) = (master()?, job());
        let input = OptimizeInput {
            master: &m,
            original_text: None,
            job: &j,
            today: TODAY,
        };
        let prepared = prepare_with(&dict, &input, dictionary_only(&dict, &j), vec![]);
        let mut draft = honest_draft();
        draft.bullets[0] = b(
            "exp_01",
            "Edité 120 videos para TikTok e Instagram con Final Cut Pro",
            &["b_01"],
        );

        let opt = apply_draft(&dict, &input, prepared, draft)?;
        let issues = &opt.review.bullets[0].issues;
        assert!(issues.contains(&Issue::NewSkill("TikTok".into())));
        assert!(issues.contains(&Issue::NewSkill("Final Cut Pro".into())));
        assert_eq!(opt.choices.bullets, [false, true, true]);
        let text = opt.document.to_plain_text();
        assert!(
            !text.contains("TikTok") && !text.contains("Final Cut"),
            "{text}"
        );
        assert!(text.contains("Edité 120 videos para YouTube con Premiere y After Effects"));
        Ok(())
    }

    #[test]
    fn el_redactor_no_recibe_datos_personales() -> R {
        let dict = SkillDict::builtin()?;
        let (m, j) = (master()?, job());
        let input = OptimizeInput {
            master: &m,
            original_text: None,
            job: &j,
            today: TODAY,
        };
        let prepared = prepare_with(&dict, &input, dictionary_only(&dict, &j), vec![]);
        let req = writer_request(&m, &j, &prepared, 6)?;
        for dato in [
            "Persona de Ejemplo",
            "ejemplo@correo.com",
            "0000 0000",
            "Ciudad de México",
            "portafolio",
        ] {
            assert!(!req.contains(dato), "se filtró «{dato}»");
        }
        assert!(req.contains("\"b_01\"") && req.contains("Editor de video"));
        Ok(())
    }

    #[tokio::test]
    async fn guardar_crea_archivos_version_y_cambia_estado() -> R {
        let dict = SkillDict::builtin()?;
        let db = Db::open_in_memory().await?;
        let p = db
            .create_profile("Editor", &[], RemotePref::SoloRemoto, &[])
            .await?;
        let (m, j) = (master()?, job());
        db.insert_job(&j).await?;
        let app = db.upsert_found(&j.id, &p.id, 70).await?;

        let input = OptimizeInput {
            master: &m,
            original_text: Some(CV_TXT),
            job: &j,
            today: TODAY,
        };
        let prepared = prepare(None, &dict, &input).await;
        let opt = apply_draft(&dict, &input, prepared, honest_draft())?;

        let dir = std::env::temp_dir().join(format!("finder-opt-{}", Db::new_id()));
        let mut choices = opt.choices.clone();
        choices.summary = false; // el usuario no quiere resumen
        let target = SaveTarget {
            profile_id: &p.id,
            application_id: &app,
            out_dir: &dir,
        };
        let v = save(&db, &dict, &input, &opt, &choices, &target).await?;

        assert!(v.pdf_path.as_deref().is_some_and(|p| Path::new(p).exists()));
        assert!(
            v.docx_path
                .as_deref()
                .is_some_and(|p| Path::new(p).exists())
        );
        // Sin el resumen (que traía "edición de video") el puntaje baja respecto a aceptarlo todo:
        // lo guardado refleja la decisión del usuario, no la propuesta.
        assert!(
            v.ats_after < Some(opt.ats_after.total),
            "{:?} vs {}",
            v.ats_after,
            opt.ats_after.total
        );
        assert!(v.ats_after >= v.ats_before);
        assert!(
            !v.content_json.contains("\"summary\":\"Editor"),
            "se respetó la decisión del usuario"
        );
        let a = db.get_application(&app).await?.ok_or("falta")?;
        assert_eq!(
            (a.status, a.cv_version_id),
            (Status::CvAdaptado, Some(v.id.clone()))
        );
        assert_eq!(v.master_hash, master_hash(&m)?);
        std::fs::remove_dir_all(&dir)?;
        Ok(())
    }

    #[test]
    fn esquema_del_borrador_coincide_con_el_tipo() -> std::result::Result<(), serde_json::Error> {
        let v = serde_json::to_value(honest_draft())?;
        let back: Draft = serde_json::from_value(v)?;
        assert_eq!(back, honest_draft());
        assert_eq!(
            draft_schema()["required"],
            json!(["summary", "bullets", "skills_order"])
        );
        Ok(())
    }

    /// Con la configuración real (.env / variables). Correr a mano:
    /// `cargo test -p finder-infra optimizer -- --ignored --nocapture`
    #[tokio::test]
    #[ignore]
    async fn en_vivo_optimiza_con_el_redactor_configurado() -> R {
        let cfg = crate::llm::LlmConfig::from_env()?;
        let local = LlmClient::new(cfg.local.clone())?;
        let writer = LlmClient::new(cfg.writer.clone())?;
        let dict = SkillDict::builtin()?;
        let (m, j) = (master()?, job());
        let input = OptimizeInput {
            master: &m,
            original_text: Some(CV_TXT),
            job: &j,
            today: TODAY,
        };
        let opt = run(Some(&local), &writer, &dict, &input).await?;
        println!(
            "ATS {} → {}",
            opt.prepared.ats_before.total, opt.ats_after.total
        );
        for c in &opt.review.bullets {
            println!(
                "[{}] {}\n    {:?}",
                if c.accepted() { "OK" } else { "RECHAZADO" },
                c.proposed,
                c.issues
            );
        }
        println!("{}", opt.document.to_plain_text());
        Ok(())
    }
}
