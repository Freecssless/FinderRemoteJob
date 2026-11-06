//! Prueba de ida y vuelta (plan §4.4 B): se lee el archivo generado como lo haría un ATS y se
//! comprueba que el texto esté completo y las secciones en orden.

use std::collections::HashSet;

use finder_core::ats::keywords::normalize;
use finder_core::document::CvDocument;
use serde::Serialize;

use super::Result;
use crate::cvfile::{CvFileKind, extract_text};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RoundTrip {
    /// Texto tal como se recupera del archivo (sirve para el puntaje ATS "después").
    pub text: String,
    /// Palabras del CV que no se recuperaron.
    pub missing_words: Vec<String>,
    /// Fracción de palabras perdidas (0.0 = nada perdido).
    pub missing_ratio: f32,
    /// ¿Los encabezados aparecen todos y en el orden esperado?
    pub sections_in_order: bool,
}

impl RoundTrip {
    pub fn ok(&self) -> bool {
        self.missing_words.is_empty() && self.sections_in_order
    }
}

pub fn verify(doc: &CvDocument, file: &[u8], kind: CvFileKind) -> Result<RoundTrip> {
    let text = extract_text(file, kind)?;
    let got_norm = normalize(&text);
    // El lector de PDF a veces une palabras al cambiar de línea; se compara por palabras sueltas.
    let got: HashSet<&str> = got_norm
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect();

    let expected_norm = normalize(&doc.to_plain_text());
    let mut seen = HashSet::new();
    let expected: Vec<&str> = expected_norm
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 3 && seen.insert(*w))
        .collect();
    // pdf-extract también pega líneas sin espacio ("VillegasEditor"; poppler las lee bien):
    // una palabra pegada a otra palabra esperada no se perdió.
    let glued = |w: &str| {
        got.iter().any(|t| {
            t.strip_prefix(w)
                .or_else(|| t.strip_suffix(w))
                .is_some_and(|rest| seen.contains(rest))
        })
    };
    let missing_words: Vec<String> = expected
        .iter()
        .filter(|w| !got.contains(*w) && !glued(w))
        .map(|w| w.to_string())
        .collect();
    let missing_ratio = if expected.is_empty() {
        0.0
    } else {
        missing_words.len() as f32 / expected.len() as f32
    };

    let mut cursor = 0;
    let sections_in_order = doc.section_headings().iter().all(|h| {
        let h = normalize(h);
        match got_norm[cursor..].find(&h) {
            Some(i) => {
                cursor += i + h.len();
                true
            }
            None => false,
        }
    });

    Ok(RoundTrip {
        text,
        missing_words,
        missing_ratio,
        sections_in_order,
    })
}
