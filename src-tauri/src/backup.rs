//! Copias de seguridad: la bóveda entera en un `.zip` y de vuelta.
//! Importar fusiona, nunca sobrescribe: un post que ya está no cambia sus archivos ni su texto,
//! y de los datos de la persona solo se suma (ver `organize::merge_user`).

use crate::db;
use crate::error::{fail, fail_with};
use crate::organize::{self, UserData};
use crate::parser;
use crate::vault;
use rusqlite::Connection;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

/// Tope de lo que puede ocupar una copia ya descomprimida.
/// ponytail: constante; pasa a ajustes si alguien guarda más de 20 GB.
pub const MAX_IMPORT_BYTES: u64 = 20 * 1024 * 1024 * 1024;

#[derive(Debug, PartialEq, Serialize)]
pub struct ExportSummary {
    pub posts: u32,
    pub files: u32,
}

#[derive(Debug, Default, PartialEq, Serialize)]
pub struct ImportSummary {
    pub added: u32,
    pub merged: u32,
    /// Archivos cuyo SHA-256 no coincide con post.json: no se guardan (Reintentar los baja otra vez)
    pub bad_hash: Vec<String>,
    /// "carpeta: código" de los posts que no se pudieron importar
    pub errors: Vec<String>,
}

/// `dest`: el `.zip` que eligió la persona. Se escribe como `.part` y se renombra al acabar.
/// `with_media` = false: solo post.json, post.md y mirlo.db (copia ligera).
pub fn export(conn: &Connection, root: &Path, dest: &Path, with_media: bool) -> Result<ExportSummary, String> {
    let part = vault::part_path(dest);
    let db_copy = dest.with_extension("db.part");
    let _ = std::fs::remove_file(&db_copy); // VACUUM INTO no escribe encima de nada
    let result = (|| {
        // copia consistente de la BD aunque esté abierta; copiar el archivo a pelo podría pillarla a medias
        conn.execute("VACUUM INTO ?1", [db_copy.to_string_lossy()]).map_err(export_err)?;
        let mut zip = ZipWriter::new(File::create(&part).map_err(export_err)?);
        let mut sum = ExportSummary { posts: 0, files: 0 };
        add_file(&mut zip, "mirlo.db", &db_copy, CompressionMethod::Deflated)?;
        for (id, dir) in post_dirs(conn)? {
            let dir = PathBuf::from(dir);
            let Ok(rel) = dir.strip_prefix(root) else { continue }; // fuera de la bóveda: no se exporta
            if !dir.join("post.json").is_file() {
                continue;
            }
            let rel = rel.components().map(|c| c.as_os_str().to_string_lossy()).collect::<Vec<_>>().join("/");
            let mut paths = vec!["post.json".to_string(), "post.md".to_string()];
            if with_media {
                let files = db::get(conn, &id).map_err(export_err)?.map(|d| d.files).unwrap_or_default();
                paths.extend(files.into_iter().filter(|f| f.status == "ok").map(|f| f.path));
            }
            for p in paths {
                let file = dir.join(&p);
                if !is_safe(&p) || !file.is_file() {
                    continue; // lo que falte lo marcará la importación como `not_in_export`
                }
                // fotos y vídeos ya vienen comprimidos: recomprimirlos solo gasta tiempo
                let method = if p.ends_with(".json") || p.ends_with(".md") { CompressionMethod::Deflated } else { CompressionMethod::Stored };
                add_file(&mut zip, &format!("{rel}/{p}"), &file, method)?;
                sum.files += 1;
            }
            sum.posts += 1;
        }
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let manifest = serde_json::json!({
            "mirlo_export": 1,
            "app_version": env!("CARGO_PKG_VERSION"),
            "created_at": parser::iso_from_unix(now as i64),
            "posts": sum.posts,
            "files": sum.files,
            "with_media": with_media,
        });
        zip.start_file("manifest.json", SimpleFileOptions::default()).map_err(export_err)?;
        zip.write_all(&serde_json::to_vec_pretty(&manifest).map_err(export_err)?).map_err(export_err)?;
        zip.finish().map_err(export_err)?.sync_all().map_err(export_err)?;
        std::fs::rename(&part, dest).map_err(export_err)?;
        Ok(sum)
    })();
    let _ = std::fs::remove_file(&db_copy);
    if result.is_err() {
        let _ = std::fs::remove_file(&part);
    }
    result
}

/// Fusiona `src` en la bóveda. Antes de escribir nada se valida todo el zip: manifest, rutas y tamaño.
pub fn import(conn: &Connection, root: &Path, src: &Path, max_bytes: u64) -> Result<ImportSummary, String> {
    let invalid = |e: &dyn std::fmt::Display| fail_with("import_invalid", e);
    let mut zip = ZipArchive::new(File::open(src).map_err(|e| invalid(&e))?).map_err(|e| invalid(&e))?;
    let mut names = HashSet::new();
    let mut total = 0u64;
    for i in 0..zip.len() {
        let f = zip.by_index(i).map_err(|e| invalid(&e))?;
        let name = f.name().map_err(|e| invalid(&e))?.into_owned();
        if !is_safe(&name) {
            return Err(fail_with("import_unsafe_path", name));
        }
        total = total.saturating_add(f.size());
        if !f.is_dir() {
            names.insert(name);
        }
    }
    if total > max_bytes {
        return Err(fail("import_too_big"));
    }
    let manifest: serde_json::Value = serde_json::from_slice(&read_entry(&mut zip, "manifest.json")?).map_err(|e| invalid(&e))?;
    if manifest["mirlo_export"] != 1 {
        return Err(fail("import_invalid"));
    }

    // del mirlo.db exportado solo sale lo que no está en ningún post.json: orden de colecciones y colores
    if names.contains("mirlo.db") {
        let tmp = root.join(".import.db.part");
        if extract(&mut zip, "mirlo.db", &tmp).is_ok() {
            // ponytail: si esa BD no se puede leer, se pierden orden y colores, no posts; no se avisa
            let _ = organize_from(conn, &tmp);
        }
        let _ = std::fs::remove_file(&tmp);
    }

    let mut sum = ImportSummary::default();
    let mut posts: Vec<&String> = names.iter().filter(|n| n.ends_with("/post.json") && n.split('/').count() == 3).collect();
    posts.sort();
    for name in posts {
        let rel = name.trim_end_matches("/post.json");
        if let Err(e) = import_post(conn, root, &mut zip, rel, &names, &mut sum) {
            sum.errors.push(format!("{rel}: {e}"));
        }
    }
    Ok(sum)
}

fn import_post(conn: &Connection, root: &Path, zip: &mut ZipArchive<File>, rel: &str, names: &HashSet<String>, sum: &mut ImportSummary) -> Result<(), String> {
    let mut json: serde_json::Value =
        serde_json::from_slice(&read_entry(zip, &format!("{rel}/post.json"))?).map_err(|e| fail_with("import_invalid", e))?;
    let id = json["post"]["id"].as_str().filter(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit())).ok_or_else(|| fail("import_invalid"))?.to_string();
    let imported: UserData = serde_json::from_value(json["user"].take()).unwrap_or_default();
    let existing = db::get(conn, &id).map_err(db_err)?.and_then(|d| d.post.dir).map(PathBuf::from);
    let (dir, before) = match existing {
        // ya está: sus archivos y su texto no se tocan, solo se suman los datos de la persona
        Some(dir) => {
            sum.merged += 1;
            (dir, organize::user_data(conn, &id)?)
        }
        None => {
            let dir = root.join(rel);
            let mut partial = false;
            for f in json["files"].as_array_mut().into_iter().flatten() {
                if f["status"] != "ok" {
                    continue;
                }
                let path = f["path"].as_str().unwrap_or("").to_string();
                if !is_safe(&path) {
                    return Err(fail_with("import_unsafe_path", format!("{rel}/{path}")));
                }
                let entry = format!("{rel}/{path}");
                let problem = if !names.contains(&entry) {
                    Some("not_in_export")
                } else if extract(zip, &entry, &dir.join(&path))? != f["sha256"].as_str().unwrap_or("") {
                    let _ = std::fs::remove_file(dir.join(&path)); // un archivo que no es el original no se queda
                    sum.bad_hash.push(entry);
                    Some("hash_mismatch")
                } else {
                    None
                };
                if let Some(code) = problem {
                    f["status"] = "error".into();
                    f["error"] = code.into();
                    partial = true;
                }
            }
            if partial {
                json["status"] = "partial".into();
            }
            if names.contains(&format!("{rel}/post.md")) {
                extract(zip, &format!("{rel}/post.md"), &dir.join("post.md"))?;
            }
            json["user"] = serde_json::json!(imported);
            let pretty = serde_json::to_string_pretty(&json).map_err(|e| fail_with("internal", e))?;
            std::fs::create_dir_all(&dir).map_err(|e| fail_with("disk_write", e))?;
            vault::write_atomic(&dir.join("post.json"), pretty.as_bytes())?;
            db::upsert(conn, &vault::load(&dir)?).map_err(db_err)?;
            sum.added += 1;
            (dir, imported.clone())
        }
    };
    organize::apply_user(conn, &id, &organize::merge_user(&organize::user_data(conn, &id)?, &imported))?;
    let after = organize::user_data(conn, &id)?;
    if after != before && dir.starts_with(root) {
        vault::write_user(&dir, &after)?;
    }
    Ok(())
}

/// Colecciones (en su orden) y etiquetas con su color, desde el mirlo.db de un export.
fn organize_from(conn: &Connection, exported: &Path) -> rusqlite::Result<()> {
    let other = Connection::open_with_flags(exported, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let mut stmt = other.prepare("SELECT name FROM collections ORDER BY position, id")?;
    for name in stmt.query_map([], |r| r.get::<_, String>(0))? {
        let _ = organize::ensure_collection(conn, &name?);
    }
    let mut stmt = other.prepare("SELECT name, color FROM tags")?;
    for row in stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?)))? {
        let (name, color) = row?;
        // un color raro no impide crear la etiqueta
        let _ = organize::ensure_tag(conn, &name, color.as_deref()).or_else(|_| organize::ensure_tag(conn, &name, None));
    }
    Ok(())
}

fn post_dirs(conn: &Connection) -> Result<Vec<(String, String)>, String> {
    let mut stmt = conn.prepare("SELECT id, dir FROM posts WHERE dir IS NOT NULL ORDER BY id").map_err(export_err)?;
    let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?))).map_err(export_err)?.collect::<rusqlite::Result<_>>().map_err(export_err);
    rows
}

fn add_file(zip: &mut ZipWriter<File>, name: &str, path: &Path, method: CompressionMethod) -> Result<(), String> {
    let mut file = File::open(path).map_err(export_err)?;
    let big = file.metadata().map_err(export_err)?.len() >= u32::MAX as u64;
    zip.start_file(name, SimpleFileOptions::default().compression_method(method).large_file(big)).map_err(export_err)?;
    std::io::copy(&mut file, zip).map_err(export_err)?;
    Ok(())
}

/// Zip-slip: solo rutas relativas normales ("a/b/c"). Fuera "..", "/x", "C:x" y "\".
fn is_safe(name: &str) -> bool {
    !name.is_empty() && !name.contains(['\\', ':', '\0']) && Path::new(name).components().all(|c| matches!(c, Component::Normal(_)))
}

/// Entrada pequeña (post.json, manifest) a memoria.
fn read_entry(zip: &mut ZipArchive<File>, name: &str) -> Result<Vec<u8>, String> {
    let entry = zip.by_name(name).map_err(|e| fail_with("import_invalid", format!("{name}: {e}")))?;
    let mut bytes = Vec::new();
    entry.take(16 * 1024 * 1024).read_to_end(&mut bytes).map_err(|e| fail_with("import_invalid", format!("{name}: {e}")))?;
    Ok(bytes)
}

/// Escribe una entrada en `dest` (vía `.part`) y devuelve su SHA-256.
fn extract(zip: &mut ZipArchive<File>, name: &str, dest: &Path) -> Result<String, String> {
    let disk = |e: std::io::Error| fail_with("disk_write", format!("{}: {e}", dest.display()));
    let mut entry = zip.by_name(name).map_err(|e| fail_with("import_invalid", format!("{name}: {e}")))?;
    let declared = entry.size();
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(disk)?;
    }
    let part = vault::part_path(dest);
    let mut out = File::create(&part).map_err(disk)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0; 64 * 1024];
    let mut size = 0u64;
    loop {
        let n = entry.read(&mut buf).map_err(|e| fail_with("import_invalid", format!("{name}: {e}")))?;
        if n == 0 {
            break;
        }
        size += n as u64;
        if size > declared {
            drop(out);
            let _ = std::fs::remove_file(&part);
            return Err(fail("import_too_big")); // el zip mintió sobre su tamaño
        }
        hasher.update(&buf[..n]);
        out.write_all(&buf[..n]).map_err(disk)?;
    }
    out.sync_all().map_err(disk)?;
    std::fs::rename(&part, dest).map_err(disk)?;
    Ok(format!("{:x}", hasher.finalize()))
}

fn export_err(e: impl std::fmt::Display) -> String {
    fail_with("export_failed", e)
}

fn db_err(e: rusqlite::Error) -> String {
    fail_with("database", e)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::Extracted;

    fn tmp(name: &str) -> PathBuf {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/test-tmp/backup").join(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn vault_at(name: &str) -> (PathBuf, Connection) {
        let root = tmp(name).join("Mirlo");
        std::fs::create_dir_all(&root).unwrap();
        let conn = db::open(&root.join("mirlo.db")).unwrap();
        (root, conn)
    }

    fn sha(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
    }

    fn json_at(dir: &Path) -> serde_json::Value {
        serde_json::from_slice(&std::fs::read(dir.join("post.json")).unwrap()).unwrap()
    }

    /// Guarda un post de fixture sin red. Con `photo`, le añade media/01.jpg como si se hubiera bajado.
    fn add_post(root: &Path, conn: &Connection, fixture: &str, photo: Option<&[u8]>) -> PathBuf {
        let path = format!("{}/../tests/fixtures/{fixture}.json", env!("CARGO_MANIFEST_DIR"));
        let post = parser::parse_tweet(&std::fs::read_to_string(path).unwrap()).unwrap();
        let ex = Extracted { post, source: "syndication", partial: false, raw_json: "{}".into() };
        let dir = tauri::async_runtime::block_on(vault::save(root, ex, vault::MAX_DOC_BYTES, "es")).unwrap().dir;
        if let Some(bytes) = photo {
            std::fs::write(dir.join("media/01.jpg"), bytes).unwrap();
            let mut json = json_at(&dir);
            json["files"] = serde_json::json!([{"path": "media/01.jpg", "url": "https://pbs.twimg.com/media/a.jpg", "status": "ok",
                "size": bytes.len(), "sha256": sha(bytes), "error": null, "alt": "una foto"}]);
            std::fs::write(dir.join("post.json"), serde_json::to_vec_pretty(&json).unwrap()).unwrap();
        }
        db::upsert(conn, &vault::load(&dir).unwrap()).unwrap();
        dir
    }

    fn organize_20(conn: &Connection, dir: &Path, collection: &str, tag: &str, note: &str) {
        let c = organize::ensure_collection(conn, collection).unwrap();
        organize::set_collection(conn, "20", Some(c)).unwrap();
        organize::add_tag(conn, "20", tag).unwrap();
        organize::set_note(conn, "20", note).unwrap();
        vault::write_user(dir, &organize::user_data(conn, "20").unwrap()).unwrap();
    }

    fn entries(zip: &Path) -> Vec<String> {
        let z = ZipArchive::new(File::open(zip).unwrap()).unwrap();
        let mut names: Vec<_> = z.file_names().map(|n| n.unwrap().to_string()).collect();
        names.sort();
        names
    }

    /// Zip a mano, con un manifest válido salvo que se pase otro.
    fn crafted(name: &str, files: &[(&str, &[u8])]) -> PathBuf {
        let path = tmp(name).join("hecho-a-mano.zip");
        let mut z = ZipWriter::new(File::create(&path).unwrap());
        let mut all = vec![("manifest.json", &br#"{"mirlo_export":1}"#[..])];
        all.retain(|(n, _)| !files.iter().any(|(f, _)| f == n));
        all.extend_from_slice(files);
        for (n, bytes) in all {
            z.start_file(n, SimpleFileOptions::default()).unwrap();
            z.write_all(bytes).unwrap();
        }
        z.finish().unwrap();
        path
    }

    #[test]
    fn export_then_import_into_an_empty_vault_gives_the_same_vault() {
        let (a, conn_a) = vault_at("roundtrip-a");
        let dir = add_post(&a, &conn_a, "text_only", Some(b"jpeg de verdad"));
        add_post(&a, &conn_a, "reply_mention", None);
        organize::ensure_collection(&conn_a, "Primera vacía").unwrap();
        organize_20(&conn_a, &dir, "Recetas", "pan", "ojo");
        organize::set_favorite(&conn_a, "20", true).unwrap();
        let pan = organize::tags(&conn_a).unwrap()[0].item.id;
        organize::set_tag_color(&conn_a, pan, Some("#123456")).unwrap();
        vault::write_user(&dir, &organize::user_data(&conn_a, "20").unwrap()).unwrap();

        let zip = tmp("roundtrip-out").join("copia.zip");
        assert_eq!(export(&conn_a, &a, &zip, true).unwrap(), ExportSummary { posts: 2, files: 5 });
        assert!(!vault::part_path(&zip).exists(), "sin .part");
        assert!(entries(&zip).contains(&"manifest.json".to_string()) && entries(&zip).contains(&"mirlo.db".to_string()));

        let (b, conn_b) = vault_at("roundtrip-b");
        assert_eq!(import(&conn_b, &b, &zip, MAX_IMPORT_BYTES).unwrap(), ImportSummary { added: 2, ..Default::default() });
        let copy = b.join(dir.strip_prefix(&a).unwrap());
        for f in ["post.json", "post.md", "media/01.jpg"] {
            assert_eq!(std::fs::read(copy.join(f)).unwrap(), std::fs::read(dir.join(f)).unwrap(), "{f}");
        }
        let d = db::get(&conn_b, "20").unwrap().unwrap();
        assert_eq!((d.post.status.as_str(), d.post.dir), ("ok", Some(copy.to_string_lossy().into_owned())));
        assert_eq!((d.files[0].sha256.clone(), d.files[0].alt.as_deref()), (sha(b"jpeg de verdad"), Some("una foto")));
        assert_eq!(organize::user_data(&conn_b, "20").unwrap(), organize::user_data(&conn_a, "20").unwrap());
        assert_eq!(organize::tags(&conn_b).unwrap()[0].item.color.as_deref(), Some("#123456"), "el color viaja en mirlo.db");
        let order: Vec<_> = organize::collections(&conn_b).unwrap().into_iter().map(|c| c.item.name).collect();
        assert_eq!(order, ["Primera vacía", "Recetas"], "también las colecciones vacías, en su orden");
        assert!(db::get(&conn_b, "2027747514321956978").unwrap().is_some());
    }

    #[test]
    fn export_without_media_is_light_and_import_marks_what_is_missing() {
        let (a, conn_a) = vault_at("light-a");
        let dir = add_post(&a, &conn_a, "text_only", Some(b"foto"));
        let zip = tmp("light-out").join("ligera.zip");
        assert_eq!(export(&conn_a, &a, &zip, false).unwrap(), ExportSummary { posts: 1, files: 2 });
        let rel = dir.strip_prefix(&a).unwrap().to_string_lossy().replace('\\', "/");
        assert_eq!(entries(&zip), [format!("{rel}/post.json"), format!("{rel}/post.md"), "manifest.json".into(), "mirlo.db".into()]);

        let (b, conn_b) = vault_at("light-b");
        assert_eq!(import(&conn_b, &b, &zip, MAX_IMPORT_BYTES).unwrap().added, 1);
        let d = db::get(&conn_b, "20").unwrap().unwrap();
        assert_eq!(d.post.status, "partial", "Reintentar baja lo que falta");
        assert_eq!((d.files[0].status.as_str(), d.files[0].error.as_deref()), ("error", Some("not_in_export")));
        let copy = b.join(&rel);
        assert_eq!(json_at(&copy)["files"][0]["error"], "not_in_export", "post.json cuenta lo mismo que la BD");
        assert!(!copy.join("media/01.jpg").exists());
    }

    #[test]
    fn importing_a_post_that_exists_merges_and_never_touches_its_files() {
        let (a, conn_a) = vault_at("merge-a");
        let dir_a = add_post(&a, &conn_a, "text_only", None);
        organize_20(&conn_a, &dir_a, "Recetas", "dulce", "importada");
        organize::set_favorite(&conn_a, "20", true).unwrap();
        vault::write_user(&dir_a, &organize::user_data(&conn_a, "20").unwrap()).unwrap();
        let zip = tmp("merge-out").join("copia.zip");
        export(&conn_a, &a, &zip, true).unwrap();

        let (b, conn_b) = vault_at("merge-b");
        let dir_b = add_post(&b, &conn_b, "text_only", None);
        organize_20(&conn_b, &dir_b, "Mía", "pan", "mía");
        std::fs::write(dir_b.join("post.md"), "mi versión").unwrap();

        assert_eq!(import(&conn_b, &b, &zip, MAX_IMPORT_BYTES).unwrap(), ImportSummary { merged: 1, ..Default::default() });
        assert_eq!(std::fs::read_to_string(dir_b.join("post.md")).unwrap(), "mi versión");
        let want = UserData { collection: Some("Mía".into()), tags: vec!["dulce".into(), "pan".into()], favorite: true, note: "mía".into() };
        assert_eq!(organize::user_data(&conn_b, "20").unwrap(), want);
        assert_eq!(serde_json::from_value::<UserData>(json_at(&dir_b)["user"].clone()).unwrap(), want, "post.json al día");
    }

    #[test]
    fn a_file_with_the_wrong_hash_is_not_kept() {
        let (a, conn_a) = vault_at("hash-a");
        let dir = add_post(&a, &conn_a, "text_only", Some(b"original"));
        std::fs::write(dir.join("media/01.jpg"), b"cambiada").unwrap();
        let zip = tmp("hash-out").join("copia.zip");
        export(&conn_a, &a, &zip, true).unwrap();
        let (b, conn_b) = vault_at("hash-b");
        let rel = dir.strip_prefix(&a).unwrap().to_string_lossy().replace('\\', "/");
        let got = import(&conn_b, &b, &zip, MAX_IMPORT_BYTES).unwrap();
        assert_eq!(got.bad_hash, [format!("{rel}/media/01.jpg")]);
        assert!(!b.join(&rel).join("media/01.jpg").exists());
        let d = db::get(&conn_b, "20").unwrap().unwrap();
        assert_eq!((d.post.status.as_str(), d.files[0].error.as_deref()), ("partial", Some("hash_mismatch")));
    }

    #[test]
    fn zip_slip_is_rejected_before_writing_anything() {
        let (b, conn_b) = vault_at("slip-b");
        for evil in ["../fuera.txt", "/etc/absoluta.txt", "a/../../fuera.txt", "a\\..\\..\\fuera.txt", "C:/fuera.txt"] {
            let zip = crafted("slip", &[("nasa/2026-01-01_1/post.md", b"ok"), (evil, b"malo")]);
            assert_eq!(import(&conn_b, &b, &zip, MAX_IMPORT_BYTES), Err(format!("import_unsafe_path: {evil}")));
            assert!(!b.parent().unwrap().join("fuera.txt").exists());
        }
        assert_eq!(std::fs::read_dir(&b).unwrap().count(), 1, "solo mirlo.db: no se escribió nada");
    }

    #[test]
    fn not_a_mirlo_export_or_too_big_is_refused() {
        let (b, conn_b) = vault_at("invalid-b");
        let not_zip = tmp("invalid").join("hola.zip");
        std::fs::write(&not_zip, "hola").unwrap();
        assert!(import(&conn_b, &b, &not_zip, MAX_IMPORT_BYTES).unwrap_err().starts_with("import_invalid"));
        let no_manifest = crafted("invalid-2", &[("manifest.json", b"{}")]);
        assert!(import(&conn_b, &b, &no_manifest, MAX_IMPORT_BYTES).unwrap_err().starts_with("import_invalid"));
        let future = crafted("invalid-3", &[("manifest.json", br#"{"mirlo_export":2}"#)]);
        assert!(import(&conn_b, &b, &future, MAX_IMPORT_BYTES).unwrap_err().starts_with("import_invalid"));
        let big = crafted("big", &[("nasa/2026-01-01_1/post.md", &[b'x'; 2000])]);
        assert_eq!(import(&conn_b, &b, &big, 1000), Err("import_too_big".into()));
        assert_eq!(std::fs::read_dir(&b).unwrap().count(), 1);
    }
}
