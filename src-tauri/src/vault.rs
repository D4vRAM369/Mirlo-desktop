//! Fase 2: guardar un post en disco.
//! `<raíz>/<usuario>/<YYYY-MM-DD>_<id>/` con post.json, post.md, media/ y attachments/.
//! Todo archivo se escribe como `.part` y se renombra al terminar: un archivo con su nombre final
//! siempre está completo, aunque se corte la luz a mitad de descarga.

use crate::parser::{self, MediaKind, Post};
use crate::sources::{self, Extracted};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::io::Write;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Tamaño máximo de un documento enlazado. ponytail: constante; pasa a ajustes en la fase 3.
pub const MAX_DOC_BYTES: u64 = 50 * 1024 * 1024;
const MEDIA_HOSTS: [&str; 2] = ["pbs.twimg.com", "video.twimg.com"];

#[derive(Debug, Serialize)]
pub struct SavedFile {
    /// Relativa a la carpeta del post, con `/`
    pub path: String,
    pub url: String,
    /// "ok" | "error" | "skipped" (documento demasiado grande)
    pub status: &'static str,
    pub size: u64,
    pub sha256: String,
    pub error: Option<String>,
    /// Texto alternativo que puso el autor en X (solo fotos)
    pub alt: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Saved {
    pub dir: PathBuf,
    /// "ok" | "partial": parcial si falta texto o falló algún archivo (se puede reintentar)
    pub status: &'static str,
    pub source: &'static str,
    pub post: Post,
    pub files: Vec<SavedFile>,
}

#[derive(Debug, PartialEq)]
struct Fetched {
    size: u64,
    sha256: String,
    /// Extensión deducida del Content-Type (solo documentos)
    ext: Option<&'static str>,
}

#[derive(Debug, PartialEq)]
enum Fetch {
    Done(Fetched),
    /// None: el enlace no es un documento (una web), se ignora. Some: documento omitido con motivo.
    Skipped(Option<String>),
    Failed(String),
}

pub async fn save(root: &Path, ex: Extracted, max_doc: u64) -> Result<Saved, String> {
    let dir = post_dir(root, &ex.post);
    std::fs::create_dir_all(dir.join("media")).map_err(|e| format!("no se pudo crear {}: {e}", dir.display()))?;
    let client = client()?;
    let mut files = Vec::new();

    // Medios del post y del citado (q01.jpg…). Vídeo que falla por HTTP → yt-dlp.
    let posts = std::iter::once(("", &ex.post)).chain(ex.post.quoted.as_deref().map(|q| ("q", q)));
    for (prefix, p) in posts {
        let mut video_n = 0;
        for (i, m) in p.media.iter().enumerate() {
            let rel = format!("media/{}", media_name(prefix, i + 1, m.kind, &m.url));
            let dest = dir.join(&rel);
            let mut got = if is_media_url(&m.url) {
                get_file(&client, &m.url, &dest).await
            } else {
                Fetch::Failed("servidor de medios no reconocido".into())
            };
            if m.kind != MediaKind::Photo {
                video_n += 1;
                if matches!(got, Fetch::Failed(_)) {
                    got = ytdlp_download(&p.url, video_n, &dest).await;
                }
            }
            files.extend(record(rel, &m.url, got).map(|f| SavedFile { alt: m.alt_text.clone(), ..f }));
        }
    }

    // Documentos enlazados: decide el Content-Type, no la URL (el PDF de la NTSB acaba en "/pdf").
    for (i, link) in ex.post.links.iter().enumerate() {
        let url = &link.expanded_url;
        if !reqwest::Url::parse(url).is_ok_and(|u| is_public_url(&u)) {
            continue; // enlaces a la red local o raros: ni se tocan
        }
        let part = dir.join(format!("attachments/.{:02}.part", i + 1));
        let got = fetch_to(&client, url, &part, max_doc, true).await;
        let rel = match &got {
            Fetch::Done(f) => {
                let rel = format!("attachments/{}", attachment_name(i + 1, url, f.ext.unwrap_or("bin")));
                if let Err(e) = std::fs::rename(&part, dir.join(&rel)) {
                    files.extend(record(rel, url, Fetch::Failed(e.to_string())));
                    continue;
                }
                rel
            }
            _ => format!("attachments/{}", attachment_name(i + 1, url, "bin")),
        };
        files.extend(record(rel, url, got));
    }

    let status = if ex.partial || files.iter().any(|f| f.status == "error") { "partial" } else { "ok" };
    let raw: serde_json::Value = serde_json::from_str(&ex.raw_json).unwrap_or(serde_json::Value::Null);
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let json = serde_json::json!({
        "mirlo": 1,
        "saved_at": parser::iso_from_unix(now as i64),
        "source": ex.source,
        "status": status,
        "post": &ex.post,
        "files": &files,
        "raw": raw,
    });
    let pretty = serde_json::to_string_pretty(&json).map_err(|e| e.to_string())?;
    write_atomic(&dir.join("post.json"), pretty.as_bytes())?;
    write_atomic(&dir.join("post.md"), render_markdown(&ex.post, &files).as_bytes())?;

    Ok(Saved { dir, status, source: ex.source, post: ex.post, files })
}

/// `<raíz>/<usuario>/<YYYY-MM-DD>_<id>`. El usuario se limpia: nunca puede salir de la raíz.
fn post_dir(root: &Path, post: &Post) -> PathBuf {
    let handle = safe_name(&post.author.handle);
    let date = post
        .created_at
        .as_deref()
        .and_then(|d| d.get(..10))
        .filter(|d| d.bytes().all(|b| b.is_ascii_digit() || b == b'-'))
        .unwrap_or("sin-fecha");
    let id: String = post.id.chars().filter(char::is_ascii_digit).collect();
    root.join(if handle.is_empty() { "_".into() } else { handle }).join(format!("{date}_{id}"))
}

/// Solo letras, dígitos, `_` y `-`: sin `/`, `..` ni caracteres que Windows prohíbe.
fn safe_name(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-').take(60).collect()
}

fn media_name(prefix: &str, n: usize, kind: MediaKind, url: &str) -> String {
    let ext = match kind {
        MediaKind::Video | MediaKind::Gif => "mp4".to_string(),
        MediaKind::Photo => {
            let path = url.split(['?', '#']).next().unwrap_or("");
            let ext = path.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase()).unwrap_or_default();
            if matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "webp" | "gif") { ext } else { "jpg".into() }
        }
    };
    format!("{prefix}{n:02}.{ext}")
}

fn attachment_name(n: usize, url: &str, ext: &str) -> String {
    let last = url.split(['?', '#']).next().unwrap_or("").trim_end_matches('/').rsplit('/').next().unwrap_or("");
    let stem = last.rsplit_once('.').map_or(last, |(s, _)| s);
    let stem = safe_name(stem);
    if stem.is_empty() { format!("{n:02}.{ext}") } else { format!("{n:02}_{stem}.{ext}") }
}

/// Lista blanca de documentos. Una web (text/html) no es un adjunto.
fn doc_ext(content_type: &str) -> Option<&'static str> {
    let ct = content_type.split(';').next().unwrap_or("").trim().to_ascii_lowercase();
    Some(match ct.as_str() {
        "application/pdf" => "pdf",
        "application/epub+zip" => "epub",
        "application/msword" => "doc",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => "docx",
        "application/vnd.ms-excel" => "xls",
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => "xlsx",
        "application/vnd.ms-powerpoint" => "ppt",
        "application/vnd.openxmlformats-officedocument.presentationml.presentation" => "pptx",
        "application/vnd.oasis.opendocument.text" => "odt",
        "text/csv" => "csv",
        _ => return None,
    })
}

fn is_media_url(url: &str) -> bool {
    reqwest::Url::parse(url).is_ok_and(|u| u.scheme() == "https" && MEDIA_HOSTS.contains(&u.host_str().unwrap_or("")))
}

/// Anti-SSRF: un enlace dentro de un post no puede hacernos tocar el router o localhost.
/// ponytail: se resuelve el DNS aquí y otra vez al conectar (rebinding posible); cerrarlo exige
/// conectar a la IP ya comprobada con `ClientBuilder::resolve`.
fn is_public_url(url: &reqwest::Url) -> bool {
    matches!(url.scheme(), "http" | "https")
        && url.socket_addrs(|| None).is_ok_and(|a| !a.is_empty() && a.iter().all(|s| is_public_ip(s.ip())))
}

fn is_public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let [a, b, ..] = v4.octets();
            !(v4.is_private()
                || v4.is_loopback()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.is_broadcast()
                || v4.is_documentation()
                || a == 0
                || (a == 100 && (64..128).contains(&b)) // CGNAT
                || a >= 224) // multicast y reservado
        }
        IpAddr::V6(v6) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                return is_public_ip(IpAddr::V4(v4));
            }
            let first = v6.segments()[0];
            !(v6.is_loopback() || v6.is_unspecified() || v6.is_multicast()
                || (first & 0xfe00) == 0xfc00 // fc00::/7 local única
                || (first & 0xffc0) == 0xfe80) // fe80::/10 enlace local
        }
    }
}

fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent(sources::UA)
        .connect_timeout(Duration::from_secs(15))
        .read_timeout(Duration::from_secs(30))
        // cada salto de una redirección pasa el mismo control que el enlace original
        .redirect(reqwest::redirect::Policy::custom(|a| {
            if a.previous().len() >= 10 {
                a.error("demasiadas redirecciones")
            } else if is_public_url(a.url()) {
                a.follow()
            } else {
                a.stop()
            }
        }))
        .build()
        .map_err(|e| e.to_string())
}

/// Medio con nombre conocido. Si ya existe (reintento), no se vuelve a bajar.
async fn get_file(client: &reqwest::Client, url: &str, dest: &Path) -> Fetch {
    if dest.is_file() {
        return hash_file(dest);
    }
    let part = part_path(dest);
    match fetch_to(client, url, &part, u64::MAX, false).await {
        Fetch::Done(f) => match std::fs::rename(&part, dest) {
            Ok(()) => Fetch::Done(f),
            Err(e) => Fetch::Failed(e.to_string()),
        },
        other => other,
    }
}

/// Descarga a `part` con hasta 3 intentos (esperas de 2 s y 4 s). Solo se reintenta lo que
/// puede arreglarse esperando: conexión caída, 429 y 5xx. Un 404 no mejora por insistir.
async fn fetch_to(client: &reqwest::Client, url: &str, part: &Path, max: u64, doc: bool) -> Fetch {
    let mut last = String::new();
    for attempt in 0..3u32 {
        if attempt > 0 {
            tokio::time::sleep(Duration::from_secs(1 << attempt)).await;
        }
        match fetch_once(client, url, part, max, doc).await {
            Ok(done) => return done,
            Err(e) => last = e,
        }
    }
    Fetch::Failed(last)
}

/// Err = vale la pena reintentar
async fn fetch_once(client: &reqwest::Client, url: &str, part: &Path, max: u64, doc: bool) -> Result<Fetch, String> {
    let mut res = client.get(url).send().await.map_err(|e| format!("sin conexión ({e})"))?;
    let status = res.status();
    if status.is_server_error() || status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return Err(format!("HTTP {status}"));
    }
    if !status.is_success() {
        return Ok(Fetch::Failed(format!("HTTP {status}")));
    }
    let ext = if doc {
        let ct = res.headers().get(reqwest::header::CONTENT_TYPE).and_then(|v| v.to_str().ok()).unwrap_or("");
        match doc_ext(ct) {
            Some(e) => Some(e),
            None => return Ok(Fetch::Skipped(None)),
        }
    } else {
        None
    };
    let too_big = || Fetch::Skipped(Some(format!("supera {} MB", max / 1024 / 1024)));
    if res.content_length().is_some_and(|n| n > max) {
        return Ok(too_big());
    }
    let fail = |e: std::io::Error| Fetch::Failed(e.to_string());
    if let Some(parent) = part.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return Ok(fail(e));
        }
    }
    let mut file = match std::fs::File::create(part) {
        Ok(f) => f,
        Err(e) => return Ok(fail(e)),
    };
    let mut hasher = Sha256::new();
    let mut size = 0u64;
    loop {
        match res.chunk().await {
            Ok(Some(chunk)) => {
                size += chunk.len() as u64;
                if size > max {
                    drop(file);
                    let _ = std::fs::remove_file(part);
                    return Ok(too_big());
                }
                hasher.update(&chunk);
                if let Err(e) = file.write_all(&chunk) {
                    drop(file);
                    let _ = std::fs::remove_file(part);
                    return Ok(fail(e));
                }
            }
            Ok(None) => break,
            Err(e) => {
                drop(file);
                let _ = std::fs::remove_file(part);
                return Err(format!("corte de conexión ({e})"));
            }
        }
    }
    if let Err(e) = file.sync_all() {
        return Ok(fail(e));
    }
    Ok(Fetch::Done(Fetched { size, sha256: format!("{:x}", hasher.finalize()), ext }))
}

/// Respaldo para vídeos: yt-dlp sabe pedirlos aunque el enlace directo caduque o dé 403.
/// `n` es la posición del vídeo entre los vídeos del post (yt-dlp los trata como lista).
async fn ytdlp_download(post_url: &str, n: usize, dest: &Path) -> Fetch {
    let tmp = dest.with_extension("ytdlp.mp4");
    let (url, out) = (post_url.to_string(), tmp.clone());
    let ran = tauri::async_runtime::spawn_blocking(move || {
        std::process::Command::new(sources::ytdlp_binary())
            .args(["--no-warnings", "--no-part", "-f", "best[ext=mp4]/best", "--playlist-items", &n.to_string(), "-o"])
            .arg(out.to_string_lossy().replace('%', "%%")) // -o es una plantilla: % literal se escapa
            .arg(&url)
            .output()
    })
    .await;
    match ran {
        Ok(Ok(o)) if o.status.success() && tmp.is_file() => match std::fs::rename(&tmp, dest) {
            Ok(()) => hash_file(dest),
            Err(e) => Fetch::Failed(e.to_string()),
        },
        _ => {
            let _ = std::fs::remove_file(&tmp);
            Fetch::Failed("ni descarga directa ni yt-dlp".into())
        }
    }
}

fn hash_file(path: &Path) -> Fetch {
    let mut hasher = Sha256::new();
    match std::fs::File::open(path).and_then(|mut f| std::io::copy(&mut f, &mut hasher)) {
        Ok(size) => Fetch::Done(Fetched { size, sha256: format!("{:x}", hasher.finalize()), ext: None }),
        Err(e) => Fetch::Failed(e.to_string()),
    }
}

fn part_path(dest: &Path) -> PathBuf {
    let name = dest.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    dest.with_file_name(format!("{name}.part"))
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let part = part_path(path);
    std::fs::write(&part, bytes)
        .and_then(|()| std::fs::rename(&part, path))
        .map_err(|e| format!("no se pudo escribir {}: {e}", path.display()))
}

/// Fetch → fila del manifiesto. Un enlace que no es documento no deja fila.
fn record(path: String, url: &str, got: Fetch) -> Option<SavedFile> {
    let (status, size, sha256, error) = match got {
        Fetch::Done(f) => ("ok", f.size, f.sha256, None),
        Fetch::Skipped(None) => return None,
        Fetch::Skipped(Some(why)) => ("skipped", 0, String::new(), Some(why)),
        Fetch::Failed(why) => ("error", 0, String::new(), Some(why)),
    };
    Some(SavedFile { path, url: url.to_string(), status, size, sha256, error, alt: None })
}

fn render_markdown(post: &Post, files: &[SavedFile]) -> String {
    let mut md = format!("# {} (@{})\n\n", post.author.name, post.author.handle);
    md += &format!("{} · [Ver en X]({})\n\n", post.created_at.as_deref().unwrap_or("Fecha desconocida"), post.url);
    md += &post.text;
    md += "\n";
    let ok = |prefix: &str| files.iter().filter(move |f| f.status == "ok" && f.path.starts_with(prefix)).collect::<Vec<_>>();
    let media = ok("media/");
    if !media.is_empty() {
        md += "\n## Medios\n\n";
        let alts = post.media.iter().chain(post.quoted.iter().flat_map(|q| q.media.iter()));
        for (f, m) in media.iter().zip(alts) {
            let alt = m.alt_text.as_deref().unwrap_or("").replace(['[', ']', '\n'], " ");
            md += &if m.kind == MediaKind::Photo { format!("![{alt}]({})\n", f.path) } else { format!("- [{}]({})\n", f.path, f.path) };
        }
    }
    if !post.links.is_empty() {
        md += "\n## Enlaces\n\n";
        for l in &post.links {
            md += &format!("- <{}>\n", l.expanded_url);
        }
    }
    let docs = ok("attachments/");
    if !docs.is_empty() {
        md += "\n## Adjuntos\n\n";
        for f in docs {
            md += &format!("- [{}]({}) ({} KB)\n", f.path, f.path, f.size.div_ceil(1024));
        }
    }
    if let Some(q) = &post.quoted {
        md += &format!("\n## Cita a @{}\n\n", q.author.handle);
        for line in q.text.lines() {
            md += &format!("> {line}\n");
        }
        md += &format!(">\n> [Ver en X]({})\n", q.url);
    }
    md
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::net::TcpListener;

    fn post(json: &str) -> Post {
        let path = format!("{}/../tests/fixtures/{json}.json", env!("CARGO_MANIFEST_DIR"));
        parser::parse_tweet(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    fn tmp(name: &str) -> PathBuf {
        // dentro de target/: ignorado por git y se limpia con `cargo clean`
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/test-tmp").join(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Servidor HTTP de usar y tirar: responde cada conexión con la siguiente respuesta de la lista.
    fn serve(responses: Vec<String>) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            for (stream, res) in listener.incoming().zip(responses) {
                let mut stream = stream.unwrap();
                let _ = stream.read(&mut [0; 4096]);
                stream.write_all(res.as_bytes()).unwrap();
            }
        });
        base
    }

    fn http(status: &str, content_type: &str, body: &str) -> String {
        format!("HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len())
    }

    #[test]
    fn folder_is_user_date_and_id() {
        let p = post("photos_4");
        assert_eq!(post_dir(Path::new("/v"), &p), Path::new("/v/NASA/2026-05-04_2051361508181856362"));
    }

    #[test]
    fn hostile_handle_and_missing_date_stay_inside_root() {
        let mut p = post("text_only");
        p.author.handle = "../../etc".into();
        p.created_at = None;
        assert_eq!(post_dir(Path::new("/v"), &p), Path::new("/v/etc/sin-fecha_20"));
        p.author.handle = "..".into();
        assert_eq!(post_dir(Path::new("/v"), &p), Path::new("/v/_/sin-fecha_20"));
    }

    #[test]
    fn media_names_are_numbered_with_safe_extensions() {
        let p = post("photos_4");
        assert_eq!(media_name("", 1, p.media[0].kind, &p.media[0].url), "01.jpg");
        assert_eq!(media_name("q", 2, MediaKind::Gif, "https://video.twimg.com/x.mp4"), "q02.mp4");
        assert_eq!(media_name("", 3, MediaKind::Photo, "https://pbs.twimg.com/media/a.exe?name=orig"), "03.jpg");
        assert_eq!(media_name("", 4, MediaKind::Photo, "https://pbs.twimg.com/media/a.PNG?name=orig"), "04.png");
    }

    #[test]
    fn attachments_are_named_from_url_and_content_type() {
        let ntsb = "https://data.ntsb.gov/carol-repgen/api/Aviation/ReportMain/GenerateNewestReport/202401/pdf";
        assert_eq!(attachment_name(1, ntsb, "pdf"), "01_pdf.pdf");
        assert_eq!(attachment_name(2, "https://a.com/docs/Informe%20final.PDF?x=1", "pdf"), "02_Informe20final.pdf");
        assert_eq!(attachment_name(3, "https://a.com/../../", "pdf"), "03.pdf");
        assert_eq!(doc_ext("application/pdf; charset=binary"), Some("pdf"));
        assert_eq!(doc_ext("text/html; charset=utf-8"), None);
        assert_eq!(doc_ext("application/x-msdownload"), None);
    }

    #[test]
    fn only_twitter_media_hosts_and_public_ips() {
        assert!(is_media_url("https://pbs.twimg.com/media/a.jpg?name=orig"));
        assert!(!is_media_url("https://pbs.twimg.com.evil.com/a.jpg"));
        assert!(!is_media_url("http://pbs.twimg.com/a.jpg"));
        for ip in ["127.0.0.1", "10.0.0.1", "192.168.1.1", "172.16.0.1", "169.254.169.254", "100.64.0.1", "0.0.0.0", "::1", "fc00::1", "fe80::1", "::ffff:127.0.0.1"] {
            assert!(!is_public_ip(ip.parse().unwrap()), "{ip}");
        }
        for ip in ["8.8.8.8", "151.101.1.1", "2606:4700::1111"] {
            assert!(is_public_ip(ip.parse().unwrap()), "{ip}");
        }
        assert!(!is_public_url(&"http://localhost:8080/a.pdf".parse().unwrap()));
        assert!(!is_public_url(&"file:///etc/passwd".parse().unwrap()));
    }

    #[test]
    fn download_writes_part_then_renames_with_sha256() {
        let dir = tmp("ok");
        let base = serve(vec![http("200 OK", "image/jpeg", "abc")]);
        let dest = dir.join("media/01.jpg");
        let got = tauri::async_runtime::block_on(get_file(&reqwest::Client::new(), &format!("{base}/a.jpg"), &dest));
        let want = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"; // sha256("abc")
        assert_eq!(got, Fetch::Done(Fetched { size: 3, sha256: want.into(), ext: None }));
        assert_eq!(std::fs::read(&dest).unwrap(), b"abc");
        assert!(!part_path(&dest).exists(), "no queda el .part");
        // reintento: el archivo ya está, no se pide otra vez (el servidor ya no responde)
        let again = tauri::async_runtime::block_on(get_file(&reqwest::Client::new(), &format!("{base}/a.jpg"), &dest));
        assert!(matches!(again, Fetch::Done(f) if f.sha256 == want));
    }

    #[test]
    fn documents_are_checked_before_downloading() {
        let dir = tmp("docs");
        let part = dir.join("attachments/.01.part");
        let c = reqwest::Client::new();
        let base = serve(vec![
            http("200 OK", "text/html", "<html>"),
            http("200 OK", "application/pdf", "%PDF-1.7 demasiado largo"),
            http("200 OK", "application/pdf", "%PDF"),
            http("404 Not Found", "text/plain", ""),
        ]);
        let run = |max| tauri::async_runtime::block_on(fetch_to(&c, &format!("{base}/x"), &part, max, true));
        assert_eq!(run(10), Fetch::Skipped(None), "una web no es un adjunto");
        assert!(matches!(run(10), Fetch::Skipped(Some(_))), "demasiado grande");
        assert!(!part.exists(), "no queda basura");
        assert!(matches!(run(10), Fetch::Done(Fetched { size: 4, ext: Some("pdf"), .. })));
        assert_eq!(run(10), Fetch::Failed("HTTP 404 Not Found".into()), "sin reintentos ni esperas");
    }

    #[test]
    fn markdown_has_text_media_and_quote() {
        let p = post("quote");
        let files = vec![SavedFile { path: "media/01.mp4".into(), url: String::new(), status: "ok", size: 1, sha256: String::new(), error: None, alt: None }];
        let md = render_markdown(&p, &files);
        assert!(md.starts_with(&format!("# {} (@{})", p.author.name, p.author.handle)));
        assert!(md.contains(&p.text));
        assert!(md.contains("[media/01.mp4](media/01.mp4)"));
        assert!(md.contains("## Cita a @"));
        assert!(!md.contains("t.co/"));
    }

    /// Contra X de verdad: `cargo test live_save -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn live_save_real_posts() {
        let root = tmp("live");
        for id in ["2051361508181856362", "1969515038512926823", "1403463121138851843", "1846679441088696480", "2024282115454419182"] {
            let saved = tauri::async_runtime::block_on(async {
                save(&root, sources::extract(id).await.unwrap(), MAX_DOC_BYTES).await.unwrap()
            });
            println!("{id} {} {} {:?}", saved.status, saved.source, saved.dir);
            for f in &saved.files {
                println!("   {} {} {} {:?}", f.status, f.path, f.size, f.error);
                assert_eq!(f.status, "ok", "{id}: {f:?}");
            }
            assert!(saved.dir.join("post.json").is_file() && saved.dir.join("post.md").is_file());
            assert!(!saved.files.is_empty(), "{id}: sin archivos");
        }
    }

    /// La rama de respaldo de vídeo, forzada: `cargo test live_ytdlp -- --ignored`
    #[test]
    #[ignore]
    fn live_ytdlp_fallback_downloads_video() {
        let dest = tmp("ytdlp").join("media/01.mp4");
        std::fs::create_dir_all(dest.parent().unwrap()).unwrap();
        let got = tauri::async_runtime::block_on(ytdlp_download("https://x.com/rauchg/status/1969515038512926823", 1, &dest));
        assert!(matches!(&got, Fetch::Done(f) if f.size > 100_000 && f.sha256.len() == 64), "{got:?}");
        assert!(!dest.with_extension("ytdlp.mp4").exists(), "no queda el temporal");
        let head = std::fs::read(&dest).unwrap();
        assert_eq!(&head[4..8], b"ftyp", "es un MP4");
    }
}
