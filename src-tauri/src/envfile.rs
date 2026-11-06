//! Lectura mínima de `.env` (`CLAVE=valor`, `#` comentarios, comillas opcionales).
//!
//! No se escribe en el entorno del proceso (`set_var` no es seguro con hilos): los valores se pasan
//! a `LlmConfig::from_lookup`, y una variable real del entorno siempre gana sobre el archivo.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub fn parse(text: &str) -> HashMap<String, String> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.strip_prefix("export ").unwrap_or(l).split_once('='))
        .map(|(k, v)| {
            let v = v.trim();
            let v = match (v.chars().next(), v.chars().last()) {
                (Some(q @ ('"' | '\'')), Some(e)) if v.len() >= 2 && q == e => &v[1..v.len() - 1],
                // Comentario al final solo si va separado por espacio: `a=b # nota`.
                _ => v.split_once(" #").map_or(v, |(v, _)| v.trim_end()),
            };
            (k.trim().to_string(), v.to_string())
        })
        .collect()
}

/// Primer `.env` que exista: carpeta de configuración de la app y, en desarrollo, el directorio
/// actual y sus padres (así `npm run tauri dev` encuentra el `.env` de la raíz del repo).
pub fn find(config_dir: &Path) -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok();
    std::iter::once(config_dir)
        .chain(cwd.iter().flat_map(|d| d.ancestors()))
        .map(|d| d.join(".env"))
        .find(|p| p.is_file())
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn parses_common_forms() {
        let m = parse(
            "# comentario\n\nFRJ_WRITER=local\nexport A = \"con espacios\"\nB='x#y'\nC=valor # nota\nD=a#b\nSIN_IGUAL\n",
        );
        assert_eq!(m["FRJ_WRITER"], "local");
        assert_eq!(m["A"], "con espacios");
        assert_eq!(m["B"], "x#y");
        assert_eq!(m["C"], "valor");
        assert_eq!(m["D"], "a#b");
        assert_eq!(m.len(), 5);
    }
}
