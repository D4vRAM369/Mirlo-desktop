#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod backup;
mod db;
mod error;
mod organize;
mod parser;
mod sources;
mod vault;
mod x_url;

use error::{fail, fail_with};
use std::path::{Path, PathBuf};
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

// ---- organizar: cada cambio va a la BD y, después, al bloque "user" de post.json ----

#[tauri::command]
fn set_favorite(app: State<'_, App>, id: String, value: bool) -> Result<(), String> {
    let conn = app.db()?;
    organize::set_favorite(&conn, &id, value)?;
    sync_user(&app, &conn, &[id])
}

#[tauri::command]
fn set_note(app: State<'_, App>, id: String, note: String) -> Result<(), String> {
    let conn = app.db()?;
    organize::set_note(&conn, &id, &note)?;
    sync_user(&app, &conn, &[id])
}

#[tauri::command]
fn set_collection(app: State<'_, App>, id: String, collection_id: Option<i64>) -> Result<(), String> {
    let conn = app.db()?;
    organize::set_collection(&conn, &id, collection_id)?;
    sync_user(&app, &conn, &[id])
}

#[tauri::command]
fn add_tag(app: State<'_, App>, id: String, name: String) -> Result<organize::Tag, String> {
    let conn = app.db()?;
    let tag = organize::add_tag(&conn, &id, &name)?;
    sync_user(&app, &conn, &[id])?;
    Ok(tag)
}

#[tauri::command]
fn remove_tag(app: State<'_, App>, id: String, tag_id: i64) -> Result<(), String> {
    let conn = app.db()?;
    organize::remove_tag(&conn, &id, tag_id)?;
    sync_user(&app, &conn, &[id])
}

#[tauri::command]
fn create_collection(app: State<'_, App>, name: String) -> Result<organize::Collection, String> {
    organize::create_collection(&*app.db()?, &name)
}

#[tauri::command]
fn rename_collection(app: State<'_, App>, id: i64, name: String) -> Result<(), String> {
    let conn = app.db()?;
    let touched = organize::rename_collection(&conn, id, &name)?;
    sync_user(&app, &conn, &touched)
}

#[tauri::command]
fn delete_collection(app: State<'_, App>, id: i64) -> Result<(), String> {
    let conn = app.db()?;
    let touched = organize::delete_collection(&conn, id)?;
    sync_user(&app, &conn, &touched)
}

#[tauri::command]
fn move_collection(app: State<'_, App>, id: i64, position: usize) -> Result<(), String> {
    organize::move_collection(&*app.db()?, id, position)
}

#[tauri::command]
fn rename_tag(app: State<'_, App>, id: i64, name: String) -> Result<(), String> {
    let conn = app.db()?;
    let touched = organize::rename_tag(&conn, id, &name)?;
    sync_user(&app, &conn, &touched)
}

/// El color no va a post.json: es cosa de cómo se ve la etiqueta en esta bóveda, no del post.
#[tauri::command]
fn set_tag_color(app: State<'_, App>, id: i64, color: Option<String>) -> Result<(), String> {
    organize::set_tag_color(&*app.db()?, id, color.as_deref())
}

#[tauri::command]
fn delete_tag(app: State<'_, App>, id: i64) -> Result<(), String> {
    let conn = app.db()?;
    let touched = organize::delete_tag(&conn, id)?;
    sync_user(&app, &conn, &touched)
}

/// Copia a disco los datos de la persona de cada post. La verdad sigue en disco: si la BD se
/// pierde, una importación los recupera de post.json.
/// ponytail: BD y disco no van en la misma transacción; si falla el disco, la BD queda por delante
/// hasta el siguiente cambio de ese post (el error se muestra).
fn sync_user(app: &App, conn: &rusqlite::Connection, ids: &[String]) -> Result<(), String> {
    for id in ids {
        let Some(dir) = db::get(conn, id).map_err(db_err)?.and_then(|d| d.post.dir).map(PathBuf::from) else {
            continue; // post que nunca llegó a guardarse: no tiene post.json
        };
        if !dir.starts_with(&app.root) {
            return Err(fail("outside_vault"));
        }
        vault::write_user(&dir, &organize::user_data(conn, id)?)?;
    }
    Ok(())
}

// ---- copias de seguridad: en un hilo aparte y con su propia conexión, para que la ventana
// y los demás comandos sigan respondiendo mientras se comprime o se descomprime ----

#[tauri::command]
async fn export_vault(app: State<'_, App>, dest: String, with_media: bool) -> Result<backup::ExportSummary, String> {
    let root = app.root.clone();
    blocking(move || backup::export(&db::open(&root.join("mirlo.db")).map_err(db_err)?, &root, Path::new(&dest), with_media)).await
}

#[tauri::command]
async fn import_vault(app: State<'_, App>, src: String) -> Result<backup::ImportSummary, String> {
    let root = app.root.clone();
    blocking(move || backup::import(&db::open(&root.join("mirlo.db")).map_err(db_err)?, &root, Path::new(&src), backup::MAX_IMPORT_BYTES)).await
}

async fn blocking<T: Send + 'static>(work: impl FnOnce() -> Result<T, String> + Send + 'static) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(work).await.map_err(|e| fail_with("internal", e))?
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

/// Icono en la bandeja del sistema. En Linux (AppIndicator) el clic sobre el icono no llega a la
/// app, solo el menú: por eso "Mostrar" va en el menú y el clic izquierdo solo sirve en Windows.
/// ponytail: textos solo en español; traducir si el menú crece.
fn tray(app: &tauri::App) -> tauri::Result<()> {
    use tauri::menu::{Menu, MenuItem};
    use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

    let show = MenuItem::with_id(app, "show", "Mostrar Mirlo", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Salir", true, None::<&str>)?;
    let mut tray = TrayIconBuilder::new()
        .tooltip("Mirlo")
        .menu(&Menu::with_items(app, &[&show, &quit])?)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => show_main(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                show_main(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

fn show_main(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let root = app.path().home_dir()?.join("Mirlo");
            std::fs::create_dir_all(&root)?;
            let conn = db::open(&root.join("mirlo.db"))?;
            app.manage(App { root, db: Mutex::new(conn) });
            tray(app)?;
            Ok(())
        })
        // minimizar = esconderse en la bandeja; se vuelve desde su menú
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Resized(_) = event {
                if window.is_minimized().unwrap_or(false) {
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            save_post, retry_post, list_posts, get_post, facets, open_folder, open_on_x, read_media,
            set_favorite, set_note, set_collection, add_tag, remove_tag, create_collection, rename_collection,
            delete_collection, move_collection, rename_tag, set_tag_color, delete_tag, export_vault, import_vault
        ])
        .run(tauri::generate_context!())
        .expect("error al iniciar Mirlo");
}
