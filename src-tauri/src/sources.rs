//! Cadena de fuentes: syndication → yt-dlp → oEmbed.
//! Cada una devuelve JSON que interpreta `parser`; aquí solo hay red y procesos.

use crate::parser::{self, ParseError, Post};
use serde::Serialize;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Serialize)]
pub struct Extracted {
    pub post: Post,
    /// "syndication" | "yt-dlp" | "oembed": se guarda en la BD para saber de dónde vino
    pub source: &'static str,
    /// true si falta algo (texto recortado, sin medios, enlaces sin expandir)
    pub partial: bool,
    /// JSON crudo de la fuente, para volver a parsearlo en el futuro (post.json)
    #[serde(skip)]
    pub raw_json: String,
}

pub(crate) const UA: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130 Safari/537.36";

pub async fn extract(id: &str) -> Result<Extracted, String> {
    let client = reqwest::Client::builder()
        .user_agent(UA)
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;
    let mut tried = Vec::new();

    match get(&client, &syndication_url(id)).await {
        Ok(raw) => match parser::parse_tweet(&raw) {
            Ok(mut post) => {
                let mut source = "syndication";
                // X corta los posts largos a ~280: FxTwitter da el texto completo (solo el texto)
                if post.text_truncated {
                    let fx = format!("https://api.fxtwitter.com/status/{id}");
                    if let Ok(text) = get(&client, &fx).await.and_then(|j| parser::parse_fxtwitter_text(&j, id).map_err(|e| describe(&e))) {
                        post.text = text;
                        post.text_truncated = false;
                        source = "syndication+fxtwitter";
                    }
                }
                let partial = post.text_truncated;
                return Ok(Extracted { post, source, partial, raw_json: raw });
            }
            Err(e) => tried.push(format!("syndication: {}", describe(&e))),
        },
        Err(e) => tried.push(format!("syndication: {e}")),
    }

    let post_url = format!("https://x.com/i/status/{id}");
    match ytdlp_json(&post_url).await {
        Ok(raw) => match parser::parse_ytdlp(&raw) {
            Ok(post) => return Ok(Extracted { post, source: "yt-dlp", partial: true, raw_json: raw }),
            Err(e) => tried.push(format!("yt-dlp: {}", describe(&e))),
        },
        Err(e) => tried.push(format!("yt-dlp: {e}")),
    }

    let oembed = format!("https://publish.x.com/oembed?omit_script=true&dnt=true&url={}", encode(&post_url));
    match get(&client, &oembed).await {
        Ok(raw) => match parser::parse_oembed(&raw, id) {
            Ok(post) => return Ok(Extracted { post, source: "oembed", partial: true, raw_json: raw }),
            Err(e) => tried.push(format!("oEmbed: {}", describe(&e))),
        },
        Err(e) => tried.push(format!("oEmbed: {e}")),
    }

    Err(format!("No se pudo leer el post. Puede estar borrado o ser privado. ({})", tried.join("; ")))
}

/// Token del embed oficial: (id / 1e15) · π en base 36 sin ceros ni punto.
/// ponytail: aproximación de Number.toString(36) de JS; hoy X acepta cualquier token no vacío
/// (probado el 2026-10-28). Si empieza a validarlo, comparar con la salida de node y ajustar.
fn syndication_token(id: &str) -> String {
    const DIGITS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let x = id.parse::<f64>().unwrap_or(0.0) / 1e15 * std::f64::consts::PI;
    let (mut int, mut frac) = (x.trunc() as u64, x.fract());
    let mut s = Vec::new();
    while int > 0 {
        s.insert(0, DIGITS[(int % 36) as usize]);
        int /= 36;
    }
    for _ in 0..20 {
        frac *= 36.0;
        s.push(DIGITS[frac.trunc() as usize]);
        frac = frac.fract();
    }
    let token: String = s.into_iter().map(char::from).filter(|&c| c != '0').collect();
    if token.is_empty() { "a".into() } else { token }
}

fn syndication_url(id: &str) -> String {
    format!("https://cdn.syndication.twimg.com/tweet-result?id={id}&token={}&lang=es", syndication_token(id))
}

async fn get(client: &reqwest::Client, url: &str) -> Result<String, String> {
    let res = client.get(url).send().await.map_err(|e| format!("sin conexión ({e})"))?;
    let status = res.status();
    if status == reqwest::StatusCode::NOT_FOUND {
        return Err("no existe (404)".into());
    }
    if !status.is_success() {
        return Err(format!("HTTP {status}"));
    }
    res.text().await.map_err(|e| e.to_string())
}

/// yt-dlp empaquetado junto al ejecutable (sidecar de Tauri) o, si no está, el del PATH.
/// El sidecar se llama `xvault-yt-dlp` para no chocar con el paquete `yt-dlp` del sistema en /usr/bin.
pub fn ytdlp_binary() -> PathBuf {
    let exe = if cfg!(windows) { ".exe" } else { "" };
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|dir| dir.join(format!("xvault-yt-dlp{exe}"))))
        .filter(|p| p.is_file())
        .unwrap_or_else(|| PathBuf::from(format!("yt-dlp{exe}")))
}

async fn ytdlp_json(post_url: &str) -> Result<String, String> {
    let url = post_url.to_string();
    tauri::async_runtime::spawn_blocking(move || {
        let out = std::process::Command::new(ytdlp_binary())
            .args(["--dump-json", "--no-warnings", "--no-playlist", "--socket-timeout", "15", &url])
            .output()
            .map_err(|e| format!("no se pudo ejecutar ({e})"))?;
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            return Err(err.lines().last().unwrap_or("falló").to_string());
        }
        String::from_utf8(out.stdout).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

fn describe(e: &ParseError) -> String {
    match e {
        ParseError::Empty => "respuesta vacía".into(),
        ParseError::Tombstone => "post retirado".into(),
        ParseError::Malformed(m) => format!("formato inesperado ({m})"),
    }
}

/// Codificación mínima para meter una URL dentro de un parámetro de query
fn encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_is_never_empty() {
        for id in ["20", "1585341984679469056", "2051361508181856362"] {
            assert!(!syndication_token(id).is_empty());
        }
    }

    /// Prueba contra X de verdad: `cargo test -- --ignored`
    #[test]
    #[ignore]
    fn live_chain_reads_first_tweet() {
        let got = tauri::async_runtime::block_on(extract("20")).unwrap();
        assert_eq!(got.source, "syndication");
        assert_eq!(got.post.text, "just setting up my twttr");
    }

    #[test]
    #[ignore]
    fn live_long_post_gets_full_text() {
        let got = tauri::async_runtime::block_on(extract("1969515038512926823")).unwrap();
        assert_eq!(got.source, "syndication+fxtwitter");
        assert!(!got.partial);
        assert!(got.post.text.ends_with("shell.turso.tech"), "{}", got.post.text);
    }

    #[test]
    #[ignore]
    fn live_missing_post_is_a_clear_error() {
        let err = tauri::async_runtime::block_on(extract("1")).unwrap_err();
        assert!(err.starts_with("No se pudo leer el post"), "{err}");
    }
}
