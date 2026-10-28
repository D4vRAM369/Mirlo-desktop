#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod parser;
mod sources;
mod vault;
mod x_url;

/// Único punto de entrada desde el frontend: valida la URL y lee el post.
#[tauri::command]
async fn extract_post(url: String) -> Result<sources::Extracted, String> {
    let id = x_url::status_id(&url)?;
    sources::extract(&id).await
}

/// Lee el post y lo guarda en ~/XVault con todos sus archivos.
#[tauri::command]
async fn save_post(app: tauri::AppHandle, url: String) -> Result<vault::Saved, String> {
    use tauri::Manager;
    let id = x_url::status_id(&url)?;
    let extracted = sources::extract(&id).await?;
    let root = app.path().home_dir().map_err(|e| e.to_string())?.join("XVault");
    vault::save(&root, extracted, vault::MAX_DOC_BYTES).await
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![extract_post, save_post])
        .run(tauri::generate_context!())
        .expect("error al iniciar XVault");
}
