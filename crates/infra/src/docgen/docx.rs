use docx_rs::{
    AbstractNumbering, Docx, IndentLevel, Level, LevelJc, LevelText, LineSpacing, NumberFormat,
    Numbering, NumberingId, Paragraph, Run, RunFonts, SpecialIndentType, Start, Style, StyleType,
};
use finder_core::document::{CvDocument, Section, Template};

use super::{DocError, Result};

/// Estilos nativos de Word ("Heading 1", lista con viñetas reales): los ATS los entienden mejor
/// que texto con formato manual. Medidas en medios puntos (tamaño) y twips (sangría, espacios).
const HEADING: &str = "Heading1";
const BULLETS: usize = 1;

/// DOCX del CV con la misma estructura que el PDF.
pub fn render_docx(doc: &CvDocument) -> Result<Vec<u8>> {
    let compact = doc.template == Template::Compacta;
    let fonts = RunFonts::new()
        .ascii("Arial")
        .hi_ansi("Arial")
        .cs("Arial")
        .east_asia("Arial");
    let mut d = Docx::new()
        .default_fonts(fonts)
        .default_size(if compact { 19 } else { 21 }) // 9.5 / 10.5 pt
        .page_size(12_240, 15_840) // carta, igual que el PDF
        .add_style(
            Style::new(HEADING, StyleType::Paragraph)
                .name("Heading 1")
                .size(23)
                .bold()
                .outline_lvl(0)
                .line_spacing(if compact {
                    LineSpacing::new().before(160).after(40)
                } else {
                    LineSpacing::new().before(240).after(80)
                }),
        )
        .add_abstract_numbering(
            AbstractNumbering::new(BULLETS).add_level(
                Level::new(
                    0,
                    Start::new(1),
                    NumberFormat::new("bullet"),
                    LevelText::new("•"),
                    LevelJc::new("left"),
                )
                .indent(
                    Some(360),
                    Some(SpecialIndentType::Hanging(240)),
                    None,
                    None,
                ),
            ),
        )
        .add_numbering(Numbering::new(BULLETS, BULLETS));

    let text = |t: &str| Paragraph::new().add_run(Run::new().add_text(t));
    let heading = |t: &str| {
        Paragraph::new()
            .style(HEADING)
            .add_run(Run::new().add_text(t.to_uppercase()))
    };
    let bullet = |t: &str| text(t).numbering(NumberingId::new(BULLETS), IndentLevel::new(0));

    d = d.add_paragraph(Paragraph::new().add_run(Run::new().add_text(&doc.name).bold().size(36)));
    if !doc.headline.is_empty() {
        d = d.add_paragraph(Paragraph::new().add_run(Run::new().add_text(&doc.headline).size(22)));
    }
    d = d.add_paragraph(text(&doc.contact.join(" | ")));

    for sec in doc.sections() {
        d = d.add_paragraph(heading(doc.heading(sec)));
        let list = |items: &[String]| text(&items.join(", "));
        d = match sec {
            Section::Summary => d.add_paragraph(text(doc.summary.as_deref().unwrap_or_default())),
            Section::Experience => doc.experience.iter().fold(d, |d, e| {
                let d = d
                    .add_paragraph(
                        Paragraph::new()
                            .add_run(Run::new().add_text(&e.title).bold())
                            .add_run(Run::new().add_text(format!(" — {}", e.company))),
                    )
                    .add_paragraph(
                        Paragraph::new().add_run(Run::new().add_text(&e.dates).italic()),
                    );
                e.bullets.iter().fold(d, |d, b| d.add_paragraph(bullet(b)))
            }),
            Section::Education => doc.education.iter().fold(d, |d, e| {
                let dates = if e.dates.is_empty() {
                    String::new()
                } else {
                    format!(", {}", e.dates)
                };
                d.add_paragraph(
                    Paragraph::new()
                        .add_run(Run::new().add_text(&e.degree).bold())
                        .add_run(Run::new().add_text(format!(" — {}{dates}", e.institution))),
                )
            }),
            Section::Skills => d.add_paragraph(list(&doc.skills)),
            Section::Certifications => d.add_paragraph(list(&doc.certifications)),
            Section::Languages => d.add_paragraph(list(&doc.languages)),
        };
    }

    let mut buf = std::io::Cursor::new(Vec::new());
    d.build()
        .pack(&mut buf)
        .map_err(|e| DocError::Docx(e.to_string()))?;
    Ok(buf.into_inner())
}
