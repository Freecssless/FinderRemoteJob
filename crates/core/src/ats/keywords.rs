//! Extracción de palabras clave: normalización, diccionario de habilidades y términos de una vacante.
//!
//! Todo se compara sobre texto normalizado (minúsculas, sin acentos), pero siempre se conserva
//! la forma original del anuncio (`surface`), que es la que el optimizador debe escribir en el CV.

use std::collections::HashMap;

use aho_corasick::{AhoCorasick, MatchKind};
use deunicode::deunicode_char;

use crate::model::{Term, TermKind};

const TECH_CSV: &str = include_str!("../../../../data/skills/tech.csv");
const CREATIVE_CSV: &str = include_str!("../../../../data/skills/creative.csv");
const SYNONYMS_CSV: &str = include_str!("../../../../data/skills/synonyms.csv");

#[derive(Debug, thiserror::Error)]
pub enum DictError {
    #[error("{file}, línea {line}: {msg}")]
    Parse {
        file: String,
        line: usize,
        msg: String,
    },
    #[error("el sinónimo «{alias}» apunta a «{canonical}», que no existe en el diccionario")]
    UnknownCanonical { alias: String, canonical: String },
    #[error("«{pattern}» está definido para «{first}» y para «{second}»")]
    Duplicate {
        pattern: String,
        first: String,
        second: String,
    },
    #[error("no se pudo construir el buscador: {0}")]
    Build(#[from] aho_corasick::BuildError),
}

// ---------------------------------------------------------------------------
// Normalización
// ---------------------------------------------------------------------------

/// Minúsculas y sin acentos: "Edición de Vídeo" → "edicion de video".
pub fn normalize(s: &str) -> String {
    Normalized::new(s).text
}

/// Texto normalizado + mapa de cada byte normalizado al byte original,
/// para recuperar la forma exacta del anuncio a partir de una coincidencia.
struct Normalized {
    text: String,
    map: Vec<usize>,
    original_len: usize,
}

impl Normalized {
    fn new(s: &str) -> Self {
        let mut text = String::with_capacity(s.len());
        let mut map = Vec::with_capacity(s.len());
        for (i, ch) in s.char_indices() {
            // ASCII se deja igual: `deunicode_char('\n')` devuelve "" y pegaría palabras de líneas distintas.
            // Caracteres sin transliteración (p. ej. algunos emoji) se vuelven espacio: cortan palabras.
            let ascii = [ch as u8];
            let rep = if ch.is_ascii() {
                std::str::from_utf8(&ascii).unwrap_or(" ")
            } else {
                deunicode_char(ch).unwrap_or(" ")
            };
            for lc in rep.chars().flat_map(char::to_lowercase) {
                text.push(lc);
                map.extend(std::iter::repeat_n(i, lc.len_utf8()));
            }
        }
        Self {
            text,
            map,
            original_len: s.len(),
        }
    }

    /// Rango en el texto original que corresponde a `start..end` del normalizado.
    fn original_range(&self, start: usize, end: usize) -> (usize, usize) {
        let s = self.map.get(start).copied().unwrap_or(self.original_len);
        let e = self.map.get(end).copied().unwrap_or(self.original_len);
        (s, e)
    }
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric()
}

/// ¿`start..end` está delimitado por bordes de palabra? Evita que "react" coincida dentro de "reactivo".
fn at_word_boundary(text: &str, start: usize, end: usize) -> bool {
    let before_ok = text[..start]
        .chars()
        .next_back()
        .is_none_or(|c| !is_word_char(c));
    let after_ok = text[end..].chars().next().is_none_or(|c| !is_word_char(c));
    before_ok && after_ok
}

/// ¿El texto normalizado contiene `needle` (ya normalizado) como palabra o frase completa?
pub fn contains_word(haystack_norm: &str, needle_norm: &str) -> bool {
    !needle_norm.is_empty()
        && haystack_norm
            .match_indices(needle_norm)
            .any(|(i, m)| at_word_boundary(haystack_norm, i, i + m.len()))
}

// ---------------------------------------------------------------------------
// Diccionario
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct DictSkill {
    pub name: String,
    pub kind: TermKind,
}

/// Una habilidad encontrada en un texto.
#[derive(Debug, Clone, PartialEq)]
pub struct Hit<'a> {
    /// Índice en el diccionario (ver [`SkillDict::skill`]).
    pub skill: usize,
    /// Texto exacto tal como aparece en el original.
    pub surface: &'a str,
    /// Byte de inicio en el texto original.
    pub start: usize,
}

/// Diccionario de habilidades con sinónimos, buscado con Aho-Corasick (miles de patrones en microsegundos).
pub struct SkillDict {
    skills: Vec<DictSkill>,
    /// Patrón normalizado → índice de habilidad.
    lookup: HashMap<String, usize>,
    pattern_skill: Vec<usize>,
    matcher: AhoCorasick,
}

impl SkillDict {
    /// Diccionario incluido en el binario (`data/skills/*.csv`).
    pub fn builtin() -> Result<Self, DictError> {
        Self::from_csv(
            &[("tech.csv", TECH_CSV), ("creative.csv", CREATIVE_CSV)],
            SYNONYMS_CSV,
        )
    }

    /// `skill_files`: (nombre, contenido) con columnas `canonical,kind`.
    /// `synonyms`: contenido con columnas `alias,canonical`.
    pub fn from_csv(skill_files: &[(&str, &str)], synonyms: &str) -> Result<Self, DictError> {
        let mut skills = Vec::new();
        let mut lookup: HashMap<String, usize> = HashMap::new();

        fn add(
            lookup: &mut HashMap<String, usize>,
            pattern: &str,
            idx: usize,
            skills: &[DictSkill],
        ) -> Result<(), DictError> {
            let norm = normalize(pattern.trim());
            match lookup.get(&norm) {
                Some(&prev) if prev != idx => Err(DictError::Duplicate {
                    pattern: pattern.to_string(),
                    first: skills[prev].name.clone(),
                    second: skills[idx].name.clone(),
                }),
                _ => {
                    lookup.insert(norm, idx);
                    Ok(())
                }
            }
        }

        for (file, content) in skill_files {
            for (line, a, b) in csv_rows(file, content)? {
                let kind = parse_kind(b).ok_or_else(|| DictError::Parse {
                    file: file.to_string(),
                    line,
                    msg: format!("tipo desconocido «{b}» (usa hard, tool, soft o language)"),
                })?;
                skills.push(DictSkill {
                    name: a.to_string(),
                    kind,
                });
                add(&mut lookup, a, skills.len() - 1, &skills)?;
            }
        }

        for (_, alias, canonical) in csv_rows("synonyms.csv", synonyms)? {
            let idx = lookup.get(&normalize(canonical)).copied().ok_or_else(|| {
                DictError::UnknownCanonical {
                    alias: alias.to_string(),
                    canonical: canonical.to_string(),
                }
            })?;
            add(&mut lookup, alias, idx, &skills)?;
        }

        let (patterns, pattern_skill): (Vec<String>, Vec<usize>) =
            lookup.iter().map(|(p, &i)| (p.clone(), i)).unzip();
        // LeftmostLongest: "react native" gana sobre "react", "c++" sobre "c".
        let matcher = AhoCorasick::builder()
            .match_kind(MatchKind::LeftmostLongest)
            .build(&patterns)?;

        Ok(Self {
            skills,
            lookup,
            pattern_skill,
            matcher,
        })
    }

    pub fn skill(&self, idx: usize) -> &DictSkill {
        &self.skills[idx]
    }

    pub fn len(&self) -> usize {
        self.skills.len()
    }

    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }

    /// Índice de la habilidad a la que corresponde un nombre o sinónimo.
    pub fn lookup(&self, name: &str) -> Option<usize> {
        self.lookup.get(&normalize(name)).copied()
    }

    /// Todas las habilidades que aparecen en `text`, en orden de aparición.
    pub fn find<'a>(&self, text: &'a str) -> Vec<Hit<'a>> {
        let norm = Normalized::new(text);
        self.matcher
            .find_iter(&norm.text)
            .filter(|m| at_word_boundary(&norm.text, m.start(), m.end()))
            .map(|m| {
                let (s, e) = norm.original_range(m.start(), m.end());
                Hit {
                    skill: self.pattern_skill[m.pattern().as_usize()],
                    surface: &text[s..e],
                    start: s,
                }
            })
            .collect()
    }
}

fn parse_kind(s: &str) -> Option<TermKind> {
    match s.trim() {
        "hard" => Some(TermKind::Hard),
        "tool" => Some(TermKind::Tool),
        "soft" => Some(TermKind::Soft),
        "language" => Some(TermKind::Language),
        _ => None,
    }
}

/// Filas `a,b` de un CSV simple (sin comillas). Ignora encabezado, vacías y comentarios `#`.
fn csv_rows<'a>(file: &str, content: &'a str) -> Result<Vec<(usize, &'a str, &'a str)>, DictError> {
    content
        .lines()
        .enumerate()
        .skip(1)
        .map(|(i, l)| (i + 1, l.trim()))
        .filter(|(_, l)| !l.is_empty() && !l.starts_with('#'))
        .map(|(line, l)| {
            l.split_once(',')
                .map(|(a, b)| (line, a.trim(), b.trim()))
                .ok_or_else(|| DictError::Parse {
                    file: file.to_string(),
                    line,
                    msg: "se esperaban 2 columnas separadas por coma".into(),
                })
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Términos de una vacante
// ---------------------------------------------------------------------------

/// Marcadores de sección "deseable". Se revisan antes que los de "requerido",
/// así "Requisitos deseables" cuenta como deseable.
const PREFERRED_MARKERS: &[&str] = &[
    "deseable",
    "deseables",
    "desirable",
    "es un plus",
    "sera un plus",
    "plus",
    "nice to have",
    "nice-to-have",
    "preferred",
    "preferible",
    "valoramos",
    "se valorara",
    "bonus",
];
const REQUIRED_MARKERS: &[&str] = &[
    "requisitos",
    "requisito",
    "requerimientos",
    "requirements",
    "required",
    "must have",
    "must-have",
    "indispensable",
    "necesario",
    "obligatorio",
    "qualifications",
    "lo que buscamos",
];

/// Pesos del plan (§4.3).
const W_REQUIRED: f32 = 3.0;
const W_PREFERRED: f32 = 1.0;
const W_IN_TITLE: f32 = 2.0;

/// Extrae los términos de una vacante con el diccionario (parte determinista, sin IA).
///
/// - Requerido vs. deseable: según la última sección vista ("Requisitos", "Deseable"...).
///   Por defecto, requerido.
/// - Peso: requerido 3 · deseable 1 · +2 si aparece en el título · +ln(frecuencia).
pub fn extract_terms(dict: &SkillDict, title: &str, description: &str) -> Vec<Term> {
    struct Acc {
        surface: String,
        surface_is_canonical: bool,
        required: bool,
        in_title: bool,
        freq: u32,
    }
    let mut order: Vec<usize> = Vec::new();
    let mut acc: HashMap<usize, Acc> = HashMap::new();
    let mut record = |skill: usize, surface: &str, required: bool, in_title: bool| {
        let e = acc.entry(skill).or_insert_with(|| {
            order.push(skill);
            Acc {
                surface: surface.to_string(),
                surface_is_canonical: false,
                required: false,
                in_title: false,
                freq: 0,
            }
        });
        // La forma a escribir en el CV es la primera aparición; pero si el anuncio también usa el
        // nombre de la habilidad ("edición de video" vs. el puesto "Editor de Video"), gana ese.
        if !e.surface_is_canonical && normalize(surface) == normalize(&dict.skill(skill).name) {
            e.surface = surface.to_string();
            e.surface_is_canonical = true;
        }
        e.required |= required;
        e.in_title |= in_title;
        e.freq += 1;
    };

    for hit in dict.find(title) {
        record(hit.skill, hit.surface, true, true);
    }

    let mut preferred = false;
    for line in description.lines() {
        let n = normalize(line);
        if PREFERRED_MARKERS.iter().any(|m| contains_word(&n, m)) {
            preferred = true;
        } else if REQUIRED_MARKERS.iter().any(|m| contains_word(&n, m)) {
            preferred = false;
        }
        for hit in dict.find(line) {
            record(hit.skill, hit.surface, !preferred, false);
        }
    }

    order
        .into_iter()
        .filter_map(|skill| {
            let a = acc.remove(&skill)?;
            let s = dict.skill(skill);
            let base = if a.required { W_REQUIRED } else { W_PREFERRED };
            let title_bonus = if a.in_title { W_IN_TITLE } else { 0.0 };
            Some(Term {
                surface: a.surface,
                canonical: s.name.clone(),
                kind: s.kind,
                required: a.required,
                weight: base + title_bonus + (a.freq as f32).ln(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    type R = Result<(), DictError>;

    fn names(dict: &SkillDict, text: &str) -> Vec<String> {
        dict.find(text)
            .into_iter()
            .map(|h| dict.skill(h.skill).name.clone())
            .collect()
    }

    #[test]
    fn diccionario_incluido_es_valido() -> R {
        let dict = SkillDict::builtin()?;
        assert!(dict.len() > 120, "solo {} habilidades", dict.len());
        Ok(())
    }

    #[test]
    fn normaliza_acentos_y_mayusculas() {
        assert_eq!(normalize("Edición de Vídeo"), "edicion de video");
        assert_eq!(normalize("Diseño UX"), "diseno ux");
        assert_eq!(normalize("Ana\nVideo\tEditor"), "ana\nvideo\teditor");
    }

    #[test]
    fn conserva_la_forma_original_del_texto() -> R {
        let dict = SkillDict::builtin()?;
        let hits = dict.find("Buscamos experiencia en Edición de Vídeo.");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].surface, "Edición de Vídeo");
        assert_eq!(dict.skill(hits[0].skill).name, "Edición de video");
        Ok(())
    }

    #[test]
    fn respeta_bordes_de_palabra_y_prefiere_la_coincidencia_mas_larga() -> R {
        let dict = SkillDict::builtin()?;
        assert_eq!(
            names(&dict, "React Native y React"),
            ["React Native", "React"]
        );
        assert!(names(&dict, "Un perfil reactivo y javascripter").is_empty());
        assert_eq!(names(&dict, "C++, C# y ASP.NET"), ["C++", "C#", "ASP.NET"]);
        Ok(())
    }

    #[test]
    fn sinonimos_apuntan_a_la_habilidad_canonica() -> R {
        let dict = SkillDict::builtin()?;
        assert_eq!(
            names(&dict, "Dominio de Premiere y JS"),
            ["Adobe Premiere Pro", "JavaScript"]
        );
        assert_eq!(dict.lookup("k8s"), dict.lookup("Kubernetes"));
        Ok(())
    }

    #[test]
    fn detecta_errores_del_diccionario() {
        let mal_tipo = SkillDict::from_csv(
            &[("t.csv", "canonical,kind\nRust,lenguaje")],
            "alias,canonical",
        );
        assert!(matches!(mal_tipo, Err(DictError::Parse { line: 2, .. })));

        let sin_canonico = SkillDict::from_csv(
            &[("t.csv", "canonical,kind\nRust,hard")],
            "alias,canonical\nrs,Ferris",
        );
        assert!(matches!(
            sin_canonico,
            Err(DictError::UnknownCanonical { .. })
        ));

        let duplicado = SkillDict::from_csv(
            &[("t.csv", "canonical,kind\nRust,hard\nRuby,hard")],
            "alias,canonical\nr,Rust\nr,Ruby",
        );
        assert!(matches!(duplicado, Err(DictError::Duplicate { .. })));
    }

    fn term<'a>(terms: &'a [Term], canonical: &str) -> &'a Term {
        terms
            .iter()
            .find(|t| t.canonical == canonical)
            .unwrap_or_else(|| panic!("falta el término {canonical}"))
    }

    #[test]
    fn extrae_requeridos_deseables_y_pesos_de_vacante_editor() -> R {
        let dict = SkillDict::builtin()?;
        let jd = include_str!("../../tests/fixtures/jd_editor_video_es.txt");
        let terms = extract_terms(&dict, "Editor de Video (Remoto - LATAM)", jd);

        let premiere = term(&terms, "Adobe Premiere Pro");
        assert!(premiere.required);
        assert_eq!(premiere.surface, "Adobe Premiere Pro");
        assert_eq!(premiere.weight, W_REQUIRED);

        let davinci = term(&terms, "DaVinci Resolve");
        assert!(!davinci.required);
        assert_eq!(davinci.weight, W_PREFERRED);

        // Aparece en el título y 2 veces más en el texto: 3 + 2 + ln(3).
        let edicion = term(&terms, "Edición de video");
        assert!(edicion.required);
        // Se prefiere el nombre de la habilidad usado en el anuncio sobre el título del puesto.
        assert_eq!(edicion.surface, "edición de video");
        assert!((edicion.weight - (5.0 + 3f32.ln())).abs() < 1e-4);

        assert!(!term(&terms, "Animación 2D").required);
        assert!(term(&terms, "Inglés").required);
        Ok(())
    }

    #[test]
    fn extrae_terminos_de_vacante_en_ingles() -> R {
        let dict = SkillDict::builtin()?;
        let jd = include_str!("../../tests/fixtures/jd_frontend_jr_en.txt");
        let terms = extract_terms(&dict, "Junior Frontend Developer", jd);

        for req in [
            "React",
            "TypeScript",
            "JavaScript",
            "API REST",
            "Git",
            "Inglés",
        ] {
            assert!(term(&terms, req).required, "{req} debería ser requerido");
        }
        for nice in [
            "Next.js",
            "Tailwind CSS",
            "Pruebas unitarias",
            "Jest",
            "Figma",
        ] {
            assert!(!term(&terms, nice).required, "{nice} debería ser deseable");
        }
        assert_eq!(term(&terms, "API REST").surface, "REST APIs");
        Ok(())
    }

    #[test]
    fn requisitos_indispensables_y_sera_un_plus() -> R {
        let dict = SkillDict::builtin()?;
        let jd = include_str!("../../tests/fixtures/jd_motion_es.txt");
        let terms = extract_terms(&dict, "Motion Graphics Designer Sr.", jd);

        assert!(term(&terms, "Motion graphics").required);
        assert!(term(&terms, "After Effects").required);
        assert!(!term(&terms, "Blender").required);
        assert!(!term(&terms, "Portugués").required);
        // "C4D" y "Cinema 4D" se agrupan en un solo término; la forma mostrada es la primera que aparece.
        assert_eq!(
            terms.iter().filter(|t| t.canonical == "Cinema 4D").count(),
            1
        );
        Ok(())
    }
}
