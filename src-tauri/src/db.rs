//! Índice SQLite de la bóveda (`~/XVault/xvault.db`).
//! La verdad está en disco (post.json de cada carpeta); esto sirve para buscar y filtrar rápido.

use crate::parser::MediaKind;
use crate::vault::Saved;
use rusqlite::{params, params_from_iter, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};
use std::path::Path;

const SCHEMA: &str = "
PRAGMA foreign_keys = ON;
CREATE TABLE IF NOT EXISTS posts (
    id              TEXT PRIMARY KEY,
    url             TEXT NOT NULL,
    handle          TEXT NOT NULL DEFAULT '',
    author_name     TEXT NOT NULL DEFAULT '',
    created_at      TEXT,
    saved_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    text            TEXT NOT NULL DEFAULT '',
    source          TEXT NOT NULL DEFAULT '',
    status          TEXT NOT NULL CHECK (status IN ('ok', 'partial', 'error')),
    error           TEXT,
    dir             TEXT,
    media_kinds     TEXT NOT NULL DEFAULT '',  -- 'photo,video', con comas para filtrar con LIKE
    has_attachments INTEGER NOT NULL DEFAULT 0,
    quoted_handle   TEXT,
    quoted_text     TEXT
);
CREATE TABLE IF NOT EXISTS files (
    post_id TEXT NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    path    TEXT NOT NULL,
    url     TEXT NOT NULL,
    status  TEXT NOT NULL,
    size    INTEGER NOT NULL,
    sha256  TEXT NOT NULL,
    error   TEXT,
    PRIMARY KEY (post_id, path)
);
-- Índice de texto completo. remove_diacritics: 'cancion' encuentra 'canción'.
CREATE VIRTUAL TABLE IF NOT EXISTS posts_fts USING fts5(
    text, author_name, handle, quoted_text,
    content = 'posts', content_rowid = 'rowid',
    tokenize = 'unicode61 remove_diacritics 2'
);
-- Triggers: el índice se actualiza solo cada vez que cambia la tabla posts.
CREATE TRIGGER IF NOT EXISTS posts_ai AFTER INSERT ON posts BEGIN
    INSERT INTO posts_fts(rowid, text, author_name, handle, quoted_text)
    VALUES (new.rowid, new.text, new.author_name, new.handle, new.quoted_text);
END;
CREATE TRIGGER IF NOT EXISTS posts_ad AFTER DELETE ON posts BEGIN
    INSERT INTO posts_fts(posts_fts, rowid, text, author_name, handle, quoted_text)
    VALUES ('delete', old.rowid, old.text, old.author_name, old.handle, old.quoted_text);
END;
CREATE TRIGGER IF NOT EXISTS posts_au AFTER UPDATE ON posts BEGIN
    INSERT INTO posts_fts(posts_fts, rowid, text, author_name, handle, quoted_text)
    VALUES ('delete', old.rowid, old.text, old.author_name, old.handle, old.quoted_text);
    INSERT INTO posts_fts(rowid, text, author_name, handle, quoted_text)
    VALUES (new.rowid, new.text, new.author_name, new.handle, new.quoted_text);
END;
";

const COLUMNS: &str = "id, url, handle, author_name, created_at, saved_at, text, source, status, error, dir, \
                       media_kinds, has_attachments, quoted_handle, quoted_text";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PostRow {
    pub id: String,
    pub url: String,
    pub handle: String,
    pub author_name: String,
    pub created_at: Option<String>,
    pub saved_at: String,
    pub text: String,
    pub source: String,
    /// "ok" | "partial" | "error"
    pub status: String,
    pub error: Option<String>,
    /// Carpeta absoluta del post (None si nunca se llegó a guardar)
    pub dir: Option<String>,
    pub media_kinds: Vec<String>,
    pub has_attachments: bool,
    pub quoted_handle: Option<String>,
    pub quoted_text: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FileRow {
    pub path: String,
    pub url: String,
    pub status: String,
    pub size: u64,
    pub sha256: String,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Detail {
    pub post: PostRow,
    pub files: Vec<FileRow>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Filter {
    pub q: Option<String>,
    pub handle: Option<String>,
    /// "photo" | "video" | "gif" | "document" | "text"
    pub kind: Option<String>,
    pub status: Option<String>,
    /// YYYY-MM-DD, sobre la fecha del post
    pub from: Option<String>,
    pub to: Option<String>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct AuthorCount {
    pub handle: String,
    pub name: String,
    pub count: u32,
}

#[derive(Debug, Default, Serialize, PartialEq)]
pub struct Facets {
    pub authors: Vec<AuthorCount>,
    pub ok: u32,
    pub partial: u32,
    pub error: u32,
}

pub fn open(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(SCHEMA)?;
    Ok(conn)
}

/// Guarda (o reemplaza) un post descargado y su lista de archivos, en una transacción.
pub fn upsert(conn: &Connection, s: &Saved) -> rusqlite::Result<()> {
    let p = &s.post;
    let mut kinds: Vec<&str> = Vec::new();
    for m in p.media.iter().chain(p.quoted.iter().flat_map(|q| q.media.iter())) {
        let k = match m.kind {
            MediaKind::Photo => "photo",
            MediaKind::Video => "video",
            MediaKind::Gif => "gif",
        };
        if !kinds.contains(&k) {
            kinds.push(k);
        }
    }
    let has_attachments = s.files.iter().any(|f| f.status == "ok" && f.path.starts_with("attachments/"));
    let tx = conn.unchecked_transaction()?;
    // ON CONFLICT … DO UPDATE (y no INSERT OR REPLACE): así salta el trigger de UPDATE y el FTS sigue al día
    tx.execute(
        "INSERT INTO posts (id, url, handle, author_name, created_at, text, source, status, error, dir,
                            media_kinds, has_attachments, quoted_handle, quoted_text)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, NULL, ?9, ?10, ?11, ?12, ?13)
         ON CONFLICT(id) DO UPDATE SET
            url = excluded.url, handle = excluded.handle, author_name = excluded.author_name,
            created_at = excluded.created_at, saved_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
            text = excluded.text, source = excluded.source, status = excluded.status, error = NULL,
            dir = excluded.dir, media_kinds = excluded.media_kinds, has_attachments = excluded.has_attachments,
            quoted_handle = excluded.quoted_handle, quoted_text = excluded.quoted_text",
        params![
            p.id,
            p.url,
            p.author.handle,
            p.author.name,
            p.created_at,
            p.text,
            s.source,
            s.status,
            s.dir.to_string_lossy(),
            kinds.join(","),
            has_attachments,
            p.quoted.as_ref().map(|q| &q.author.handle),
            p.quoted.as_ref().map(|q| &q.text),
        ],
    )?;
    tx.execute("DELETE FROM files WHERE post_id = ?1", [&p.id])?;
    for f in &s.files {
        tx.execute(
            "INSERT INTO files (post_id, path, url, status, size, sha256, error) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![p.id, f.path, f.url, f.status, f.size as i64, f.sha256, f.error],
        )?;
    }
    tx.commit()
}

/// Apunta un intento fallido. Si el post ya estaba guardado, se conserva la copia: solo cambia el mensaje.
pub fn record_error(conn: &Connection, id: &str, url: &str, error: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO posts (id, url, status, error) VALUES (?1, ?2, 'error', ?3)
         ON CONFLICT(id) DO UPDATE SET error = excluded.error",
        params![id, url, error],
    )?;
    Ok(())
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<Detail>> {
    let post = conn
        .query_row(&format!("SELECT {COLUMNS} FROM posts WHERE id = ?1"), [id], to_row)
        .optional()?;
    let Some(post) = post else { return Ok(None) };
    let mut stmt = conn.prepare("SELECT path, url, status, size, sha256, error FROM files WHERE post_id = ?1 ORDER BY path")?;
    let files = stmt
        .query_map([id], |r| {
            Ok(FileRow { path: r.get(0)?, url: r.get(1)?, status: r.get(2)?, size: r.get::<_, i64>(3)? as u64, sha256: r.get(4)?, error: r.get(5)? })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(Some(Detail { post, files }))
}

pub fn list(conn: &Connection, f: &Filter) -> rusqlite::Result<Vec<PostRow>> {
    let mut sql = format!("SELECT {COLUMNS} FROM posts WHERE 1 = 1");
    let mut args: Vec<String> = Vec::new();
    // cada `?` se numera según su posición en args
    fn push(sql: &mut String, args: &mut Vec<String>, clause: &str, value: String) {
        args.push(value);
        *sql += &clause.replace('?', &format!("?{}", args.len()));
    }
    if let Some(q) = f.q.as_deref().and_then(fts_query) {
        push(&mut sql, &mut args, " AND rowid IN (SELECT rowid FROM posts_fts WHERE posts_fts MATCH ?)", q);
    }
    if let Some(h) = f.handle.as_deref().filter(|h| !h.is_empty()) {
        push(&mut sql, &mut args, " AND handle = ?", h.to_string());
    }
    match f.kind.as_deref() {
        Some(k @ ("photo" | "video" | "gif")) => push(&mut sql, &mut args, " AND ',' || media_kinds || ',' LIKE ?", format!("%,{k},%")),
        Some("document") => sql += " AND has_attachments = 1",
        Some("text") => sql += " AND media_kinds = '' AND has_attachments = 0 AND status <> 'error'",
        _ => {}
    }
    if let Some(s) = f.status.as_deref().filter(|s| matches!(*s, "ok" | "partial" | "error")) {
        push(&mut sql, &mut args, " AND status = ?", s.to_string());
    }
    if let Some(d) = f.from.as_deref().filter(|d| !d.is_empty()) {
        push(&mut sql, &mut args, " AND substr(created_at, 1, 10) >= ?", d.to_string());
    }
    if let Some(d) = f.to.as_deref().filter(|d| !d.is_empty()) {
        push(&mut sql, &mut args, " AND substr(created_at, 1, 10) <= ?", d.to_string());
    }
    // ponytail: tope de 500 filas; paginar cuando una bóveda real lo pida
    sql += " ORDER BY saved_at DESC, rowid DESC LIMIT 500";
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_from_iter(args), to_row)?.collect();
    rows
}

pub fn facets(conn: &Connection) -> rusqlite::Result<Facets> {
    let mut out = Facets::default();
    let mut stmt = conn.prepare(
        "SELECT handle, max(author_name), count(*) FROM posts WHERE handle <> ''
         GROUP BY handle ORDER BY count(*) DESC, handle COLLATE NOCASE",
    )?;
    out.authors = stmt
        .query_map([], |r| Ok(AuthorCount { handle: r.get(0)?, name: r.get(1)?, count: r.get(2)? }))?
        .collect::<rusqlite::Result<_>>()?;
    let mut stmt = conn.prepare("SELECT status, count(*) FROM posts GROUP BY status")?;
    for row in stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, u32>(1)?)))? {
        let (status, n) = row?;
        match status.as_str() {
            "ok" => out.ok = n,
            "partial" => out.partial = n,
            _ => out.error = n,
        }
    }
    Ok(out)
}

/// Lo que escribe la persona → consulta FTS5 segura. Cada palabra va entre comillas (así
/// `OR`, `-` o `"` no son sintaxis) y con `*` para que "sqli" encuentre "sqlite".
fn fts_query(q: &str) -> Option<String> {
    let terms: Vec<String> = q.split_whitespace().map(|t| format!("\"{}\"*", t.replace('"', "\"\""))).collect();
    (!terms.is_empty()).then(|| terms.join(" "))
}

fn to_row(r: &Row) -> rusqlite::Result<PostRow> {
    let kinds: String = r.get(11)?;
    Ok(PostRow {
        id: r.get(0)?,
        url: r.get(1)?,
        handle: r.get(2)?,
        author_name: r.get(3)?,
        created_at: r.get(4)?,
        saved_at: r.get(5)?,
        text: r.get(6)?,
        source: r.get(7)?,
        status: r.get(8)?,
        error: r.get(9)?,
        dir: r.get(10)?,
        media_kinds: kinds.split(',').filter(|k| !k.is_empty()).map(String::from).collect(),
        has_attachments: r.get(12)?,
        quoted_handle: r.get(13)?,
        quoted_text: r.get(14)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::vault::SavedFile;
    use std::path::PathBuf;

    fn db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(SCHEMA).unwrap();
        conn
    }

    fn saved(fixture: &str, status: &'static str, files: &[&str]) -> Saved {
        let path = format!("{}/../tests/fixtures/{fixture}.json", env!("CARGO_MANIFEST_DIR"));
        let post = parser::parse_tweet(&std::fs::read_to_string(path).unwrap()).unwrap();
        let files = files
            .iter()
            .map(|p| SavedFile { path: p.to_string(), url: String::new(), status: "ok", size: 1, sha256: "x".into(), error: None })
            .collect();
        Saved { dir: PathBuf::from(format!("/v/{fixture}")), status, source: "syndication", post, files }
    }

    fn ids(conn: &Connection, f: Filter) -> Vec<String> {
        list(conn, &f).unwrap().into_iter().map(|p| p.id).collect()
    }

    fn q(s: &str) -> Filter {
        Filter { q: Some(s.into()), ..Default::default() }
    }

    #[test]
    fn full_text_search_by_prefix_author_and_quote() {
        let conn = db();
        upsert(&conn, &saved("quote", "ok", &["media/01.mp4"])).unwrap();
        upsert(&conn, &saved("photos_4", "ok", &[])).unwrap();
        assert_eq!(ids(&conn, q("sqli")), ["1846679441088696480"], "prefijo");
        assert_eq!(ids(&conn, q("ARTEMIS")), ["2051361508181856362"], "sin distinguir mayúsculas");
        assert_eq!(ids(&conn, q("fractaledmind")), ["1846679441088696480"], "texto del citado");
        assert_eq!(ids(&conn, q("nasa artemis")), ["2051361508181856362"], "todas las palabras");
        assert!(ids(&conn, q("artemis zzz")).is_empty());
    }

    #[test]
    fn search_ignores_accents_and_fts_syntax() {
        let conn = db();
        let mut s = saved("text_only", "ok", &[]);
        s.post.text = "Una canción de Málaga".into();
        upsert(&conn, &s).unwrap();
        assert_eq!(ids(&conn, q("cancion malaga")).len(), 1);
        for hostile in ["\"", "OR", "a OR", "-x", "NEAR(", "*", "col:val"] {
            assert!(list(&conn, &q(hostile)).is_ok(), "{hostile}");
        }
    }

    #[test]
    fn saving_again_replaces_files_and_index() {
        let conn = db();
        let mut s = saved("text_only", "partial", &["media/01.jpg", "media/02.jpg"]);
        upsert(&conn, &s).unwrap();
        s.post.text = "texto nuevo".into();
        s.status = "ok";
        s.files.truncate(1);
        upsert(&conn, &s).unwrap();
        let d = get(&conn, "20").unwrap().unwrap();
        assert_eq!(d.post.status, "ok");
        assert_eq!(d.files.len(), 1);
        assert!(ids(&conn, q("twttr")).is_empty(), "el texto viejo sale del índice");
        assert_eq!(ids(&conn, q("nuevo")), ["20"]);
    }

    #[test]
    fn filters_by_kind_author_status_and_date() {
        let conn = db();
        upsert(&conn, &saved("photos_4", "ok", &[])).unwrap(); // NASA 2026-05-04, fotos
        upsert(&conn, &saved("gif", "ok", &[])).unwrap(); // 2021, gif
        upsert(&conn, &saved("text_only", "partial", &[])).unwrap(); // jack 2006
        upsert(&conn, &saved("pdf_link", "ok", &["attachments/01_pdf.pdf"])).unwrap();
        let kind = |k: &str| ids(&conn, Filter { kind: Some(k.into()), ..Default::default() });
        assert_eq!(kind("photo"), ["2051361508181856362"]);
        assert_eq!(kind("gif"), ["1403463121138851843"]);
        assert_eq!(kind("document"), ["2024282115454419182"]);
        assert_eq!(kind("text"), ["20"], "el del PDF tiene adjunto: no es solo texto");
        assert_eq!(ids(&conn, Filter { handle: Some("jack".into()), ..Default::default() }), ["20"]);
        assert_eq!(ids(&conn, Filter { status: Some("partial".into()), ..Default::default() }), ["20"]);
        let range = Filter { from: Some("2021-01-01".into()), to: Some("2021-12-31".into()), ..Default::default() };
        assert_eq!(ids(&conn, range), ["1403463121138851843"]);
    }

    #[test]
    fn failed_attempt_never_overwrites_a_saved_copy() {
        let conn = db();
        record_error(&conn, "99", "https://x.com/i/status/99", "no existe (404)").unwrap();
        upsert(&conn, &saved("text_only", "ok", &[])).unwrap();
        record_error(&conn, "20", "https://x.com/i/status/20", "sin conexión").unwrap();
        let ok = get(&conn, "20").unwrap().unwrap().post;
        assert_eq!((ok.status.as_str(), ok.error.as_deref()), ("ok", Some("sin conexión")));
        assert!(ok.text.contains("twttr"));
        let f = facets(&conn).unwrap();
        assert_eq!((f.ok, f.partial, f.error), (1, 0, 1));
        assert_eq!(f.authors, [AuthorCount { handle: "jack".into(), name: "jack".into(), count: 1 }]);
    }
}
