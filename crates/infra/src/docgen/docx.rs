use docx_rs::{
    AbstractNumbering, Docx, IndentLevel, Level, LevelJc, LevelText, LineSpacing, NumberFormat,
    Numbering, NumberingId, PageMargin, Paragraph, Run, RunFonts, SpecialIndentType, Start, Style,
    StyleType, Tab, TabValueType,
};
use finder_core::document::{CvDocument, Section, Template};

use super::{DocError, Result};

/// Estilos nativos de Word ("Heading 1", lista con viñetas reales): los ATS los entienden mejor
/// que texto con formato manual. Medidas en medios puntos (tamaño) y twips (sangría, espacios).
const HEADING: &str = "Heading1";
const BULLETS: usize = 1;
/// Mismos colores que la plantilla del PDF y la vista previa de la app.
const ACCENT: &str = "1F50A0";
const SOFT: &str = "4A4E58";

/// DOCX del CV con la misma estructura que el PDF.
pub fn render_docx(doc: &CvDocument) -> Result<Vec<u8>> {
    let compact = doc.template == Template::Compacta;
    // Márgenes del PDF en twips (1,9 × 1,7 cm; compacta 1,5 × 1,3 cm).
    let (margin_x, margin_y) = if compact { (850, 737) } else { (1077, 964) };
    let right_edge = (12_240 - 2 * margin_x) as usize;
    let fonts = RunFonts::new()
        .ascii("Arial")
        .hi_ansi("Arial")
        .cs("Arial")
        .east_asia("Arial");
    let mut d = Docx::new()
        .default_fonts(fonts)
        .default_size(if compact { 19 } else { 21 }) // 9.5 / 10.5 pt
        .page_size(12_240, 15_840) // carta, igual que el PDF
        .page_margin(
            PageMargin::new()
                .top(margin_y)
                .bottom(margin_y)
                .left(margin_x)
                .right(margin_x),
        )
        .add_style(
            Style::new(HEADING, StyleType::Paragraph)
                .name("Heading 1")
                .size(if compact { 20 } else { 22 })
                .bold()
                .color(ACCENT)
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
            .add_run(Run::new().add_text(t))
    };
    let bullet = |t: &str| text(t).numbering(NumberingId::new(BULLETS), IndentLevel::new(0));

    let name_size = if compact { 38 } else { 42 };
    d = d.add_paragraph(
        Paragraph::new().add_run(Run::new().add_text(&doc.name).bold().size(name_size)),
    );
    if !doc.headline.is_empty() {
        d = d.add_paragraph(
            Paragraph::new().add_run(
                Run::new()
                    .add_text(&doc.headline)
                    .bold()
                    .size(name_size / 2 + 3)
                    .color(ACCENT),
            ),
        );
    }
    if !doc.contact.is_empty() {
        d = d.add_paragraph(
            Paragraph::new().add_run(Run::new().add_text(doc.contact.join("  ·  ")).color(SOFT)),
        );
    }
    // Título a la izquierda y fechas a la derecha (tabulación), y el lugar en gris debajo.
    let entry = |d: Docx, title: &str, dates: &str, place: &str| {
        let d = d.add_paragraph(
            Paragraph::new()
                .add_tab(Tab::new().val(TabValueType::Right).pos(right_edge))
                .line_spacing(
                    LineSpacing::new()
                        .before(if compact { 100 } else { 160 })
                        .after(0),
                )
                .add_run(Run::new().add_text(title).bold())
                .add_run(Run::new().add_tab().add_text(dates).color(SOFT)),
        );
        if place.is_empty() {
            d
        } else {
            d.add_paragraph(Paragraph::new().add_run(Run::new().add_text(place).color(SOFT)))
        }
    };

    for sec in doc.sections() {
        d = d.add_paragraph(heading(doc.heading(sec)));
        let bullets =
            |d: Docx, items: &[String]| items.iter().fold(d, |d, b| d.add_paragraph(bullet(b)));
        d = match sec {
            Section::Summary => d.add_paragraph(text(doc.summary.as_deref().unwrap_or_default())),
            Section::Experience => doc.experience.iter().fold(d, |d, e| {
                let d = entry(d, &e.title, &e.dates, &e.company);
                bullets(d, &e.bullets)
            }),
            Section::Education => doc
                .education
                .iter()
                .fold(d, |d, e| entry(d, &e.degree, &e.dates, &e.institution)),
            Section::Skills => d.add_paragraph(text(&doc.skills.join("  ·  "))),
            Section::Certifications => bullets(d, &doc.certifications),
            Section::Languages => bullets(d, &doc.languages),
        };
    }

    let mut buf = std::io::Cursor::new(Vec::new());
    d.build()
        .pack(&mut buf)
        .map_err(|e| DocError::Docx(e.to_string()))?;
    Ok(buf.into_inner())
}
