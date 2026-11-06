//! Carta de presentación (plan §4.6): el redactor propone 3 párrafos (por qué la empresa y el
//! puesto, 2 o 3 logros, cierre), cada uno citando de qué parte del CV sale, y el guardián los
//! revisa con las mismas reglas que el resumen. Los rechazados no entran en la carta.
//!
//! El saludo y la firma se ponen aquí: tu nombre nunca se envía al redactor.

use finder_core::ats::keywords::SkillDict;
use finder_core::document::DocLang;
use finder_core::guard::{Checked, DraftText, Issue, validate_paragraphs};
use finder_core::matching::years_of_experience;
use finder_core::model::{CvMaster, Job};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::llm::LlmClient;
use crate::optimizer::{Result, TokenEstimate, for_writer, job_lang, tokens_for};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LetterDraft {
    pub paragraphs: Vec<DraftText>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Letter {
    /// Cada párrafo propuesto con su revisión (aceptado o rechazado con motivo).
    pub paragraphs: Vec<Checked>,
    /// Carta lista para editar: saludo, párrafos aceptados y firma.
    pub text: String,
}

pub async fn write(
    writer: &LlmClient,
    dict: &SkillDict,
    master: &CvMaster,
    job: &Job,
    today: &str,
) -> Result<Letter> {
    let lang = job_lang(job);
    let years = years_of_experience(master, today).floor() as u32;
    let ask = async |forbidden: &[String]| -> Result<LetterDraft> {
        let user = request(master, job, years, forbidden)?;
        let value = writer
            .complete_json(&system(lang), &user, &schema())
            .await?;
        Ok(serde_json::from_value(value)?)
    };
    let mut draft = ask(&[]).await?;
    let first = assemble(dict, master, job, today, lang, &draft);
    let forbidden = forbidden_terms(&first.paragraphs);
    if forbidden.is_empty() {
        return Ok(first);
    }
    // Segunda vuelta (los modelos locales cuelan habilidades blandas): se reemplaza cada párrafo
    // rechazado por el de la misma posición si este sí pasa el guardián.
    let retry = ask(&forbidden).await?;
    let checked = validate_paragraphs(dict, master, &retry.paragraphs, today);
    for ((p, c), new) in draft
        .paragraphs
        .iter_mut()
        .zip(&first.paragraphs)
        .zip(retry.paragraphs.into_iter().zip(checked))
    {
        if !c.accepted() && new.1.accepted() {
            *p = new.0;
        }
    }
    Ok(assemble(dict, master, job, today, lang, &draft))
}

/// Habilidades y cifras por las que se rechazó algún párrafo.
fn forbidden_terms(paragraphs: &[Checked]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for issue in paragraphs.iter().flat_map(|p| &p.issues) {
        if let Issue::NewSkill(t) | Issue::NewNumber(t) = issue
            && !out.contains(t)
        {
            out.push(t.clone());
        }
    }
    out
}

/// Guardián + saludo y firma (sin IA; se prueba con borradores escritos a mano).
pub fn assemble(
    dict: &SkillDict,
    master: &CvMaster,
    job: &Job,
    today: &str,
    lang: DocLang,
    draft: &LetterDraft,
) -> Letter {
    let paragraphs = validate_paragraphs(dict, master, &draft.paragraphs, today);
    let company = job.company.as_deref().filter(|c| !c.trim().is_empty());
    let (greeting, closing) = match (lang, company) {
        (DocLang::Es, Some(c)) => (format!("Estimado equipo de {c}:"), "Atentamente,"),
        (DocLang::Es, None) => ("Estimado equipo de selección:".into(), "Atentamente,"),
        (DocLang::En, Some(c)) => (format!("Dear {c} team,"), "Sincerely,"),
        (DocLang::En, None) => ("Dear hiring team,".into(), "Sincerely,"),
    };
    let mut parts = vec![greeting];
    parts.extend(
        paragraphs
            .iter()
            .filter(|p| p.accepted())
            .map(|p| p.proposed.trim().to_string()),
    );
    parts.push(format!("{closing}\n{}", master.basics.name.trim()));
    Letter {
        text: parts.join("\n\n"),
        paragraphs,
    }
}

fn system(lang: DocLang) -> String {
    let idioma = match lang {
        DocLang::Es => "español",
        DocLang::En => "inglés",
    };
    format!(
        "Escribes cartas de presentación breves y concretas a partir de un CV maestro y una vacante.
Reglas obligatorias:
1. Exactamente 3 párrafos en \"paragraphs\", sin saludo ni firma (se agregan aparte):
   - Párrafo 1: por qué te interesa este puesto en esta empresa, conectando lo que pide la vacante con tu experiencia.
   - Párrafo 2: 2 o 3 logros del CV que más se relacionan con la vacante.
   - Párrafo 3: cierre breve con disponibilidad para conversar.
2. Solo puedes usar hechos del CV maestro. Nunca agregues experiencia, habilidades, herramientas, cifras, empresas ni logros que no estén en él. No digas que dominas algo que la vacante pide si el CV no lo menciona.
   Las únicas habilidades y áreas que puedes nombrar son las de \"habilidades_que_puedes_nombrar\" y las palabras textuales de los logros. No generalices ni renombres (si el logro dice \"corrección de color y sonido\", no escribas \"edición de audio\"; si dice \"YouTube\", no escribas \"redes sociales\").
3. Cada párrafo cita en \"source_ids\" los ids del CV (b_xx, sk_xx o exp_xx) que lo respaldan; el de cierre puede citar exp_xx de tu experiencia más reciente.
4. Conserva cada cifra exactamente como aparece en el CV. No uses cifras del anuncio. Si mencionas años de experiencia, usa exactamente los años indicados.
5. Usa la forma exacta de los términos de la vacante cuando el CV los respalde.
6. Tono profesional y cercano, en primera persona, sin frases genéricas ni exageraciones. Máximo 90 palabras por párrafo.
7. Nunca escribas los términos de \"no_menciones_porque_no_estan_en_el_cv\", ni siquiera para describir la vacante o la empresa.
8. Escribe todo en {idioma}."
    )
}

/// Igual que con el CV: sin nombre, correo, teléfono, ubicación ni enlaces.
fn request(master: &CvMaster, job: &Job, years: u32, forbidden: &[String]) -> Result<String> {
    Ok(serde_json::to_string(&json!({
        "no_menciones_porque_no_estan_en_el_cv": forbidden,
        "vacante": { "puesto": job.title, "empresa": job.company, "descripcion": job.description },
        "anos_de_experiencia": years,
        "habilidades_que_puedes_nombrar": master.skills.iter().map(|s| &s.name).collect::<Vec<_>>(),
        "cv_maestro": for_writer(master),
    }))?)
}

/// Tokens aproximados de una vuelta; si el guardián rechaza algún párrafo hay una segunda.
pub fn estimate_tokens(master: &CvMaster, job: &Job) -> Result<TokenEstimate> {
    let chars = system(job_lang(job)).len()
        + request(master, job, 0, &[])?.len()
        + schema().to_string().len();
    Ok(tokens_for(chars, 600))
}

pub fn schema() -> Value {
    json!({
        "type": "object", "additionalProperties": false, "required": ["paragraphs"],
        "properties": {
            "paragraphs": {"type": "array", "minItems": 3, "maxItems": 3, "items": {
                "type": "object", "additionalProperties": false, "required": ["text", "source_ids"],
                "properties": {"text": {"type": "string"}, "source_ids": {"type": "array", "items": {"type": "string"}}}
            }}
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use finder_core::model::dedupe_key;

    type R = std::result::Result<(), Box<dyn std::error::Error>>;

    fn master() -> std::result::Result<CvMaster, serde_json::Error> {
        serde_json::from_str(include_str!(
            "../../core/tests/fixtures/cv_editor_video.json"
        ))
    }

    fn job(company: Option<&str>) -> Job {
        let title = "Editor de Video";
        Job {
            id: "j1".into(),
            source_id: "manual".into(),
            external_id: None,
            url: "https://example.com".into(),
            company: company.map(String::from),
            title: title.into(),
            location: None,
            remote_type: None,
            salary_min: None,
            salary_max: None,
            currency: None,
            description: "Buscamos editor de video con Premiere para nuestro canal.".into(),
            language: Some("es".into()),
            requirements: None,
            posted_at: None,
            found_at: "2026-11-05T00:00:00Z".into(),
            dedupe_hash: dedupe_key(company, title, None),
        }
    }

    fn p(text: &str, src: &[&str]) -> DraftText {
        DraftText {
            text: text.into(),
            source_ids: src.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn la_carta_solo_lleva_parrafos_aprobados_con_saludo_y_firma() -> R {
        let (dict, m) = (SkillDict::builtin()?, master()?);
        let draft = LetterDraft {
            paragraphs: vec![
                p("Me interesa editar los videos de su canal.", &["exp_01"]),
                p("Edité 900 videos con Final Cut Pro.", &["b_01"]),
                p(
                    "Edité 120 videos para YouTube con Premiere y After Effects.",
                    &["b_01"],
                ),
            ],
        };
        let l = assemble(
            &dict,
            &m,
            &job(Some("Agencia X")),
            "2026-11",
            DocLang::Es,
            &draft,
        );
        assert!(
            l.paragraphs[1]
                .issues
                .contains(&Issue::NewNumber("900".into()))
        );
        assert!(
            l.text.starts_with("Estimado equipo de Agencia X:"),
            "{}",
            l.text
        );
        assert!(
            l.text
                .ends_with(&format!("Atentamente,\n{}", m.basics.name)),
            "{}",
            l.text
        );
        assert!(
            l.text.contains("120 videos") && !l.text.contains("900"),
            "{}",
            l.text
        );
        Ok(())
    }

    #[test]
    fn el_redactor_no_ve_datos_personales() -> R {
        let m = master()?;
        let req = request(&m, &job(None), 6, &[])?;
        for dato in [&m.basics.name, &m.basics.email, &m.basics.phone] {
            assert!(dato.is_empty() || !req.contains(dato.as_str()), "{dato}");
        }
        Ok(())
    }

    #[test]
    fn el_estimado_crece_con_la_vacante() -> R {
        let m = master()?;
        let corta = estimate_tokens(&m, &job(None))?;
        let mut j = job(None);
        j.description = "x".repeat(3000);
        let larga = estimate_tokens(&m, &j)?;
        assert!(corta.input > 500, "{corta:?}");
        assert!(larga.input >= corta.input + 950, "{corta:?} {larga:?}");
        assert!(crate::optimizer::estimate_tokens(&m, &j)?.input > larga.input / 2);
        Ok(())
    }

    /// Con la configuración real (.env / variables). Correr a mano:
    /// `cargo test -p finder-infra letter -- --ignored --nocapture`
    #[tokio::test]
    #[ignore]
    async fn en_vivo_carta_con_el_redactor_configurado() -> R {
        let cfg = crate::llm::LlmConfig::from_env()?;
        let writer = LlmClient::new(cfg.writer)?;
        let (dict, m) = (SkillDict::builtin()?, master()?);
        let mut j = job(Some("Agencia X"));
        j.description = include_str!("../../core/tests/fixtures/jd_editor_video_es.txt").into();
        let l = write(&writer, &dict, &m, &j, "2026-11").await?;
        for p in &l.paragraphs {
            let estado = if p.accepted() { "OK" } else { "RECHAZADO" };
            println!("[{estado}] {}\n    {:?}", p.proposed, p.issues);
        }
        println!("\n{}", l.text);
        Ok(())
    }
}
