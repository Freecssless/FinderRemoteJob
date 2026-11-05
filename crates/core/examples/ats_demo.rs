//! Prueba rápida del puntaje ATS con tus propios archivos:
//!
//! cargo run -p finder-core --example ats_demo -- vacante.txt cv.txt "Título del puesto"

use finder_core::ats::keywords::{SkillDict, extract_terms};
use finder_core::ats::score::ats_score;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [jd_path, cv_path, title] = args.as_slice() else {
        return Err("uso: ats_demo <vacante.txt> <cv.txt> \"<título del puesto>\"".into());
    };
    let jd = std::fs::read_to_string(jd_path)?;
    let cv = std::fs::read_to_string(cv_path)?;

    let dict = SkillDict::builtin()?;
    let terms = extract_terms(&dict, title, &jd);
    println!("Términos de la vacante:");
    for t in &terms {
        let tipo = if t.required { "requerido" } else { "deseable " };
        println!("  [{tipo}] {:<28} peso {:.1}", t.surface, t.weight);
    }

    let r = ats_score(&dict, &cv, title, &terms);
    let b = &r.breakdown;
    println!(
        "\nPuntaje ATS: {}/100  (palabras clave {:.1}/60 · título {:.1}/10 · secciones {:.1}/10 · legibilidad {:.1}/20)",
        r.total, b.keywords, b.title, b.sections, b.legibility
    );
    println!("Exactas: {}", r.matched.join(", "));
    println!(
        "Faltan: {}",
        r.missing
            .iter()
            .map(|t| t.surface.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!("\nSugerencias:");
    for s in &r.suggestions {
        println!("  • {s}");
    }
    Ok(())
}
