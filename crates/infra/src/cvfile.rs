//! Texto de un CV en PDF o DOCX, para que la IA lo convierta al CV maestro.

use std::panic::{AssertUnwindSafe, catch_unwind};

use docx_rs::{
    DocumentChild, ParagraphChild, RunChild, TableCellContent, TableChild, TableRowChild,
};

#[derive(Debug, thiserror::Error)]
pub enum CvFileError {
    #[error("formato no soportado: usa un archivo .pdf o .docx")]
    Unsupported,
    #[error("no se pudo leer el PDF: {0}")]
    Pdf(String),
    #[error("no se pudo leer el DOCX: {0}")]
    Docx(String),
    #[error("el archivo no tiene texto (¿es un PDF escaneado como imagen?)")]
    Empty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CvFileKind {
    Pdf,
    Docx,
}

impl CvFileKind {
    /// Por la extensión del nombre del archivo.
    pub fn from_name(name: &str) -> Option<Self> {
        let ext = name.rsplit_once('.')?.1.to_ascii_lowercase();
        match ext.as_str() {
            "pdf" => Some(Self::Pdf),
            "docx" => Some(Self::Docx),
            _ => None,
        }
    }
}

/// Extrae el texto del archivo. Devuelve error si no hay texto útil.
pub fn extract_text(bytes: &[u8], kind: CvFileKind) -> Result<String, CvFileError> {
    let text = match kind {
        CvFileKind::Pdf => {
            // `pdf-extract` puede entrar en pánico con PDFs dañados o raros: se contiene aquí
            // para que un archivo malo no tumbe la app.
            catch_unwind(AssertUnwindSafe(|| {
                pdf_extract::extract_text_from_mem(bytes)
            }))
            .map_err(|_| {
                CvFileError::Pdf("el archivo está dañado o usa un formato no soportado".into())
            })?
            .map_err(|e| CvFileError::Pdf(e.to_string()))?
        }
        CvFileKind::Docx => docx_text(bytes)?,
    };
    let text = tidy(&text);
    if text.chars().filter(|c| c.is_alphanumeric()).count() < 20 {
        return Err(CvFileError::Empty);
    }
    Ok(text)
}

fn docx_text(bytes: &[u8]) -> Result<String, CvFileError> {
    let docx = docx_rs::read_docx(bytes).map_err(|e| CvFileError::Docx(e.to_string()))?;
    let mut out = String::new();
    for child in &docx.document.children {
        match child {
            DocumentChild::Paragraph(p) => paragraph(&p.children, &mut out),
            // Muchos CVs usan tablas para maquetar: se leen celda por celda.
            DocumentChild::Table(t) => table(t, &mut out),
            _ => {}
        }
    }
    Ok(out)
}

fn paragraph(children: &[ParagraphChild], out: &mut String) {
    runs(children, out);
    out.push('\n');
}

fn runs(children: &[ParagraphChild], out: &mut String) {
    for c in children {
        match c {
            ParagraphChild::Run(r) => run(&r.children, out),
            ParagraphChild::Hyperlink(h) => runs(&h.children, out),
            ParagraphChild::Insert(ins) => {
                for ic in &ins.children {
                    if let docx_rs::InsertChild::Run(r) = ic {
                        run(&r.children, out);
                    }
                }
            }
            _ => {}
        }
    }
}

fn run(children: &[RunChild], out: &mut String) {
    for c in children {
        match c {
            RunChild::Text(t) => out.push_str(&t.text),
            RunChild::Tab(_) => out.push(' '),
            RunChild::Break(_) | RunChild::CarriageReturn(_) => out.push('\n'),
            _ => {}
        }
    }
}

fn table(t: &docx_rs::Table, out: &mut String) {
    for TableChild::TableRow(row) in &t.rows {
        for TableRowChild::TableCell(cell) in &row.cells {
            for content in &cell.children {
                match content {
                    TableCellContent::Paragraph(p) => paragraph(&p.children, out),
                    TableCellContent::Table(inner) => table(inner, out),
                    _ => {}
                }
            }
        }
    }
}

/// Espacios simples por línea y sin líneas vacías repetidas.
fn tidy(text: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    for line in text.lines() {
        let line = line.split_whitespace().collect::<Vec<_>>().join(" ");
        if line.is_empty() && out.last().is_none_or(|l| l.is_empty()) {
            continue;
        }
        out.push(line);
    }
    out.join("\n").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use docx_rs::{Docx, Paragraph, Run, Table, TableCell, TableRow};

    #[test]
    fn tipo_por_extension() {
        assert_eq!(CvFileKind::from_name("Mi CV.PDF"), Some(CvFileKind::Pdf));
        assert_eq!(CvFileKind::from_name("cv.docx"), Some(CvFileKind::Docx));
        assert_eq!(CvFileKind::from_name("cv.doc"), None);
        assert_eq!(CvFileKind::from_name("sin_extension"), None);
    }

    #[test]
    fn lee_texto_de_pdf() -> Result<(), CvFileError> {
        let text = extract_text(
            include_bytes!("../tests/fixtures/cv_simple.pdf"),
            CvFileKind::Pdf,
        )?;
        assert!(text.contains("Persona de Ejemplo"), "{text}");
        assert!(text.contains("Edite 120 videos para YouTube"), "{text}");
        Ok(())
    }

    #[test]
    fn pdf_danado_da_error_sin_panico() {
        assert!(matches!(
            extract_text(b"%PDF-1.4 esto no es un pdf", CvFileKind::Pdf),
            Err(CvFileError::Pdf(_))
        ));
    }

    #[test]
    fn lee_texto_de_docx_con_tablas() -> Result<(), Box<dyn std::error::Error>> {
        let mut buf = std::io::Cursor::new(Vec::new());
        Docx::new()
            .add_paragraph(Paragraph::new().add_run(Run::new().add_text("Persona de Ejemplo")))
            .add_paragraph(Paragraph::new().add_run(Run::new().add_text("Editor de vídeo")))
            .add_table(Table::new(vec![TableRow::new(vec![
                TableCell::new()
                    .add_paragraph(Paragraph::new().add_run(Run::new().add_text("HABILIDADES"))),
                TableCell::new().add_paragraph(
                    Paragraph::new()
                        .add_run(Run::new().add_text("Adobe Premiere Pro, After Effects")),
                ),
            ])]))
            .build()
            .pack(&mut buf)?;
        let text = extract_text(buf.get_ref(), CvFileKind::Docx)?;
        assert_eq!(
            text,
            "Persona de Ejemplo\nEditor de vídeo\nHABILIDADES\nAdobe Premiere Pro, After Effects"
        );
        Ok(())
    }

    #[test]
    fn archivo_sin_texto() {
        assert!(matches!(
            extract_text(b"no es zip", CvFileKind::Docx),
            Err(CvFileError::Docx(_))
        ));
    }
}
