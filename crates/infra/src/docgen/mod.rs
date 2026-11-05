//! Generación del CV en PDF (Typst) y DOCX (docx-rs), y prueba de ida y vuelta (plan §4.4 B, §4.6).
//!
//! La fuente (Liberation Sans, licencia OFL) va incluida en el binario: el PDF sale idéntico en
//! Linux, Windows y Android. El DOCX no incrusta fuentes, así que usa Arial (misma métrica).

mod docx;
mod pdf;
mod roundtrip;

pub use docx::render_docx;
pub use pdf::render_pdf;
pub use roundtrip::{RoundTrip, verify};

#[derive(Debug, thiserror::Error)]
pub enum DocError {
    #[error("error en la plantilla del PDF: {0}")]
    Typst(String),
    #[error("error al generar el DOCX: {0}")]
    Docx(String),
    #[error("no se pudo leer el archivo generado: {0}")]
    Read(#[from] crate::cvfile::CvFileError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, DocError>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cvfile::CvFileKind;
    use finder_core::ats::keywords::{SkillDict, extract_terms};
    use finder_core::ats::score::{ats_score, with_roundtrip};
    use finder_core::document::{CvDocument, DocLang};
    use finder_core::model::CvMaster;

    type R = std::result::Result<(), Box<dyn std::error::Error>>;

    fn doc(lang: DocLang) -> std::result::Result<CvDocument, serde_json::Error> {
        let m: CvMaster = serde_json::from_str(include_str!(
            "../../../core/tests/fixtures/cv_editor_video.json"
        ))?;
        let mut d = CvDocument::from_master(&m, lang);
        d.summary =
            Some("Editor de video con 6 años de experiencia en YouTube y comerciales.".into());
        Ok(d)
    }

    #[test]
    fn pdf_ida_y_vuelta_sin_perdidas() -> R {
        let d = doc(DocLang::Es)?;
        let pdf = render_pdf(&d)?;
        assert!(pdf.starts_with(b"%PDF"));
        let rt = verify(&d, &pdf, CvFileKind::Pdf)?;
        assert!(
            rt.ok(),
            "faltan {:?}; orden {}\n{}",
            rt.missing_words,
            rt.sections_in_order,
            rt.text
        );
        Ok(())
    }

    #[test]
    fn docx_ida_y_vuelta_sin_perdidas() -> R {
        for lang in [DocLang::Es, DocLang::En] {
            let d = doc(lang)?;
            let rt = verify(&d, &render_docx(&d)?, CvFileKind::Docx)?;
            assert!(
                rt.ok(),
                "{lang:?}: faltan {:?}; orden {}",
                rt.missing_words,
                rt.sections_in_order
            );
        }
        Ok(())
    }

    #[test]
    fn el_pdf_obtiene_puntaje_ats_completo_en_formato() -> R {
        let d = doc(DocLang::Es)?;
        let rt = verify(&d, &render_pdf(&d)?, CvFileKind::Pdf)?;
        let dict = SkillDict::builtin()?;
        let jd = include_str!("../../../core/tests/fixtures/jd_editor_video_es.txt");
        let title = "Editor de Video (Remoto - LATAM)";
        let r = with_roundtrip(
            ats_score(&dict, &rt.text, title, &extract_terms(&dict, title, jd)),
            rt.missing_ratio,
            rt.sections_in_order,
        );
        let b = &r.breakdown;
        assert_eq!(
            (b.title, b.sections, b.legibility),
            (10.0, 10.0, 20.0),
            "{:?}",
            r.suggestions
        );
        Ok(())
    }

    #[test]
    fn texto_con_simbolos_de_typst_se_imprime_literal() -> R {
        let mut d = doc(DocLang::Es)?;
        d.experience[0]
            .bullets
            .push("Campaña #1 con *50%* de alcance, costo $200 <beta> @marca".into());
        let rt = verify(&d, &render_pdf(&d)?, CvFileKind::Pdf)?;
        assert!(
            rt.text.contains("#1") && rt.text.contains("*50%*") && rt.text.contains("$200"),
            "{}",
            rt.text
        );
        assert!(
            rt.text.contains("<beta>") && rt.text.contains("@marca"),
            "{}",
            rt.text
        );
        Ok(())
    }

    #[test]
    fn detecta_perdidas_y_desorden() -> R {
        let d = doc(DocLang::Es)?;
        let pdf = render_pdf(&d)?;
        let mut otro = d.clone();
        otro.skills.push("Habilidad Inexistente".into());
        let rt = verify(&otro, &pdf, CvFileKind::Pdf)?;
        assert_eq!(rt.missing_words, ["habilidad", "inexistente"]);
        assert!(!rt.ok());
        Ok(())
    }

    /// Genera ejemplos para revisarlos a ojo:
    /// `cargo test -p finder-infra muestras -- --ignored` → /tmp/finder-muestras/
    #[test]
    #[ignore]
    fn muestras_para_revision_visual() -> R {
        let dir = std::env::temp_dir().join("finder-muestras");
        std::fs::create_dir_all(&dir)?;
        let d = doc(DocLang::Es)?;
        let stem = d.file_stem("Editor de Video", Some("Agencia X"));
        std::fs::write(dir.join(format!("{stem}.pdf")), render_pdf(&d)?)?;
        std::fs::write(dir.join(format!("{stem}.docx")), render_docx(&d)?)?;
        println!("{}", dir.display());
        Ok(())
    }
}
