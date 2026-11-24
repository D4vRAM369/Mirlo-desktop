//! Bandeja de entrada: posts que llegan como archivos en `~/Mirlo/.inbox/`, p. ej. desde el móvil
//! por Syncthing. Cada envío es un archivo `<ID>.mirlo`; solo cuenta el nombre, no el contenido.
//!
//! Va con punto delante porque en la raíz de la bóveda cada carpeta es un usuario de X, y
//! "inbox" podría ser uno. Un nombre de usuario de X no puede empezar por punto.
//!
//! Un archivo por envío y no una lista: dos dispositivos escribiendo a la vez nunca chocan, y al
//! borrar el archivo Syncthing lo borra también en el móvil, así la cola se vacía en los dos lados.

use std::path::{Path, PathBuf};

pub const DIR: &str = ".inbox";
const EXT: &str = ".mirlo";

/// `"2051361508181856362.mirlo"` → el ID. Cualquier otra cosa (temporales de Syncthing
/// `.syncthing.*.tmp` / `~syncthing~*.tmp`, copias en conflicto, rutas raras) → None.
pub fn id_from_name(name: &str) -> Option<String> {
    let id = name.strip_suffix(EXT)?;
    if id.is_empty() || id.len() > 20 || !id.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some(id.to_string())
}

/// Los envíos que esperan, del más antiguo al más nuevo (los IDs de X crecen con el tiempo).
/// Solo archivos normales: un enlace simbólico o una carpeta con nombre de envío se ignoran.
pub fn pending(dir: &Path) -> Vec<(String, PathBuf)> {
    let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut found: Vec<(String, PathBuf)> = entries
        .flatten()
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .filter_map(|e| Some((id_from_name(e.file_name().to_str()?)?, e.path())))
        .collect();
    found.sort_by(|(a, _), (b, _)| a.len().cmp(&b.len()).then_with(|| a.cmp(b)));
    found
}

/// Pasa a `accept` cada envío en orden. `accept` dice si el ID entró en la cola (o ya estaba):
/// entonces se borra el archivo. Si devuelve false la cola está llena: se para y el resto espera
/// a la siguiente pasada. Devuelve cuántos entraron.
pub fn drain(dir: &Path, mut accept: impl FnMut(&str) -> bool) -> usize {
    let mut taken = 0;
    for (id, path) in pending(dir) {
        if !accept(&id) {
            break;
        }
        // si no se puede borrar se volverá a leer; la cola ignora IDs repetidos
        let _ = std::fs::remove_file(&path);
        taken += 1;
    }
    taken
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("mirlo-inbox-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn only_digit_names_with_the_extension_count() {
        assert_eq!(id_from_name("20.mirlo"), Some("20".into()));
        assert_eq!(id_from_name("2051361508181856362.mirlo"), Some("2051361508181856362".into()));
        for name in [
            ".mirlo",
            "20",
            "20.txt",
            "20.mirlo.tmp",
            ".syncthing.20.mirlo.tmp",
            "~syncthing~20.mirlo.tmp",
            "20.sync-conflict-20261124-101010-ABCDEFG.mirlo",
            "123456789012345678901.mirlo",
            "../20.mirlo",
            "٢٠.mirlo",
            "20 .mirlo",
        ] {
            assert_eq!(id_from_name(name), None, "{name}");
        }
    }

    #[test]
    fn drain_takes_in_order_deletes_and_stops_when_full() {
        let dir = tmp("drain");
        for name in ["300.mirlo", "1000.mirlo", "20.mirlo", "nota.txt", ".syncthing.5.mirlo.tmp"] {
            std::fs::write(dir.join(name), b"").unwrap();
        }
        std::fs::create_dir(dir.join("7.mirlo")).unwrap(); // carpeta: no es un envío

        let mut queue = Vec::new();
        let taken = drain(&dir, |id| {
            if queue.len() == 2 {
                return false;
            }
            queue.push(id.to_string());
            true
        });
        assert_eq!(taken, 2);
        assert_eq!(queue, ["20", "300"]);
        assert!(!dir.join("20.mirlo").exists() && !dir.join("300.mirlo").exists());
        assert!(dir.join("1000.mirlo").exists(), "con la cola llena el resto espera");
        assert!(dir.join("nota.txt").exists() && dir.join(".syncthing.5.mirlo.tmp").exists());

        assert_eq!(drain(&dir, |_| true), 1);
        assert!(!dir.join("1000.mirlo").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_missing_folder_is_an_empty_inbox() {
        assert_eq!(drain(&std::env::temp_dir().join("mirlo-inbox-no-existe"), |_| true), 0);
    }
}
