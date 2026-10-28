#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod error;
mod parser;
mod sources;
mod vault;
mod x_url;

use error::{fail, fail_with};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{Manager, State};

/// Estado compartido por todos los comandos: la raíz de la bóveda y la conexión a SQLite.
struct App {
    root: PathBuf,
    db: Mutex<rusqlite::Connection>,
}

impl App {
    fn db(&self) -> Result<std::sync::MutexGuard<'_, rusqlite::Connection>, String> {
        self.db.lock().map_err(|_| fail("db_locked"))
    }
}

/// Pegar enlace → leer → descargar → indexar. Si falla, queda apuntado como error para reintentar.
#[tauri::command]
async fn save_post(app: State<'_, App>, url: String, lang: String) -> Result<db::PostRow, String> {
    let id = x_url::status_id(&url)?;
    save_by_id(&app, &id, &lang).await
}

#[tauri::command]
async fn retry_post(app: State<'_, App>, id: String, lang: String) -> Result<db::PostRow, String> {
    if id.is_empty() || id.len() > 20 || !id.bytes().all(|b| b.is_ascii_digit()) {
        return Err(fail("invalid_post_id"));
    }
    save_by_id(&app, &id, &lang).await
}

async fn save_by_id(app: &App, id: &str, lang: &str) -> Result<db::PostRow, String> {
    let lang = if lang == "en" { "en" } else { "es" };
    let result = match sources::extract(id).await {
        Ok(extracted) => vault::save(&app.root, extracted, vault::MAX_DOC_BYTES, lang).await,
        Err(e) => Err(e),
    };
    // el candado se toma después de las esperas de red: nunca se bloquea la BD mientras se descarga
    let conn = app.db()?;
    match result {
        Ok(saved) => db::upsert(&conn, &saved).map_err(db_err)?,
        Err(e) => {
            db::record_error(&conn, id, &format!("https://x.com/i/status/{id}"), &e).map_err(db_err)?;
            return Err(e);
        }
    }
    db::get(&conn, id).map_err(db_err)?.map(|d| d.post).ok_or_else(|| fail("internal"))
}

#[tauri::command]
fn list_posts(app: State<'_, App>, filter: db::Filter) -> Result<Vec<db::PostRow>, String> {
    db::list(&*app.db()?, &filter).map_err(db_err)
}

#[tauri::command]
fn get_post(app: State<'_, App>, id: String) -> Result<Option<db::Detail>, String> {
    db::get(&*app.db()?, &id).map_err(db_err)
}

#[tauri::command]
fn facets(app: State<'_, App>) -> Result<db::Facets, String> {
    db::facets(&*app.db()?).map_err(db_err)
}

/// Abre la carpeta del post en el explorador de archivos del sistema.
#[tauri::command]
fn open_folder(app: State<'_, App>, id: String) -> Result<(), String> {
    let detail = db::get(&*app.db()?, &id).map_err(db_err)?;
    let dir = detail.and_then(|d| d.post.dir).map(PathBuf::from).ok_or_else(|| fail("no_folder"))?;
    // la ruta sale de la BD, pero se comprueba igual: nunca se abre nada fuera de la bóveda
    if !dir.starts_with(&app.root) || !dir.is_dir() {
        return Err(fail("folder_missing"));
    }
    open_with_system(dir.as_os_str())
}

/// Bytes de un vídeo guardado. WebKitGTK (Linux) no reproduce `<video>` servido por el protocolo
/// `asset:`, así que el frontend lo pide aquí y lo reproduce como `blob:`.
/// ponytail: el vídeo entero pasa a memoria; con vídeos de varios cientos de MB, servir por rangos.
#[tauri::command]
fn read_media(app: State<'_, App>, id: String, path: String) -> Result<tauri::ipc::Response, String> {
    let detail = db::get(&*app.db()?, &id).map_err(db_err)?.ok_or_else(|| fail("not_found"))?;
    // solo archivos que Mirlo apuntó para ese post: nada de rutas que lleguen del frontend
    if !detail.files.iter().any(|f| f.path == path && f.status == "ok") {
        return Err(fail("unknown_file"));
    }
    let file = PathBuf::from(detail.post.dir.ok_or_else(|| fail("no_folder"))?).join(&path);
    if !file.starts_with(&app.root) {
        return Err(fail("outside_vault"));
    }
    std::fs::read(&file).map(tauri::ipc::Response::new).map_err(|e| fail_with("read_file", format!("{path}: {e}")))
}

/// Abre el post original en el navegador. La URL sale de la BD y se vuelve a validar.
#[tauri::command]
fn open_on_x(app: State<'_, App>, id: String) -> Result<(), String> {
    let post = db::get(&*app.db()?, &id).map_err(db_err)?.ok_or_else(|| fail("not_found"))?.post;
    x_url::status_id(&post.url)?;
    open_with_system(post.url.as_ref())
}

fn open_with_system(target: &std::ffi::OsStr) -> Result<(), String> {
    let opener = if cfg!(windows) { "explorer" } else if cfg!(target_os = "macos") { "open" } else { "xdg-open" };
    std::process::Command::new(opener).arg(target).spawn().map(|_| ()).map_err(|e| fail_with("open_failed", e))
}

fn db_err(e: rusqlite::Error) -> String {
    fail_with("database", e)
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let root = app.path().home_dir()?.join("Mirlo");
            std::fs::create_dir_all(&root)?;
            let conn = db::open(&root.join("mirlo.db"))?;
            app.manage(App { root, db: Mutex::new(conn) });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            save_post, retry_post, list_posts, get_post, facets, open_folder, open_on_x, read_media
        ])
        .run(tauri::generate_context!())
        .expect("error al iniciar Mirlo");
}
