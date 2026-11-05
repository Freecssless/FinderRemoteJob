use docx_rs::{
    AbstractNumbering, Docx, IndentLevel, Level, LevelJc, LevelText, LineSpacing, NumberFormat,
    Numbering, NumberingId, Paragraph, Run, RunFonts, SpecialIndentType, Start, Style, StyleType,
};
use finder_core::document::CvDocument;

use super::{DocError, Result};

/// Estilos nativos de Word ("Heading 1", lista con viñetas reales): los ATS los entienden mejor
/// que texto con formato manual. Medidas en medios puntos (tamaño) y twips (sangría, espacios).
const HEADING: &str = "Heading1";
const BULLETS: usize = 1;

/// DOCX del CV con la misma estructura que el PDF.
pub fn render_docx(doc: &CvDocument) -> Result<Vec<u8>> {
    let fonts = RunFonts::new()
        .ascii("Arial")
        .hi_ansi("Arial")
        .cs("Arial")
        .east_asia("Arial");
    let mut d = Docx::new()
        .default_fonts(fonts)
        .default_size(21) // 10.5 pt
        .page_size(12_240, 15_840) // carta, igual que el PDF
        .add_style(
            Style::new(HEADING, StyleType::Paragraph)
                .name("Heading 1")
                .size(23)
                .bold()
                .outline_lvl(0)
                .line_spacing(LineSpacing::new().before(240).after(80)),
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

    let h = &doc.headings;
    if let Some(s) = &doc.summary {
        d = d.add_paragraph(heading(h.summary)).add_paragraph(text(s));
    }
    if !doc.experience.is_empty() {
        d = d.add_paragraph(heading(h.experience));
        for e in &doc.experience {
            d = d
                .add_paragraph(
                    Paragraph::new()
                        .add_run(Run::new().add_text(&e.title).bold())
                        .add_run(Run::new().add_text(format!(" — {}", e.company))),
                )
                .add_paragraph(Paragraph::new().add_run(Run::new().add_text(&e.dates).italic()));
            for b in &e.bullets {
                d = d.add_paragraph(bullet(b));
            }
        }
    }
    if !doc.education.is_empty() {
        d = d.add_paragraph(heading(h.education));
        for e in &doc.education {
            let dates = if e.dates.is_empty() {
                String::new()
            } else {
                format!(", {}", e.dates)
            };
            d = d.add_paragraph(
                Paragraph::new()
                    .add_run(Run::new().add_text(&e.degree).bold())
                    .add_run(Run::new().add_text(format!(" — {}{dates}", e.institution))),
            );
        }
    }
    for (title, items) in [
        (h.skills, &doc.skills),
        (h.certifications, &doc.certifications),
        (h.languages, &doc.languages),
    ] {
        if !items.is_empty() {
            d = d
                .add_paragraph(heading(title))
                .add_paragraph(text(&items.join(", ")));
        }
    }

    let mut buf = std::io::Cursor::new(Vec::new());
    d.build()
        .pack(&mut buf)
        .map_err(|e| DocError::Docx(e.to_string()))?;
    Ok(buf.into_inner())
}
