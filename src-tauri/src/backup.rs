//! Respaldo y restauración en un solo `.zip`: la base (copia consistente) y la carpeta `cvs/`.
//! Restaurar deja el archivo pendiente y reinicia la app: la base se cambia antes de abrirla.

use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager, State};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

use crate::{AppState, CmdResult, err};

const DB: &str = "finder.db";
const CVS: &str = "cvs";
/// Carpeta de datos de donde salió el respaldo (para corregir las rutas de los CV).
const ORIGIN: &str = "carpeta.txt";
const PENDING: &str = "restaurar.zip";
const OLD: &str = ".antes-de-restaurar";

/// Crea `FinderRemoteJob-respaldo-AAAA-MM-DD.zip` en Documentos y devuelve su ruta.
#[tauri::command]
pub async fn create_backup(app: AppHandle, state: State<'_, AppState>) -> CmdResult<String> {
    let dir = app
        .path()
        .document_dir()
        .or_else(|_| app.path().home_dir())
        .map_err(err)?;
    let date: String = state
        .db
        .now()
        .await
        .map_err(err)?
        .chars()
        .take(10)
        .collect();
    let dest = dir.join(format!("FinderRemoteJob-respaldo-{date}.zip"));

    let tmp = state.data_dir.join("respaldo.db");
    let _ = fs::remove_file(&tmp);
    state.db.backup_to(&tmp).await.map_err(err)?;
    let result = write_zip(&dest, &tmp, &state.data_dir);
    let _ = fs::remove_file(&tmp);
    result.map_err(|e| format!("No se pudo crear el respaldo: {e}"))?;
    Ok(dest.display().to_string())
}

fn write_zip(dest: &Path, db: &Path, data_dir: &Path) -> std::io::Result<()> {
    let file = fs::File::create(dest)?;
    // El respaldo lleva tu CV: en Documentos, solo legible por tu usuario.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(fs::Permissions::from_mode(0o600))?;
    }
    let mut z = ZipWriter::new(file);
    let opts = SimpleFileOptions::default();
    z.start_file(DB, opts)?;
    std::io::copy(&mut fs::File::open(db)?, &mut z)?;
    z.start_file(ORIGIN, opts)?;
    z.write_all(data_dir.to_string_lossy().as_bytes())?;
    let mut stack = vec![data_dir.join(CVS)];
    while let Some(d) = stack.pop() {
        let Ok(entries) = fs::read_dir(&d) else {
            continue;
        };
        for e in entries {
            let path = e?.path();
            if path.is_dir() {
                stack.push(path);
            } else if let Ok(rel) = path.strip_prefix(data_dir) {
                let name = rel
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy())
                    .collect::<Vec<_>>()
                    .join("/");
                z.start_file(name, opts)?;
                std::io::copy(&mut fs::File::open(&path)?, &mut z)?;
            }
        }
    }
    z.finish()?;
    Ok(())
}

/// Valida el respaldo elegido, lo deja pendiente y reinicia la app para aplicarlo.
#[tauri::command]
pub async fn restore_backup(
    app: AppHandle,
    state: State<'_, AppState>,
    bytes: Vec<u8>,
) -> CmdResult<()> {
    validate(&bytes)?;
    fs::write(state.data_dir.join(PENDING), &bytes).map_err(err)?;
    app.restart()
}

fn validate(bytes: &[u8]) -> CmdResult<()> {
    const NOT: &str = "Ese archivo no es un respaldo de FinderRemoteJob.";
    let mut z = ZipArchive::new(Cursor::new(bytes)).map_err(|_| NOT)?;
    let mut head = [0u8; 16];
    z.by_name(DB)
        .map_err(|_| NOT)?
        .read_exact(&mut head)
        .map_err(|_| NOT)?;
    if &head != b"SQLite format 3\0" {
        return Err(NOT.into());
    }
    Ok(())
}

/// Al arrancar, antes de abrir la base: aplica un respaldo pendiente. Lo actual queda como
/// `finder.db.antes-de-restaurar` y `cvs.antes-de-restaurar`. Si restauró, devuelve la carpeta de
/// origen (vacía si el respaldo no la trae).
pub(crate) fn apply_pending(data_dir: &Path) -> std::io::Result<Option<String>> {
    let pending = data_dir.join(PENDING);
    if !pending.exists() {
        return Ok(None);
    }
    let staging = data_dir.join("restaurando");
    let _ = fs::remove_dir_all(&staging);
    let bytes = fs::read(&pending)?;
    let mut z = ZipArchive::new(Cursor::new(bytes)).map_err(std::io::Error::other)?;
    z.extract(&staging).map_err(std::io::Error::other)?; // `extract` rechaza rutas fuera de `staging`.

    let swap = |name: &str| -> std::io::Result<()> {
        let cur = data_dir.join(name);
        let old = PathBuf::from(format!("{}{OLD}", cur.display()));
        let _ = fs::remove_dir_all(&old);
        let _ = fs::remove_file(&old);
        if cur.exists() {
            fs::rename(&cur, &old)?;
        }
        let new = staging.join(name);
        if new.exists() {
            fs::rename(new, &cur)
        } else {
            Ok(())
        }
    };
    // La base actual ya está cerrada: su WAL va con ella.
    for ext in ["-wal", "-shm"] {
        let wal = data_dir.join(format!("{DB}{ext}"));
        if wal.exists() {
            fs::rename(&wal, data_dir.join(format!("{DB}{OLD}{ext}")))?;
        }
    }
    swap(DB)?;
    swap(CVS)?;
    let origin = fs::read_to_string(staging.join(ORIGIN)).unwrap_or_default();
    fs::remove_dir_all(&staging)?;
    fs::remove_file(&pending)?;
    Ok(Some(origin))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn respaldo_ida_y_vuelta() -> Result<(), Box<dyn std::error::Error>> {
        let root = std::env::temp_dir().join(format!("finder-respaldo-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let (a, b) = (root.join("a"), root.join("b"));
        fs::create_dir_all(a.join("cvs/sub"))?;
        fs::create_dir_all(b.join("cvs"))?;
        let db = root.join("copia.db");
        fs::write(&db, b"SQLite format 3\0resto")?;
        fs::write(a.join("cvs/sub/cv.pdf"), b"pdf")?;
        fs::write(b.join(DB), b"vieja")?;
        fs::write(b.join("cvs/viejo.pdf"), b"viejo")?;

        let zip = root.join("r.zip");
        write_zip(&zip, &db, &a)?;
        let bytes = fs::read(&zip)?;
        validate(&bytes)?;
        assert!(validate(b"no es zip").is_err());

        fs::write(b.join(PENDING), &bytes)?;
        assert_eq!(apply_pending(&b)?, Some(a.display().to_string()));
        assert_eq!(fs::read(b.join(DB))?, b"SQLite format 3\0resto");
        assert_eq!(fs::read(b.join("cvs/sub/cv.pdf"))?, b"pdf");
        assert_eq!(fs::read(b.join(format!("{DB}{OLD}")))?, b"vieja");
        assert!(b.join(format!("cvs{OLD}/viejo.pdf")).exists());
        assert!(!b.join(PENDING).exists());
        assert_eq!(apply_pending(&b)?, None);
        fs::remove_dir_all(&root)?;
        Ok(())
    }

    /// Un respaldo manipulado no debe escribir fuera de la carpeta de datos ni dejar enlaces
    /// que apunten fuera de ella.
    #[test]
    fn respaldo_hostil_no_escapa() -> Result<(), Box<dyn std::error::Error>> {
        let root = std::env::temp_dir().join(format!("finder-hostil-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let data = root.join("datos");
        let outside = root.join("fuera");
        fs::create_dir_all(&data)?;
        fs::create_dir_all(&outside)?;

        let mut z = ZipWriter::new(Cursor::new(Vec::new()));
        let o = SimpleFileOptions::default();
        z.start_file(DB, o)?;
        z.write_all(b"SQLite format 3\0resto")?;
        z.start_file("../escapado.txt", o)?;
        z.write_all(b"x")?;
        z.start_file(format!("{}/absoluto.txt", outside.display()), o)?;
        z.write_all(b"x")?;
        z.add_symlink("cvs/enlace", outside.display().to_string(), o)?;
        let bytes = z.finish()?.into_inner();
        validate(&bytes)?;
        fs::write(data.join(PENDING), &bytes)?;
        let r = apply_pending(&data);

        assert!(!root.join("escapado.txt").exists(), "escapó con ../");
        assert!(
            !outside.join("absoluto.txt").exists(),
            "escapó con ruta absoluta"
        );
        let link = data.join("cvs/enlace");
        assert!(
            fs::symlink_metadata(&link).map_or(true, |m| !m.file_type().is_symlink()),
            "dejó un enlace simbólico hacia fuera ({r:?})"
        );
        fs::remove_dir_all(&root)?;
        Ok(())
    }
}
