//! Traducción en línea del texto de un post (y su cita) con el endpoint gratuito de Google que usa su
//! extensión de Chrome (`client=dict-chrome-ex`). Solo se llama cuando la persona pulsa «Traducir».
//! El resultado se guarda en la tabla `translations` para no volver a enviar el texto y poder releerlo
//! sin conexión.
//! Por qué este endpoint y no `translate_a/single?client=gtx`: gtx responde 429 a HTTP/1.1 y el reqwest
//! de Mirlo va sin HTTP/2 (activarlo añade la crate h2). dict-chrome-ex acepta HTTP/1.1.

use crate::error::{fail, fail_with};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::time::Duration;

const ENDPOINT: &str = "https://clients5.google.com/translate_a/t";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Translation {
    /// idioma de destino: "es" | "en"
    pub lang: String,
    /// idioma que Google detectó en el original (p. ej. "ja"); vacío si no hubo texto que traducir
    pub source_lang: String,
    pub text: String,
    pub quoted_text: Option<String>,
}

/// Respuesta con `sl=auto`: `[["texto traducido","idioma_origen"]]` (los `\n` se conservan dentro).
fn parse(json: &str) -> Result<(String, String), String> {
    let v: serde_json::Value = serde_json::from_str(json).map_err(|e| fail_with("translate_failed", e))?;
    let first = v.get(0).ok_or_else(|| fail("translate_failed"))?;
    let (text, source) = match first {
        serde_json::Value::String(t) => (t.as_str(), ""), // forma sin idioma detectado
        _ => (first.get(0).and_then(|t| t.as_str()).unwrap_or_default(), first.get(1).and_then(|s| s.as_str()).unwrap_or_default()),
    };
    if text.is_empty() {
        return Err(fail("translate_failed"));
    }
    Ok((text.to_string(), source.to_string()))
}

pub async fn translate(text: &str, lang: &str) -> Result<(String, String), String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|e| fail_with("internal", e))?;
    // ponytail: todo el post en una petición (probado con 6000 caracteres; los posts largos de X llegan
    // a ~25 000). Si Google rechaza los muy largos, trocear por párrafos aquí.
    let res = client
        .post(ENDPOINT)
        .query(&[("client", "dict-chrome-ex"), ("sl", "auto"), ("tl", lang)])
        .form(&[("q", text)])
        .send()
        .await
        .map_err(|e| fail_with("network", e))?;
    if !res.status().is_success() {
        return Err(fail_with("http", res.status()));
    }
    parse(&res.text().await.map_err(|e| fail_with("network", e))?)
}

pub fn load(conn: &Connection, id: &str, lang: &str) -> rusqlite::Result<Option<Translation>> {
    conn.query_row(
        "SELECT lang, source_lang, text, quoted_text FROM translations WHERE post_id = ?1 AND lang = ?2",
        params![id, lang],
        |r| Ok(Translation { lang: r.get(0)?, source_lang: r.get(1)?, text: r.get(2)?, quoted_text: r.get(3)? }),
    )
    .optional()
}

pub fn store(conn: &Connection, id: &str, t: &Translation) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO translations (post_id, lang, source_lang, text, quoted_text) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, t.lang, t.source_lang, t.text, t.quoted_text],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{self, tests::{db, saved}};

    #[test]
    fn parses_text_and_source_lang() {
        assert_eq!(parse(r#"[["Hola\n\nmundo","en"]]"#), Ok(("Hola\n\nmundo".into(), "en".into())));
        assert_eq!(parse(r#"["Hola"]"#), Ok(("Hola".into(), "".into())));
    }

    #[test]
    fn bad_response_is_an_error() {
        for bad in ["[]", "x", "[[]]", "[[\"\",\"en\"]]", "{}"] {
            assert!(parse(bad).unwrap_err().starts_with("translate_failed"), "{bad}");
        }
    }

    /// Con red: `cargo test live_translates -- --ignored`
    #[test]
    #[ignore]
    fn live_translates() {
        let (text, source) = tauri::async_runtime::block_on(translate("Good morning\nfriends", "es")).unwrap();
        assert_eq!(source, "en");
        assert!(text.contains('\n'), "{text}");
    }

    #[test]
    fn store_load_survives_resave_and_cascades() {
        let conn = db();
        let s = saved("text_only", "ok", &[]);
        db::upsert(&conn, &s).unwrap();
        let id = s.post.id.clone();
        let t = Translation { lang: "es".into(), source_lang: "en".into(), text: "".into(), quoted_text: Some("cita".into()) };
        store(&conn, &id, &t).unwrap();
        assert_eq!(load(&conn, &id, "es").unwrap(), Some(t.clone()));
        assert_eq!(load(&conn, &id, "en").unwrap(), None);

        db::upsert(&conn, &s).unwrap(); // re-guardar el post no borra su traducción
        assert_eq!(load(&conn, &id, "es").unwrap(), Some(t));

        conn.execute("DELETE FROM posts WHERE id = ?1", [&id]).unwrap();
        assert_eq!(load(&conn, &id, "es").unwrap(), None);
    }
}
