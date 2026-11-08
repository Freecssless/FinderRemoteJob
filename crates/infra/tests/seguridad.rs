//! Pruebas de ataque: entradas hostiles (CV, vacantes, archivos dañados) no deben ejecutar nada,
//! leer archivos, romper la base ni tumbar la app.

use std::io::Write;
use std::panic::{AssertUnwindSafe, catch_unwind};

use finder_infra::core::ats::keywords::SkillDict;
use finder_infra::core::document::{CvDocument, DocLang};
use finder_infra::core::model::CvMaster;
use finder_infra::cvfile::{CvFileKind, extract_text};
use finder_infra::db::Db;
use finder_infra::docgen::{render_docx, render_pdf};
use finder_infra::llm::tasks::import_without_ai;
use finder_infra::sources::manual;

type R = Result<(), Box<dyn std::error::Error>>;

/// Textos que en Typst, XML o HTML serían código si se pegaran sin escapar.
const HOSTILE: [&str; 8] = [
    r#"#read("/etc/passwd")"#,
    r#"#import "/etc/hosts": *"#,
    "]#panic(\"x\")[",
    "<w:t>inyectado</w:t></w:r>",
    "& < > \" '",
    "@etiqueta_inexistente $x^2$ `code` *negrita*",
    "<script>alert(1)</script>",
    "\u{202e}texto invertido \u{0}nulo",
];

fn hostile_doc() -> Result<CvDocument, serde_json::Error> {
    let m: CvMaster = serde_json::from_str(include_str!(
        "../../core/tests/fixtures/cv_editor_video.json"
    ))?;
    let mut d = CvDocument::from_master(&m, DocLang::Es);
    d.name = HOSTILE[0].into();
    d.headline = HOSTILE[1].into();
    d.summary = Some(HOSTILE.join(" | "));
    d.experience[0].title = HOSTILE[2].into();
    d.experience[0].bullets = HOSTILE.iter().map(|s| s.to_string()).collect();
    d.skills.push(HOSTILE[3].into());
    Ok(d)
}

#[test]
fn pdf_no_ejecuta_codigo_typst_del_cv() -> R {
    let pdf = render_pdf(&hostile_doc()?)?;
    let text = extract_text(&pdf, CvFileKind::Pdf)?;
    assert!(
        text.contains("#read("),
        "el texto debe salir literal: {text}"
    );
    assert!(!text.contains("root:"), "no debe leer /etc/passwd");
    assert!(!text.contains("localhost"), "no debe leer /etc/hosts");
    Ok(())
}

#[test]
fn docx_escapa_xml_del_cv() -> R {
    let docx = render_docx(&hostile_doc()?)?;
    let text = extract_text(&docx, CvFileKind::Docx)?;
    assert!(text.contains("<w:t>inyectado</w:t></w:r>"), "{text}");
    assert!(text.contains("& < > \" '"));
    Ok(())
}

/// Generador pseudoaleatorio fijo: las pruebas se repiten igual.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
}

/// Archivos dañados al azar: pueden fallar, pero nunca con pánico.
fn fuzz(seed: &[u8], kind: CvFileKind, rounds: usize) {
    let mut rng = Rng(0x5eed_1234_abcd);
    for i in 0..rounds {
        let mut b = seed.to_vec();
        match i % 3 {
            0 => b.truncate(rng.below(b.len())),
            1 => {
                for _ in 0..1 + rng.below(32) {
                    let at = rng.below(b.len());
                    b[at] = rng.next() as u8;
                }
            }
            _ => {
                let at = rng.below(b.len());
                let n = rng.below(4096);
                b.splice(at..at, std::iter::repeat_n(rng.next() as u8, n));
            }
        }
        let r = catch_unwind(AssertUnwindSafe(|| extract_text(&b, kind)));
        assert!(r.is_ok(), "pánico con la mutación {i} de {kind:?}");
    }
}

#[test]
fn pdf_danado_no_tumba_la_app() {
    fuzz(
        include_bytes!("fixtures/cv_simple.pdf"),
        CvFileKind::Pdf,
        300,
    );
}

#[test]
fn docx_danado_no_tumba_la_app() -> R {
    fuzz(&render_docx(&hostile_doc()?)?, CvFileKind::Docx, 300);
    Ok(())
}

#[test]
fn basura_y_vacios_dan_error_claro() {
    for b in [&b""[..], b"%PDF-1.7", b"PK\x03\x04", &[0xff; 64]] {
        for k in [CvFileKind::Pdf, CvFileKind::Docx] {
            assert!(extract_text(b, k).is_err());
        }
    }
}

/// DOCX "bomba": un DOCX válido cuyo `document.xml` comprimido se infla a cientos de MB.
#[test]
fn docx_bomba_no_se_come_la_memoria() -> R {
    let real = render_docx(&hostile_doc()?)?;
    let mut src = zip::ZipArchive::new(std::io::Cursor::new(real))?;
    let mut z = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .large_file(true);
    for i in 0..src.len() {
        let f = src.by_index(i)?;
        let name = f.name()?.to_string();
        if name == "word/document.xml" {
            continue;
        }
        z.raw_copy_file(f)?;
    }
    z.start_file("word/document.xml", opts)?;
    z.write_all(br#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>"#)?;
    let chunk = vec![b'a'; 1 << 20];
    for _ in 0..400 {
        z.write_all(&chunk)?;
    }
    z.write_all(b"</w:t></w:r></w:p></w:body></w:document>")?;
    let bytes = z.finish()?.into_inner();
    assert!(bytes.len() < 2 << 20, "la bomba pesa {} bytes", bytes.len());
    let t = std::time::Instant::now();
    let r = extract_text(&bytes, CvFileKind::Docx);
    println!(
        "bomba: {:?} en {:?}",
        r.as_ref().map(String::len),
        t.elapsed()
    );
    assert!(r.is_err(), "debe rechazarse");
    assert!(t.elapsed().as_secs() < 2, "tardó {:?}", t.elapsed());
    Ok(())
}

#[tokio::test]
async fn sql_y_html_hostiles_en_vacantes() -> R {
    let db = Db::open_in_memory().await?;
    let title = "x'); DROP TABLE jobs; --";
    let html = r#"<html><body><main><h1>Puesto</h1>
        <script>alert('xss')</script><img src=x onerror="alert(1)">
        <p>Descripción real del puesto con suficientes palabras para pasar el mínimo de texto
        que pide el importador de páginas de empleo de la aplicación.</p></main></body></html>"#;
    let mut raw = manual::parse_page("https://ejemplo.com/vacante", html)?;
    raw.title = title.into();
    let id = manual::save(&db, raw).await?.id;
    let job = db.get_job(&id).await?.ok_or("la vacante no se guardó")?;
    assert_eq!(job.title, title, "el título se guarda literal");
    assert!(!job.description.contains("<script"), "{}", job.description);
    assert!(!job.description.contains("onerror"), "{}", job.description);
    assert!(job.description.contains("Descripción real"));
    assert_eq!(db.list_jobs(10).await?.len(), 1, "la tabla sigue viva");
    Ok(())
}

/// El límite contra bombas no debe rechazar DOCX legítimos escritos "en flujo" (con data
/// descriptors, como los que exporta Google Docs).
#[test]
fn docx_con_data_descriptors_se_acepta() -> R {
    let real = render_docx(&hostile_doc()?)?;
    let mut src = zip::ZipArchive::new(std::io::Cursor::new(real))?;
    let mut out = Vec::new();
    {
        let mut z = zip::ZipWriter::new_stream(&mut out);
        for i in 0..src.len() {
            let mut f = src.by_index(i)?;
            z.start_file(
                f.name()?.to_string(),
                zip::write::SimpleFileOptions::default(),
            )?;
            std::io::copy(&mut f, &mut z)?;
        }
        z.finish()?;
    }
    let check = zip::ZipArchive::new(std::io::Cursor::new(&out))?;
    assert_eq!(
        check.decompressed_size(),
        None,
        "debe usar data descriptors"
    );
    let text = extract_text(&out, CvFileKind::Docx)?;
    assert!(text.contains("<w:t>inyectado</w:t></w:r>"));
    Ok(())
}

/// Texto al azar con acentos, emoji, ligaduras, marcas combinadas y escrituras de derecha a
/// izquierda: cortar un texto por bytes en mal sitio es un pánico (y la app se cae).
#[test]
fn texto_unicode_al_azar_no_tumba_el_analisis() -> R {
    use finder_infra::core::ats::keywords::extract_terms;
    use finder_infra::core::cvparse::{LineRange, ParsedCv};
    use finder_infra::core::guard::numbers;

    const PIECES: [&str; 24] = [
        "React",
        "react",
        "Node.js",
        "C++",
        "C#",
        "UX/UI",
        "Premiere Pro",
        "ﬁ",
        "ß",
        "İ",
        "æ",
        "é",
        "e\u{301}",
        "ñ",
        "👩‍💻",
        "🇲🇽",
        "\u{202e}",
        "\u{200b}",
        "٣",
        "日本語",
        "—",
        "•",
        "\n",
        "2019 – 2022 · 15 % · 1.500,50",
    ];
    let dict = SkillDict::builtin()?;
    let mut rng = Rng(0xacce_57ed);
    for i in 0..3000 {
        let text: String = (0..1 + rng.below(40))
            .map(|_| match rng.below(4) {
                0 => " ",
                1 => "\n",
                _ => PIECES[rng.below(PIECES.len())],
            })
            .collect();
        let r = catch_unwind(AssertUnwindSafe(|| {
            dict.find(&text);
            extract_terms(&dict, &text, &text);
            numbers(&text);
            let p = ParsedCv::new(&text);
            p.for_llm();
            let (a, b) = (rng.below(50), rng.below(50));
            p.bullets(LineRange { from: a, to: b });
            import_without_ai(&dict, text.clone());
        }));
        assert!(r.is_ok(), "pánico con el texto {i}: {text:?}");
    }
    Ok(())
}
