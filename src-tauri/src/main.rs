#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod parser;
mod sources;
mod x_url;

/// Único punto de entrada desde el frontend: valida la URL y lee el post.
#[tauri::command]
async fn extract_post(url: String) -> Result<sources::Extracted, String> {
    let id = x_url::status_id(&url)?;
    sources::extract(&id).await
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![extract_post])
        .run(tauri::generate_context!())
        .expect("error al iniciar XVault");
}
