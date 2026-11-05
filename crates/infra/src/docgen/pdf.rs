use std::sync::LazyLock;

use finder_core::document::CvDocument;
use typst::foundations::{Dict, IntoValue};
use typst_as_lib::TypstEngine;
use typst_layout::PagedDocument;

use super::{DocError, Result};

const CLASICA: &str = include_str!("../../templates/clasica.typ");
const FONTS: [&[u8]; 3] = [
    include_bytes!("../../assets/fonts/LiberationSans-Regular.ttf"),
    include_bytes!("../../assets/fonts/LiberationSans-Bold.ttf"),
    include_bytes!("../../assets/fonts/LiberationSans-Italic.ttf"),
];

/// El motor (plantilla + fuentes ya cargadas) se construye una vez y se reutiliza.
static ENGINE: LazyLock<TypstEngine<typst_as_lib::TypstTemplateMainFile>> = LazyLock::new(|| {
    TypstEngine::builder()
        .main_file(CLASICA)
        .fonts(FONTS)
        .build()
});

/// PDF del CV con la plantilla Clásica.
pub fn render_pdf(doc: &CvDocument) -> Result<Vec<u8>> {
    let mut inputs = Dict::new();
    inputs.insert("data".into(), serde_json::to_string(doc)?.into_value());

    let compiled: PagedDocument = ENGINE
        .compile_with_input(inputs)
        .output
        .map_err(|e| DocError::Typst(e.to_string()))?;
    typst_pdf::pdf(&compiled, &Default::default()).map_err(|errs| {
        DocError::Typst(
            errs.iter()
                .map(|e| e.message.to_string())
                .collect::<Vec<_>>()
                .join("; "),
        )
    })
}
