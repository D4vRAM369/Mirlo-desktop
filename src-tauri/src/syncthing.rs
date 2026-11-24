//! Conectar el móvil sin saber Syncthing: Mirlo habla con el Syncthing de este ordenador por su API
//! local (la misma que usa su página web), con la clave de su `config.xml`.
//!
//! Lo único que hace: crear (o reutilizar) la carpeta `~/Mirlo/.inbox` en «Enviar y recibir» con papelera,
//! aceptar el dispositivo que la persona elige y compartirle esa carpeta. En el móvil queda escanear el QR
//! y aceptar la carpeta: Syncthing-Fork no deja que otra app lo haga por ella.

use crate::error::{fail, fail_with};
use serde::Serialize;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// Id de la carpeta si hay que crearla. Si ya existe una con la ruta de la bandeja, se usa esa.
const FOLDER_ID: &str = "mirlo-inbox";

#[derive(Serialize, Clone, PartialEq, Debug)]
pub struct Device {
    pub id: String,
    pub name: String,
}

#[derive(Serialize)]
pub struct Status {
    /// ID de este ordenador: lo que el móvil escanea.
    pub my_id: String,
    /// El mismo ID en QR (PNG que pinta el propio Syncthing).
    pub qr: Vec<u8>,
    /// Dispositivos que ya reciben la bandeja.
    pub phones: Vec<Device>,
    /// Dispositivos que han intentado conectar y esperan a que se les acepte.
    pub pending: Vec<Device>,
}

struct Api {
    base: String,
    key: String,
    http: reqwest::Client,
}

/// Dónde guarda Syncthing su configuración en cada sistema (la primera que exista gana).
fn config_candidates(home: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(state) = std::env::var_os("XDG_STATE_HOME") {
        dirs.push(PathBuf::from(state).join("syncthing"));
    }
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        dirs.push(PathBuf::from(local).join("Syncthing"));
    }
    dirs.push(home.join(".local/state/syncthing"));
    dirs.push(home.join(".config/syncthing"));
    dirs.push(home.join("Library/Application Support/Syncthing"));
    dirs.into_iter().map(|d| d.join("config.xml")).collect()
}

fn between<'a>(text: &'a str, open: &str, close: &str) -> Option<&'a str> {
    let start = text.find(open)? + open.len();
    Some(&text[start..start + text[start..].find(close)?])
}

/// (dirección base, clave) desde el `config.xml`. Sin parser XML: son tres etiquetas fijas.
fn parse_config(xml: &str) -> Option<(String, String)> {
    let gui = between(xml, "<gui ", "</gui>")?;
    let tls = gui.split('>').next()?.contains(r#"tls="true""#);
    let address = between(gui, "<address>", "</address>")?.trim();
    let key = between(xml, "<apikey>", "</apikey>")?.trim();
    if address.starts_with("unix") || key.is_empty() {
        return None; // ponytail: GUI por socket Unix no soportada; avisar si alguien la usa
    }
    // escucha en todas las interfaces → se le habla por la local
    let port = address.rsplit(':').next()?;
    let host = match address.rsplit_once(':')?.0 {
        "0.0.0.0" | "[::]" | "" => "127.0.0.1",
        h => h,
    };
    Some((format!("{}://{host}:{port}", if tls { "https" } else { "http" }), key.to_string()))
}

/// `~/Mirlo/.inbox/` y `/home/x/Mirlo/.inbox` son la misma carpeta.
fn same_path(syncthing: &str, home: &Path, target: &Path) -> bool {
    let p = syncthing.trim_end_matches(['/', '\\']);
    let p = match p.strip_prefix("~") {
        Some(rest) => home.join(rest.trim_start_matches(['/', '\\'])),
        None => PathBuf::from(p),
    };
    p == target
}

impl Api {
    fn find(home: &Path) -> Result<Api, String> {
        let xml = config_candidates(home)
            .iter()
            .find_map(|p| std::fs::read_to_string(p).ok())
            .ok_or_else(|| fail("syncthing_missing"))?;
        let (base, key) = parse_config(&xml).ok_or_else(|| fail("syncthing_missing"))?;
        let http = reqwest::Client::builder()
            // solo habla con 127.0.0.1; con tls="true" Syncthing usa un certificado propio, sin CA
            .danger_accept_invalid_certs(true)
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| fail_with("syncthing_off", e))?;
        Ok(Api { base, key, http })
    }

    async fn send(&self, method: reqwest::Method, path: &str, body: Option<&Value>) -> Result<reqwest::Response, String> {
        let mut req = self.http.request(method, format!("{}{path}", self.base)).header("X-API-Key", &self.key);
        if let Some(b) = body {
            req = req.json(b);
        }
        let res = req.send().await.map_err(|e| fail_with("syncthing_off", e))?;
        if !res.status().is_success() {
            return Err(fail_with("syncthing_refused", res.status()));
        }
        Ok(res)
    }

    async fn get(&self, path: &str) -> Result<Value, String> {
        self.send(reqwest::Method::GET, path, None).await?.json().await.map_err(|e| fail_with("syncthing_refused", e))
    }

    async fn put(&self, path: &str, body: &Value) -> Result<(), String> {
        self.send(reqwest::Method::PUT, path, Some(body)).await.map(drop)
    }

    async fn my_id(&self) -> Result<String, String> {
        Ok(self.get("/rest/system/status").await?["myID"].as_str().unwrap_or_default().to_string())
    }

    /// La carpeta de Syncthing cuya ruta es la bandeja, si existe.
    async fn inbox_folder(&self, home: &Path, inbox: &Path) -> Result<Option<Value>, String> {
        let folders = self.get("/rest/config/folders").await?;
        Ok(folders.as_array().into_iter().flatten().find(|f| f["path"].as_str().is_some_and(|p| same_path(p, home, inbox))).cloned())
    }
}

fn devices_of(folder: &Value) -> Vec<String> {
    folder["devices"].as_array().into_iter().flatten().filter_map(|d| d["deviceID"].as_str().map(String::from)).collect()
}

pub async fn status(home: &Path, inbox: &Path) -> Result<Status, String> {
    let api = Api::find(home)?;
    let my_id = api.my_id().await?;
    let names: Vec<Device> = api
        .get("/rest/config/devices")
        .await?
        .as_array()
        .into_iter()
        .flatten()
        .map(|d| Device { id: d["deviceID"].as_str().unwrap_or_default().into(), name: d["name"].as_str().unwrap_or_default().into() })
        .collect();
    let shared = api.inbox_folder(home, inbox).await?.map(|f| devices_of(&f)).unwrap_or_default();
    let phones = names.iter().filter(|d| d.id != my_id && shared.contains(&d.id)).cloned().collect();
    let pending = api
        .get("/rest/cluster/pending/devices")
        .await?
        .as_object()
        .into_iter()
        .flatten()
        .map(|(id, d)| Device { id: id.clone(), name: d["name"].as_str().unwrap_or_default().into() })
        .collect();
    let qr = api
        .send(reqwest::Method::GET, &format!("/qr/?text={my_id}"), None)
        .await?
        .bytes()
        .await
        .map_err(|e| fail_with("syncthing_refused", e))?
        .to_vec();
    Ok(Status { my_id, qr, phones, pending })
}

/// Acepta el dispositivo (si no lo estaba) y le comparte la bandeja, creándola si no existe.
pub async fn accept(home: &Path, inbox: &Path, device: Device) -> Result<(), String> {
    let api = Api::find(home)?;
    let known = api.get("/rest/config/devices").await?;
    let is_known = known.as_array().into_iter().flatten().any(|d| d["deviceID"] == device.id.as_str());
    // uno que ya existe no se toca: reescribirlo borraría sus direcciones y ajustes
    if !is_known {
        let mut d = api.get("/rest/config/defaults/device").await?;
        d["deviceID"] = json!(device.id);
        d["name"] = json!(device.name);
        api.put(&format!("/rest/config/devices/{}", device.id), &d).await?;
    }
    match api.inbox_folder(home, inbox).await? {
        Some(folder) => {
            let mut devices = devices_of(&folder);
            if !devices.contains(&device.id) {
                devices.push(device.id);
                let id = folder["id"].as_str().unwrap_or_default();
                let list: Vec<Value> = devices.iter().map(|d| json!({ "deviceID": d })).collect();
                api.send(reqwest::Method::PATCH, &format!("/rest/config/folders/{id}"), Some(&json!({ "devices": list }))).await?;
            }
        }
        None => {
            let mut f = api.get("/rest/config/defaults/folder").await?;
            f["id"] = json!(FOLDER_ID);
            f["label"] = json!("Mirlo");
            f["path"] = json!(inbox);
            f["type"] = json!("sendreceive");
            f["versioning"] = json!({ "type": "trashcan", "params": { "cleanoutDays": "30" } });
            f["devices"] = json!([{ "deviceID": api.my_id().await? }, { "deviceID": device.id }]);
            api.put(&format!("/rest/config/folders/{FOLDER_ID}"), &f).await?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_gui_address_and_key() {
        let xml = r#"<configuration><gui enabled="true" tls="false"><address>127.0.0.1:37961</address>
            <apikey>abc</apikey></gui></configuration>"#;
        assert_eq!(parse_config(xml), Some(("http://127.0.0.1:37961".into(), "abc".into())));
        let any = r#"<gui tls="true"><address>0.0.0.0:8384</address><apikey>k</apikey></gui>"#;
        assert_eq!(parse_config(any), Some(("https://127.0.0.1:8384".into(), "k".into())));
        let unix = r#"<gui><address>unix:///run/st.sock</address><apikey>k</apikey></gui>"#;
        assert_eq!(parse_config(unix), None);
    }

    #[test]
    fn tilde_and_trailing_slash_match() {
        let home = Path::new("/home/ana");
        let inbox = Path::new("/home/ana/Mirlo/.inbox");
        assert!(same_path("~/Mirlo/.inbox", home, inbox));
        assert!(same_path("/home/ana/Mirlo/.inbox/", home, inbox));
        assert!(!same_path("~/Mirlo", home, inbox));
    }

    /// Contra el Syncthing real de este equipo, solo lectura: `cargo test live_status -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn live_status() {
        let home = PathBuf::from(std::env::var_os("HOME").unwrap());
        let s = tauri::async_runtime::block_on(status(&home, &home.join("Mirlo/.inbox"))).unwrap();
        assert!(s.qr.starts_with(b"\x89PNG"));
        println!("id {}… móviles {:?} pendientes {:?}", &s.my_id[..7], s.phones, s.pending);
    }
}
