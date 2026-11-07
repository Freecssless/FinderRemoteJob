//! Lectura determinista del texto de un CV, antes y después de la IA.
//!
//! La IA local es lenta generando texto (en CPU, ~10 palabras por segundo), y copiar cada logro
//! letra por letra era casi todo el tiempo de la importación. Aquí:
//! - se numeran las líneas y se detectan las secciones por sus encabezados;
//! - el contacto (correo, teléfono, enlaces), las habilidades y los idiomas se leen sin IA;
//! - la IA solo devuelve los datos cortos de cada empleo y **qué líneas** son cada logro
//!   ([`LineRange`]); el texto se copia de aquí, literal, sin riesgo de que la IA lo cambie.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::ats::keywords::{SkillDict, normalize};
use crate::model::{Basics, Language, TermKind};

/// Secciones habituales de un CV (español e inglés).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionKind {
    Summary,
    Experience,
    Education,
    Skills,
    Languages,
    Certifications,
    Other,
}

const HEADINGS: &[(&str, SectionKind)] = &[
    ("perfil", SectionKind::Summary),
    ("perfil profesional", SectionKind::Summary),
    ("resumen", SectionKind::Summary),
    ("resumen profesional", SectionKind::Summary),
    ("sobre mi", SectionKind::Summary),
    ("acerca de mi", SectionKind::Summary),
    ("objetivo", SectionKind::Summary),
    ("objetivo profesional", SectionKind::Summary),
    ("summary", SectionKind::Summary),
    ("profile", SectionKind::Summary),
    ("about me", SectionKind::Summary),
    ("experiencia", SectionKind::Experience),
    ("experiencia profesional", SectionKind::Experience),
    ("experiencia laboral", SectionKind::Experience),
    ("trayectoria profesional", SectionKind::Experience),
    ("historial laboral", SectionKind::Experience),
    ("experience", SectionKind::Experience),
    ("work experience", SectionKind::Experience),
    ("professional experience", SectionKind::Experience),
    ("employment", SectionKind::Experience),
    ("employment history", SectionKind::Experience),
    ("educacion", SectionKind::Education),
    ("formacion", SectionKind::Education),
    ("formacion academica", SectionKind::Education),
    ("estudios", SectionKind::Education),
    ("education", SectionKind::Education),
    ("habilidades", SectionKind::Skills),
    ("habilidades tecnicas", SectionKind::Skills),
    ("competencias", SectionKind::Skills),
    ("aptitudes", SectionKind::Skills),
    ("conocimientos", SectionKind::Skills),
    ("herramientas", SectionKind::Skills),
    ("skills", SectionKind::Skills),
    ("technical skills", SectionKind::Skills),
    ("idiomas", SectionKind::Languages),
    ("lenguas", SectionKind::Languages),
    ("languages", SectionKind::Languages),
    ("certificaciones", SectionKind::Certifications),
    ("certificados", SectionKind::Certifications),
    ("cursos", SectionKind::Certifications),
    ("cursos y certificaciones", SectionKind::Certifications),
    ("certifications", SectionKind::Certifications),
    ("licenses & certifications", SectionKind::Certifications),
    ("proyectos", SectionKind::Other),
    ("projects", SectionKind::Other),
    ("referencias", SectionKind::Other),
    ("references", SectionKind::Other),
    ("voluntariado", SectionKind::Other),
    ("intereses", SectionKind::Other),
    ("logros", SectionKind::Other),
    ("premios", SectionKind::Other),
];

fn heading_kind(line: &str) -> Option<SectionKind> {
    let t = line.trim().trim_end_matches(':').trim();
    if t.is_empty() || t.chars().count() > 40 {
        return None;
    }
    let n = normalize(t);
    let n = n.trim_matches(|c: char| !c.is_alphanumeric() && c != '&');
    HEADINGS.iter().find(|(h, _)| *h == n).map(|(_, k)| *k)
}

/// Rango de líneas (1 = primera línea del CV), ambos extremos incluidos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineRange {
    pub from: usize,
    pub to: usize,
}

/// El CV ya leído: líneas, secciones y lo que se pudo sacar sin IA.
#[derive(Debug, Clone)]
pub struct ParsedCv {
    lines: Vec<String>,
    /// (tipo, primera línea del contenido, última línea), índices desde 0.
    sections: Vec<(SectionKind, usize, usize)>,
    pub contact: Basics,
    /// `None` si el CV no tiene una sección de habilidades reconocible (entonces se le pide a la IA).
    pub skills: Option<Vec<String>>,
    pub languages: Option<Vec<Language>>,
}

impl ParsedCv {
    pub fn new(text: &str) -> Self {
        let lines: Vec<String> = text.lines().map(|l| l.trim_end().to_string()).collect();
        let mut sections = Vec::new();
        let mut current: Option<(SectionKind, usize)> = None;
        for (i, l) in lines.iter().enumerate() {
            if let Some(k) = heading_kind(l) {
                if let Some((pk, start)) = current.take() {
                    sections.push((pk, start, i.saturating_sub(1)));
                }
                current = Some((k, i + 1));
            }
        }
        if let Some((k, start)) = current {
            sections.push((k, start, lines.len().saturating_sub(1)));
        }
        let mut cv = Self {
            contact: contact(text),
            skills: None,
            languages: None,
            lines,
            sections,
        };
        cv.skills = cv
            .section_text(SectionKind::Skills)
            .map(|t| split_items(&t));
        cv.languages = cv
            .section_text(SectionKind::Languages)
            .map(|t| split_items(&t).iter().filter_map(|i| language(i)).collect());
        cv
    }

    fn section_text(&self, kind: SectionKind) -> Option<String> {
        let (_, a, b) = *self.sections.iter().find(|(k, _, _)| *k == kind)?;
        (a <= b).then(|| self.lines[a..=b].join("\n"))
    }

    /// Lo que se le manda a la IA: el encabezado del CV y las secciones que necesita, con su número
    /// de línea. Sin resumen, habilidades ni idiomas si ya se leyeron aquí (menos texto = más rápido).
    pub fn for_llm(&self) -> String {
        let first_section = self
            .sections
            .first()
            .map_or(self.lines.len(), |s| s.1.saturating_sub(1));
        let skip: Vec<SectionKind> = [
            Some(SectionKind::Summary),
            self.skills.is_some().then_some(SectionKind::Skills),
            self.languages.is_some().then_some(SectionKind::Languages),
            Some(SectionKind::Other),
        ]
        .into_iter()
        .flatten()
        .collect();
        let skipped = |i: usize| {
            self.sections
                .iter()
                .any(|(k, a, b)| skip.contains(k) && (a.saturating_sub(1)..=*b).contains(&i))
        };
        // Sin secciones reconocibles se manda todo.
        let keep = |i: usize| self.sections.is_empty() || i < first_section || !skipped(i);
        self.lines
            .iter()
            .enumerate()
            .filter(|(i, l)| keep(*i) && !l.trim().is_empty())
            .map(|(i, l)| format!("{}| {}", i + 1, l.trim()))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Líneas de las secciones, sin el encabezado ni los idiomas.
    fn body_text(&self) -> String {
        self.sections
            .iter()
            .filter(|(k, _, _)| *k != SectionKind::Languages)
            .filter(|(_, a, b)| a <= b)
            .map(|(_, a, b)| self.lines[*a..=*b].join("\n"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// ¿Hace falta pedirle a la IA las habilidades / los idiomas?
    pub fn needs_skills(&self) -> bool {
        self.skills.is_none()
    }
    pub fn needs_languages(&self) -> bool {
        self.languages.is_none()
    }

    /// Logros literales de un bloque de líneas (la descripción de un empleo): uno por viñeta; sin
    /// viñetas, uno por frase que termina en punto. Une los renglones que cortó el PDF y quita las
    /// viñetas. Vacío si el rango no existe o cruza un encabezado de sección.
    pub fn bullets(&self, r: LineRange) -> Vec<String> {
        let (Some(from), Some(to)) = (r.from.checked_sub(1), r.to.checked_sub(1)) else {
            return Vec::new();
        };
        if from > to || to >= self.lines.len() || to - from > 60 {
            return Vec::new();
        }
        let mut block: Vec<&str> = self.lines[from..=to].iter().map(|l| l.trim()).collect();
        if block.iter().any(|l| heading_kind(l).is_some()) {
            return Vec::new();
        }
        let marked = block.iter().any(|l| strip_bullet(l).len() < l.len());
        // La IA a veces corta el bloque una línea antes: se siguen tomando las viñetas y los
        // renglones de continuación que vienen justo después, hasta una línea en blanco.
        for l in self.lines[to + 1..].iter().map(|l| l.trim()) {
            let prev = block.last().copied().unwrap_or("");
            let bullet = strip_bullet(l).len() < l.len();
            let unfinished = !prev.ends_with(['.', ';', '!', '?']);
            // En una lista con viñetas, un renglón sin viñeta tras un logro sin punto final lo
            // continúa ("…en React y" + "TypeScript."), salvo que parezca el encabezado de otro empleo.
            let continues = unfinished
                && !bullet
                && (l.starts_with(char::is_lowercase) || (marked && !looks_like_header(l)));
            if l.is_empty() || heading_kind(l).is_some() || !((marked && bullet) || continues) {
                break;
            }
            block.push(l);
        }
        let mut out: Vec<String> = Vec::new();
        let mut prev_ends_sentence = true;
        for l in block {
            if l.is_empty() {
                prev_ends_sentence = true;
                continue;
            }
            let starts_item = if marked {
                strip_bullet(l).len() < l.len()
            } else {
                prev_ends_sentence
            };
            let text = strip_bullet(l);
            match out.last_mut() {
                Some(cur) if !starts_item => {
                    if cur.ends_with('-') && text.starts_with(char::is_lowercase) {
                        cur.pop(); // palabra partida con guion al cambiar de línea
                    } else {
                        cur.push(' ');
                    }
                    cur.push_str(text);
                }
                _ => out.push(text.to_string()),
            }
            prev_ends_sentence = text.ends_with(['.', ';', '!', '?']);
        }
        out.retain(|b| !b.is_empty());
        out
    }

    /// Habilidades finales: las de la sección (o las que dio la IA) y, después, las del diccionario
    /// que el CV menciona en otro lugar (en un logro, por ejemplo). Están escritas en el CV: no se
    /// inventa nada, y mejoran la compatibilidad con las vacantes. Sin repetir (sinónimos incluidos).
    pub fn merge_skills(&self, dict: &SkillDict, listed: Vec<String>) -> Vec<String> {
        let mut seen_canon = HashSet::new();
        let mut seen_text = HashSet::new();
        let mut out = Vec::new();
        let mut push = |name: String, out: &mut Vec<String>| {
            let key = dict.lookup(&name);
            if !seen_text.insert(normalize(&name)) || key.is_some_and(|k| !seen_canon.insert(k)) {
                return;
            }
            out.push(name);
        };
        for s in listed {
            push(s, &mut out);
        }
        // Solo en el cuerpo del CV: el encabezado trae enlaces ("github.com") y la sección de
        // idiomas ya va aparte. Los idiomas del diccionario tampoco son habilidades.
        let text = self.body_text();
        for hit in dict.find(&text) {
            let skill = dict.skill(hit.skill);
            if skill.kind == TermKind::Language {
                continue;
            }
            push(skill.name.clone(), &mut out);
        }
        out
    }
}

/// "Diseñadora | Empresa", "Editor — Agencia", "Mar 2022 – Presente": no es un renglón de un logro.
fn looks_like_header(l: &str) -> bool {
    l.contains('|')
        || l.contains(" — ")
        || l.contains(" – ")
        || l.split(|c: char| !c.is_ascii_digit())
            .any(|p| p.len() == 4 && (p.starts_with("19") || p.starts_with("20")))
}

/// Quita la viñeta del principio de un logro.
fn strip_bullet(l: &str) -> &str {
    l.trim_start_matches([
        '•', '-', '*', '–', '—', '▪', '●', '○', '·', '►', '◦', '\u{f0b7}', '\u{f0a7}', '✓', '>',
    ])
    .trim_start()
}

/// Elementos de una lista escrita en línea o en renglones: "Figma, Sketch · React | Git".
fn split_items(text: &str) -> Vec<String> {
    // Une renglones partidos ("Pruebas de\nusabilidad") cuando el siguiente empieza en minúscula.
    let mut joined = String::new();
    for l in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
        let l = strip_bullet(l);
        if !joined.is_empty() {
            let cont = l.starts_with(char::is_lowercase) && !joined.ends_with([',', ';']);
            joined.push(if cont { ' ' } else { '\n' });
        }
        joined.push_str(l);
    }
    let mut seen = HashSet::new();
    joined
        .split([',', ';', '·', '•', '|', '\n'])
        .map(|s| s.trim().trim_end_matches('.').trim())
        // Una «habilidad» de más de 6 palabras es una frase, no una lista.
        .filter(|s| !s.is_empty() && s.split_whitespace().count() <= 6)
        .filter(|s| seen.insert(normalize(s)))
        .map(String::from)
        .collect()
}

/// "Inglés: avanzado (C1)", "Inglés (B2)", "Inglés - nativo", "Inglés C1".
fn language(item: &str) -> Option<Language> {
    let item = item.trim();
    let (name, level) = if let Some((n, l)) = item.split_once([':', '–', '—']) {
        (n, l)
    } else if let Some((n, l)) = item.split_once(" - ") {
        (n, l)
    } else if let Some((n, l)) = item.split_once('(') {
        (n, l.trim_end_matches(')'))
    } else if let Some((n, l)) = item.rsplit_once(' ') {
        // "Inglés C1": solo si lo último parece un nivel.
        if is_level(l) { (n, l) } else { (item, "") }
    } else {
        (item, "")
    };
    let name = name.trim();
    (!name.is_empty() && name.split_whitespace().count() <= 3).then(|| Language {
        name: name.to_string(),
        level: level.trim().to_string(),
    })
}

fn is_level(s: &str) -> bool {
    let n = normalize(s);
    [
        "a1",
        "a2",
        "b1",
        "b2",
        "c1",
        "c2",
        "nativo",
        "native",
        "basico",
        "intermedio",
        "avanzado",
        "fluido",
        "fluent",
    ]
    .contains(&n.as_str())
}

/// Correo, teléfono y enlaces, sin IA.
fn contact(text: &str) -> Basics {
    let mut b = Basics::default();
    for token in text.split(|c: char| c.is_whitespace() || matches!(c, '·' | '|' | '•' | ',' | ';'))
    {
        let t = token
            .trim_matches(|c: char| matches!(c, '(' | ')' | '<' | '>' | '"' | '\'' | '.' | ':'));
        if t.is_empty() {
            continue;
        }
        if b.email.is_empty() && is_email(t) {
            b.email = t.to_string();
        } else if !t.contains('@') && is_link(t) && !b.links.iter().any(|l| l == t) {
            b.links.push(t.to_string());
        }
    }
    b.phone = phone(text).unwrap_or_default();
    b
}

fn is_email(t: &str) -> bool {
    t.split_once('@').is_some_and(|(u, d)| {
        !u.is_empty() && d.contains('.') && !d.starts_with('.') && !d.ends_with('.')
    })
}

fn is_link(t: &str) -> bool {
    let l = t.to_lowercase();
    if l.starts_with("http://") || l.starts_with("https://") || l.starts_with("www.") {
        return true;
    }
    // "linkedin.com/in/ana", "ana.design": un dominio conocido o una ruta.
    const TLDS: &[&str] = &[
        ".com", ".dev", ".io", ".me", ".design", ".net", ".org", ".mx", ".co", ".app", ".site",
        ".xyz", ".ai",
    ];
    let host = l.split('/').next().unwrap_or("");
    host.contains('.')
        && host
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '.' | '-'))
        && TLDS.iter().any(|t| host.ends_with(t))
        && !host.chars().all(|c| c.is_ascii_digit() || c == '.')
}

/// El primer número de teléfono: 8 a 15 dígitos, con +, espacios, guiones, puntos o paréntesis.
fn phone(text: &str) -> Option<String> {
    for line in text.lines() {
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '+' || chars[i].is_ascii_digit() || chars[i] == '(' {
                let start = i;
                let mut digits = 0;
                let mut end = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || " +-().".contains(chars[i]))
                {
                    if chars[i].is_ascii_digit() {
                        digits += 1;
                        end = i;
                    }
                    i += 1;
                }
                let s: String = chars[start..=end].iter().collect();
                // Un rango de años ("2019 – 2022") no es un teléfono.
                let years = s
                    .split(|c: char| !c.is_ascii_digit())
                    .filter(|p| !p.is_empty())
                    .all(|p| p.len() == 4 && (p.starts_with("19") || p.starts_with("20")));
                if (8..=15).contains(&digits) && !years {
                    return Some(s.trim().to_string());
                }
            }
            i += 1;
        }
    }
    None
}

/// "MARÍA FERNANDA LÓPEZ" → "María Fernanda López" (solo si todo viene en mayúsculas).
pub fn tidy_name(name: &str) -> String {
    let name = name.trim();
    if !name.chars().any(char::is_lowercase) && name.chars().any(char::is_uppercase) {
        name.split_whitespace()
            .map(|w| {
                let lower = w.to_lowercase();
                // Partículas en minúscula: "de", "del", "la", "y".
                if ["de", "del", "la", "las", "los", "y"].contains(&lower.as_str()) {
                    return lower;
                }
                let mut c = lower.chars();
                c.next()
                    .map(|f| f.to_uppercase().chain(c).collect())
                    .unwrap_or_default()
            })
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        name.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CV: &str = include_str!("../tests/fixtures/cv_realista.txt");

    #[test]
    fn lee_contacto_habilidades_e_idiomas_sin_ia() {
        let p = ParsedCv::new(CV);
        assert_eq!(p.contact.email, "maria.lopez@correo.com");
        assert_eq!(p.contact.phone, "+52 33 1234 5678");
        assert_eq!(
            p.contact.links,
            [
                "linkedin.com/in/mariafernandalopez",
                "github.com/mflopez",
                "mflopez.design"
            ]
        );
        let skills = p.skills.clone().unwrap_or_default();
        assert!(
            skills.contains(&"Pruebas de usabilidad".to_string()),
            "{skills:?}"
        );
        assert!(skills.contains(&"Design Systems".to_string()));
        assert_eq!(skills.len(), 13, "{skills:?}");
        let langs = p.languages.clone().unwrap_or_default();
        assert_eq!(
            langs,
            [
                Language {
                    name: "Español".into(),
                    level: "nativo".into()
                },
                Language {
                    name: "Inglés".into(),
                    level: "avanzado (C1)".into()
                },
                Language {
                    name: "Portugués".into(),
                    level: "básico".into()
                },
            ]
        );
    }

    #[test]
    fn a_la_ia_solo_le_llega_lo_necesario_con_numero_de_linea() {
        let p = ParsedCv::new(CV);
        let t = p.for_llm();
        assert!(t.starts_with("1| MARÍA FERNANDA LÓPEZ RUIZ"), "{t}");
        assert!(t.contains("| Diseñadora UX/UI Senior | Banco Digital Nova"));
        assert!(t.contains("| EDUCACIÓN") && t.contains("| CERTIFICACIONES"));
        // Resumen, habilidades e idiomas ya se leyeron: no se mandan.
        assert!(
            !t.contains("Combino investigación")
                && !t.contains("Figma, Sketch")
                && !t.contains("Portugués"),
            "{t}"
        );
    }

    #[test]
    fn separa_los_logros_y_une_renglones() {
        let p = ParsedCv::new(CV);
        let n = |needle: &str| {
            CV.lines()
                .position(|l| l.contains(needle))
                .map(|i| i + 1)
                .unwrap_or(0)
        };
        // La IA da el bloque entero; los logros se separan por sus viñetas.
        let b = p.bullets(LineRange {
            from: n("Rediseñé el flujo"),
            to: n("Colaboré con 4") + 1,
        });
        assert_eq!(b.len(), 4, "{b:?}");
        assert_eq!(
            b[0],
            "Rediseñé el flujo de apertura de cuenta y aumenté la conversión 18% en seis meses."
        );
        assert_eq!(
            p.bullets(LineRange {
                from: n("Colaboré con 4"),
                to: n("Colaboré con 4")
            }),
            [b[3].clone()]
        );
        assert_eq!(
            b[3],
            "Colaboré con 4 equipos de desarrollo para implementar componentes en React y TypeScript."
        );
        // Un rango que cruza un encabezado o sale del CV no sirve.
        assert!(
            p.bullets(LineRange {
                from: n("Integré CMS"),
                to: n("EDUCACIÓN")
            })
            .is_empty()
        );
        assert!(p.bullets(LineRange { from: 900, to: 901 }).is_empty());
        // Sin viñetas: un logro por frase; guion de palabra partida.
        let sin = ParsedCv::new(
            "Optimicé el desa-\nrrollo del sitio.\nAtendí a 20 clientes por\nsemana.",
        );
        assert_eq!(
            sin.bullets(LineRange { from: 1, to: 4 }),
            [
                "Optimicé el desarrollo del sitio.",
                "Atendí a 20 clientes por semana."
            ]
        );
    }

    #[test]
    fn suma_las_habilidades_que_aparecen_en_los_logros() -> Result<(), Box<dyn std::error::Error>> {
        let dict = SkillDict::builtin()?;
        let p = ParsedCv::new(CV);
        let all = p.merge_skills(&dict, p.skills.clone().unwrap_or_default());
        // WordPress y Scrum solo están en los logros y las certificaciones.
        assert!(all.iter().any(|s| s == "WordPress"), "{all:?}");
        assert!(all.iter().any(|s| s == "Scrum"), "{all:?}");
        // Ni el enlace de GitHub ni los idiomas cuentan como habilidad.
        assert!(
            !all.iter()
                .any(|s| normalize(s) == "github" || s == "Inglés"),
            "{all:?}"
        );
        // Sin repetir lo que ya estaba en la lista.
        assert_eq!(all.iter().filter(|s| normalize(s) == "figma").count(), 1);
        assert_eq!(all[..13], p.skills.clone().unwrap_or_default()[..]);
        Ok(())
    }

    #[test]
    fn nombres_en_mayusculas_y_telefonos() {
        assert_eq!(
            tidy_name("MARÍA FERNANDA LÓPEZ DE LA ROSA"),
            "María Fernanda López de la Rosa"
        );
        assert_eq!(tidy_name("Ana McDonald"), "Ana McDonald");
        assert_eq!(phone("Trabajé 2019 – 2022 en X"), None);
        assert_eq!(
            phone("Tel. (55) 1234-5678").as_deref(),
            Some("(55) 1234-5678")
        );
        assert_eq!(
            language("Inglés C1").map(|l| l.level).as_deref(),
            Some("C1")
        );
        assert_eq!(language("Francés").map(|l| l.level).as_deref(), Some(""));
    }
}
