//! Benchmarks de los procesos pesados. Correr en release:
//! `cargo test --release -p finder-infra --test rendimiento -- --ignored --nocapture`

use std::time::{Duration, Instant};

use finder_infra::core::ats::keywords::SkillDict;
use finder_infra::core::document::{CvDocument, DocLang};
use finder_infra::core::model::CvMaster;
use finder_infra::cvfile::{CvFileKind, extract_text};
use finder_infra::db::Db;
use finder_infra::docgen::{render_docx, render_pdf};
use finder_infra::llm::tasks::import_without_ai;
use finder_infra::sources::manual;

type R = Result<(), Box<dyn std::error::Error>>;

/// Mediana de `n` corridas (la primera calienta cachés y no cuenta).
fn bench<T>(name: &str, n: usize, limit: Duration, mut f: impl FnMut() -> T) -> Duration {
    f();
    let mut times: Vec<Duration> = (0..n)
        .map(|_| {
            let t = Instant::now();
            std::hint::black_box(f());
            t.elapsed()
        })
        .collect();
    times.sort();
    let median = times[n / 2];
    println!(
        "{name:<38} mediana {median:>10.2?}   peor {:>10.2?}",
        times[n - 1]
    );
    assert!(median < limit, "{name}: {median:?} supera {limit:?}");
    median
}

fn doc() -> Result<CvDocument, serde_json::Error> {
    let m: CvMaster = serde_json::from_str(include_str!(
        "../../core/tests/fixtures/cv_editor_video.json"
    ))?;
    Ok(CvDocument::from_master(&m, DocLang::Es))
}

#[test]
#[ignore]
fn banco_documentos_y_lectura() -> R {
    let d = doc()?;
    let pdf = render_pdf(&d)?;
    let docx = render_docx(&d)?;
    bench(
        "generar PDF (Typst)",
        15,
        Duration::from_millis(500),
        || render_pdf(&d),
    );
    bench("generar DOCX", 15, Duration::from_millis(100), || {
        render_docx(&d)
    });
    bench("leer texto de PDF", 15, Duration::from_millis(500), || {
        extract_text(&pdf, CvFileKind::Pdf)
    });
    bench("leer texto de DOCX", 15, Duration::from_millis(100), || {
        extract_text(&docx, CvFileKind::Docx)
    });

    let dict = SkillDict::builtin()?;
    let cv = include_str!("../../core/tests/fixtures/cv_realista.txt");
    bench(
        "importar CV sin IA (normal)",
        15,
        Duration::from_millis(50),
        || import_without_ai(&dict, cv.to_string()),
    );
    // CV absurdo (~550 KB de texto): no debe congelar la app.
    let huge = cv.repeat(300);
    println!("  (CV enorme: {} KB)", huge.len() / 1024);
    bench(
        "importar CV sin IA (300× más largo)",
        3,
        Duration::from_secs(5),
        || import_without_ai(&dict, huge.clone()),
    );
    Ok(())
}

#[tokio::test]
#[ignore]
async fn banco_base_de_datos() -> R {
    let dir = std::env::temp_dir().join(format!("finder-banco-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir)?;
    let db = Db::open(&dir.join("banco.db")).await?;
    let body = include_str!("../../core/tests/fixtures/jd_editor_video_es.txt");

    let n = 5000;
    let t = Instant::now();
    for i in 0..n {
        let raw = manual::from_text(
            &format!("Editor de video {i}"),
            Some("Empresa"),
            Some(&format!("https://ejemplo.com/vacante/{i}")),
            body,
        );
        manual::save(&db, raw).await?;
    }
    let insert = t.elapsed();
    println!(
        "{:<38} {:>10.2?}  ({:.2?} c/u)",
        format!("guardar {n} vacantes"),
        insert,
        insert / n
    );
    assert!(insert / n < Duration::from_millis(20));

    for limit in [500, 5000] {
        let t = Instant::now();
        let jobs = db.list_jobs(limit).await?;
        let e = t.elapsed();
        println!(
            "{:<38} {:>10.2?}",
            format!("listar {} vacantes", jobs.len()),
            e
        );
        assert!(e < Duration::from_secs(1));
    }
    let id = db.list_jobs(1).await?.remove(0).id;
    let t = Instant::now();
    for _ in 0..100 {
        db.get_job(&id).await?;
    }
    println!("{:<38} {:>10.2?}", "abrir una vacante (×100)", t.elapsed());
    let size = std::fs::metadata(dir.join("banco.db"))?.len()
        + std::fs::metadata(dir.join("banco.db-wal")).map_or(0, |m| m.len());
    println!("  (base con {n} vacantes: {} MB)", size >> 20);
    std::fs::remove_dir_all(&dir)?;
    Ok(())
}
