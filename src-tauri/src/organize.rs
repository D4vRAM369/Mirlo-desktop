//! Organizar: favorito, colección, etiquetas y nota de cada post.
//! Son datos de la persona, no de X: viven en columnas que `db::upsert` nunca toca y se copian al
//! bloque "user" de post.json (`vault::write_user`), así que importar o reconstruir no los pierde.

use crate::error::{fail, fail_with};
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

/// Paleta fija (además se admite un `#rrggbb` propio). El frontend pinta cada nombre con un token que contrasta en los dos temas (app.css).
pub const COLORS: [&str; 8] = ["red", "orange", "yellow", "green", "teal", "blue", "purple", "pink"];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Collection {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Counted<T> {
    #[serde(flatten)]
    pub item: T,
    pub count: u32,
}

/// Bloque "user" de post.json. Por nombre y no por id: los ids cambian de una bóveda a otra.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UserData {
    pub collection: Option<String>,
    pub tags: Vec<String>,
    pub favorite: bool,
    pub note: String,
}

type R<T> = Result<T, String>;

fn sql(e: rusqlite::Error) -> String {
    fail_with("database", e)
}

/// Nombre repetido (índice UNIQUE de colecciones o etiquetas) → `name_taken`.
fn named(e: rusqlite::Error) -> String {
    match &e {
        rusqlite::Error::SqliteFailure(f, _) if f.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE => fail("name_taken"),
        _ => sql(e),
    }
}

/// Espacios de sobra fuera: "  Mis   recetas " → "Mis recetas".
fn clean_name(name: &str) -> R<String> {
    let name = name.split_whitespace().collect::<Vec<_>>().join(" ");
    if name.is_empty() { Err(fail("name_empty")) } else { Ok(name) }
}

/// UPDATE/DELETE que no tocó ninguna fila = el id no existe.
fn one(n: usize, code: &str) -> R<()> {
    if n == 0 { Err(fail(code)) } else { Ok(()) }
}

fn exists(conn: &Connection, table: &str, id: &dyn rusqlite::ToSql, code: &str) -> R<()> {
    let found = conn.query_row(&format!("SELECT 1 FROM {table} WHERE id = ?1"), [id], |_| Ok(())).optional().map_err(sql)?;
    found.ok_or_else(|| fail(code))
}

fn post_ids(conn: &Connection, sql_text: &str, arg: i64) -> R<Vec<String>> {
    let mut stmt = conn.prepare(sql_text).map_err(sql)?;
    let ids = stmt.query_map([arg], |r| r.get(0)).map_err(sql)?.collect::<rusqlite::Result<_>>().map_err(sql);
    ids
}

/// `tags_text` = nombres de las etiquetas del post, para que la búsqueda (FTS) las encuentre.
/// Se rehace en la misma transacción que el cambio de etiquetas: nunca queda desfasado.
fn refresh_tags_text(conn: &Connection, ids: &[String]) -> R<()> {
    let mut stmt = conn
        .prepare(
            "UPDATE posts SET tags_text = coalesce((SELECT group_concat(t.name, ' ') FROM post_tags pt
             JOIN tags t ON t.id = pt.tag_id WHERE pt.post_id = posts.id), '') WHERE id = ?1",
        )
        .map_err(sql)?;
    for id in ids {
        stmt.execute([id]).map_err(sql)?;
    }
    Ok(())
}

fn to_tag(r: &Row) -> rusqlite::Result<Tag> {
    Ok(Tag { id: r.get(0)?, name: r.get(1)?, color: r.get(2)? })
}

pub fn set_favorite(conn: &Connection, id: &str, on: bool) -> R<()> {
    one(conn.execute("UPDATE posts SET favorite = ?2 WHERE id = ?1", params![id, on]).map_err(sql)?, "not_found")
}

pub fn set_note(conn: &Connection, id: &str, note: &str) -> R<()> {
    one(conn.execute("UPDATE posts SET note = ?2 WHERE id = ?1", params![id, note]).map_err(sql)?, "not_found")
}

pub fn set_collection(conn: &Connection, id: &str, collection: Option<i64>) -> R<()> {
    if let Some(c) = collection {
        exists(conn, "collections", &c, "collection_not_found")?;
    }
    one(conn.execute("UPDATE posts SET collection_id = ?2 WHERE id = ?1", params![id, collection]).map_err(sql)?, "not_found")
}

/// Añade una etiqueta por nombre; si no existe (sin distinguir mayúsculas), la crea.
pub fn add_tag(conn: &Connection, id: &str, name: &str) -> R<Tag> {
    let name = clean_name(name)?;
    let tx = conn.unchecked_transaction().map_err(sql)?;
    exists(&tx, "posts", &id, "not_found")?;
    tx.execute("INSERT INTO tags (name) VALUES (?1) ON CONFLICT(name) DO NOTHING", [&name]).map_err(sql)?;
    let tag = tx.query_row("SELECT id, name, color FROM tags WHERE name = ?1", [&name], to_tag).map_err(sql)?;
    tx.execute("INSERT OR IGNORE INTO post_tags (post_id, tag_id) VALUES (?1, ?2)", params![id, tag.id]).map_err(sql)?;
    refresh_tags_text(&tx, &[id.to_string()])?;
    tx.commit().map_err(sql)?;
    Ok(tag)
}

pub fn remove_tag(conn: &Connection, id: &str, tag: i64) -> R<()> {
    let tx = conn.unchecked_transaction().map_err(sql)?;
    tx.execute("DELETE FROM post_tags WHERE post_id = ?1 AND tag_id = ?2", params![id, tag]).map_err(sql)?;
    refresh_tags_text(&tx, &[id.to_string()])?;
    tx.commit().map_err(sql)
}

/// La nueva va al final de la lista.
pub fn create_collection(conn: &Connection, name: &str) -> R<Collection> {
    let name = clean_name(name)?;
    conn.execute(
        "INSERT INTO collections (name, position) VALUES (?1, (SELECT coalesce(max(position) + 1, 0) FROM collections))",
        [&name],
    )
    .map_err(named)?;
    Ok(Collection { id: conn.last_insert_rowid(), name })
}

/// Devuelve los posts afectados: su post.json lleva el nombre y hay que reescribirlo.
pub fn rename_collection(conn: &Connection, collection: i64, name: &str) -> R<Vec<String>> {
    let name = clean_name(name)?;
    one(conn.execute("UPDATE collections SET name = ?2 WHERE id = ?1", params![collection, name]).map_err(named)?, "collection_not_found")?;
    post_ids(conn, "SELECT id FROM posts WHERE collection_id = ?1", collection)
}

/// Los posts no se borran: se quedan sin colección (ON DELETE SET NULL).
pub fn delete_collection(conn: &Connection, collection: i64) -> R<Vec<String>> {
    let tx = conn.unchecked_transaction().map_err(sql)?;
    let ids = post_ids(&tx, "SELECT id FROM posts WHERE collection_id = ?1", collection)?;
    one(tx.execute("DELETE FROM collections WHERE id = ?1", [collection]).map_err(sql)?, "collection_not_found")?;
    tx.commit().map_err(sql)?;
    Ok(ids)
}

/// Mueve una colección a `position` (0 = primera) y renumera todas: 0, 1, 2…
pub fn move_collection(conn: &Connection, collection: i64, position: usize) -> R<()> {
    let tx = conn.unchecked_transaction().map_err(sql)?;
    let mut order: Vec<i64> = {
        let mut stmt = tx.prepare("SELECT id FROM collections ORDER BY position, id").map_err(sql)?;
        let ids = stmt.query_map([], |r| r.get(0)).map_err(sql)?.collect::<rusqlite::Result<_>>().map_err(sql)?;
        ids
    };
    let from = order.iter().position(|&c| c == collection).ok_or_else(|| fail("collection_not_found"))?;
    order.remove(from);
    order.insert(position.min(order.len()), collection);
    for (i, c) in order.iter().enumerate() {
        tx.execute("UPDATE collections SET position = ?2 WHERE id = ?1", params![c, i as i64]).map_err(sql)?;
    }
    tx.commit().map_err(sql)
}

pub fn rename_tag(conn: &Connection, tag: i64, name: &str) -> R<Vec<String>> {
    let name = clean_name(name)?;
    let tx = conn.unchecked_transaction().map_err(sql)?;
    one(tx.execute("UPDATE tags SET name = ?2 WHERE id = ?1", params![tag, name]).map_err(named)?, "tag_not_found")?;
    let ids = post_ids(&tx, "SELECT post_id FROM post_tags WHERE tag_id = ?1", tag)?;
    refresh_tags_text(&tx, &ids)?;
    tx.commit().map_err(sql)?;
    Ok(ids)
}

/// `None` quita el color. Vale un nombre de la paleta o un `#rrggbb`; nada más, porque el valor
/// acaba en el CSS de la página y un texto libre podría colar otras reglas.
pub fn set_tag_color(conn: &Connection, tag: i64, color: Option<&str>) -> R<()> {
    let color = match color {
        None => None,
        Some(c) if COLORS.contains(&c) => Some(c.to_string()),
        Some(c) if c.len() == 7 && c.starts_with('#') && c[1..].bytes().all(|b| b.is_ascii_hexdigit()) => Some(c.to_ascii_lowercase()),
        Some(_) => return Err(fail("invalid_color")),
    };
    one(conn.execute("UPDATE tags SET color = ?2 WHERE id = ?1", params![tag, color]).map_err(sql)?, "tag_not_found")
}

pub fn delete_tag(conn: &Connection, tag: i64) -> R<Vec<String>> {
    let tx = conn.unchecked_transaction().map_err(sql)?;
    let ids = post_ids(&tx, "SELECT post_id FROM post_tags WHERE tag_id = ?1", tag)?;
    one(tx.execute("DELETE FROM tags WHERE id = ?1", [tag]).map_err(sql)?, "tag_not_found")?;
    refresh_tags_text(&tx, &ids)?;
    tx.commit().map_err(sql)?;
    Ok(ids)
}

pub fn tags_of(conn: &Connection, id: &str) -> rusqlite::Result<Vec<Tag>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.color FROM tags t JOIN post_tags pt ON pt.tag_id = t.id
         WHERE pt.post_id = ?1 ORDER BY t.name COLLATE NOCASE",
    )?;
    let tags = stmt.query_map([id], to_tag)?.collect();
    tags
}

pub fn user_data(conn: &Connection, id: &str) -> R<UserData> {
    let row = conn
        .query_row(
            "SELECT c.name, p.favorite, p.note FROM posts p LEFT JOIN collections c ON c.id = p.collection_id WHERE p.id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()
        .map_err(sql)?;
    let (collection, favorite, note) = row.ok_or_else(|| fail("not_found"))?;
    let tags = tags_of(conn, id).map_err(sql)?.into_iter().map(|t| t.name).collect();
    Ok(UserData { collection, tags, favorite, note })
}

/// Para el sidebar: en su orden manual, con cuántos posts tiene cada una.
pub fn collections(conn: &Connection) -> rusqlite::Result<Vec<Counted<Collection>>> {
    let mut stmt = conn.prepare(
        "SELECT c.id, c.name, count(p.id) FROM collections c LEFT JOIN posts p ON p.collection_id = c.id
         GROUP BY c.id ORDER BY c.position, c.id",
    )?;
    let rows = stmt.query_map([], |r| Ok(Counted { item: Collection { id: r.get(0)?, name: r.get(1)? }, count: r.get(2)? }))?.collect();
    rows
}

pub fn tags(conn: &Connection) -> rusqlite::Result<Vec<Counted<Tag>>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.color, count(pt.post_id) FROM tags t LEFT JOIN post_tags pt ON pt.tag_id = t.id
         GROUP BY t.id ORDER BY t.name COLLATE NOCASE",
    )?;
    let rows = stmt.query_map([], |r| Ok(Counted { item: to_tag(r)?, count: r.get(3)? }))?.collect();
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{self, tests::{db, saved}, Filter};

    const NASA: &str = "2051361508181856362";

    fn two_posts() -> Connection {
        let conn = db();
        db::upsert(&conn, &saved("text_only", "ok", &[])).unwrap(); // id 20
        db::upsert(&conn, &saved("photos_4", "ok", &[])).unwrap();
        conn
    }

    fn ids(conn: &Connection, f: Filter) -> Vec<String> {
        db::list(conn, &f).unwrap().into_iter().map(|p| p.id).collect()
    }

    fn search(conn: &Connection, q: &str) -> Vec<String> {
        ids(conn, Filter { q: Some(q.into()), ..Default::default() })
    }

    #[test]
    fn saving_again_never_touches_what_the_person_added() {
        let conn = two_posts();
        let c = create_collection(&conn, "Recetas").unwrap();
        set_collection(&conn, "20", Some(c.id)).unwrap();
        set_favorite(&conn, "20", true).unwrap();
        set_note(&conn, "20", "probar con masa madre").unwrap();
        add_tag(&conn, "20", "Cocina").unwrap();
        let mut again = saved("text_only", "ok", &[]);
        again.post.text = "texto nuevo".into();
        db::upsert(&conn, &again).unwrap();
        let p = db::get(&conn, "20").unwrap().unwrap().post;
        assert_eq!((p.collection_id, p.favorite, p.note.as_str()), (Some(c.id), true, "probar con masa madre"));
        assert_eq!(tags_of(&conn, "20").unwrap().len(), 1);
        assert_eq!(search(&conn, "masa"), ["20"], "la nota sigue en el índice");
        assert_eq!(search(&conn, "cocina"), ["20"], "las etiquetas también");
        assert_eq!(search(&conn, "nuevo"), ["20"]);
    }

    #[test]
    fn names_are_trimmed_and_unique_ignoring_case() {
        let conn = two_posts();
        assert_eq!(create_collection(&conn, "  Mis   recetas ").unwrap().name, "Mis recetas");
        assert_eq!(create_collection(&conn, "MIS RECETAS"), Err("name_taken".into()));
        assert_eq!(create_collection(&conn, "   "), Err("name_empty".into()));
        let rust = add_tag(&conn, "20", "Rust").unwrap();
        assert_eq!(add_tag(&conn, NASA, "rust").unwrap(), rust, "misma etiqueta, no una nueva");
        assert_eq!(add_tag(&conn, "20", "RUST").unwrap(), rust, "añadirla otra vez no duplica");
        assert_eq!(tags_of(&conn, "20").unwrap(), std::slice::from_ref(&rust));
        let other = add_tag(&conn, "20", "Tauri").unwrap();
        assert_eq!(rename_tag(&conn, other.id, "rust"), Err("name_taken".into()));
        assert_eq!(add_tag(&conn, "20", ""), Err("name_empty".into()));
    }

    #[test]
    fn unknown_ids_give_their_own_error() {
        let conn = two_posts();
        assert_eq!(set_favorite(&conn, "999", true), Err("not_found".into()));
        assert_eq!(add_tag(&conn, "999", "x"), Err("not_found".into()));
        assert_eq!(set_collection(&conn, "20", Some(42)), Err("collection_not_found".into()));
        assert_eq!(rename_collection(&conn, 42, "x"), Err("collection_not_found".into()));
        assert_eq!(delete_collection(&conn, 42), Err("collection_not_found".into()));
        assert_eq!(move_collection(&conn, 42, 0), Err("collection_not_found".into()));
        assert_eq!(rename_tag(&conn, 42, "x"), Err("tag_not_found".into()));
        assert_eq!(delete_tag(&conn, 42), Err("tag_not_found".into()));
        assert_eq!(set_tag_color(&conn, 42, None), Err("tag_not_found".into()));
        let t = add_tag(&conn, "20", "x").unwrap();
        for bad in ["#ff00", "#GGGGGG", "red; background: url(x)", "#ff0000ff", "rojo", ""] {
            assert_eq!(set_tag_color(&conn, t.id, Some(bad)), Err("invalid_color".into()), "{bad}");
        }
        set_tag_color(&conn, t.id, Some("teal")).unwrap();
        assert_eq!(tags_of(&conn, "20").unwrap()[0].color.as_deref(), Some("teal"));
        set_tag_color(&conn, t.id, Some("#FF8800")).unwrap();
        assert_eq!(tags_of(&conn, "20").unwrap()[0].color.as_deref(), Some("#ff8800"), "HEX propio, en minúsculas");
    }

    #[test]
    fn renaming_or_deleting_a_tag_updates_search_and_reports_posts() {
        let conn = two_posts();
        let t = add_tag(&conn, "20", "pendiente").unwrap();
        add_tag(&conn, NASA, "pendiente").unwrap();
        add_tag(&conn, NASA, "espacio").unwrap();
        let mut touched = rename_tag(&conn, t.id, "leído").unwrap();
        touched.sort();
        assert_eq!(touched, ["20", NASA]);
        assert_eq!(search(&conn, "leido").len(), 2);
        assert!(search(&conn, "pendiente").is_empty());
        assert_eq!(delete_tag(&conn, t.id).unwrap().len(), 2);
        assert!(search(&conn, "leido").is_empty());
        assert_eq!(search(&conn, "espacio"), [NASA], "las otras etiquetas siguen");
        remove_tag(&conn, NASA, tags_of(&conn, NASA).unwrap()[0].id).unwrap();
        assert!(tags_of(&conn, NASA).unwrap().is_empty());
        assert!(search(&conn, "espacio").is_empty());
    }

    #[test]
    fn collections_keep_manual_order_and_deleting_one_frees_its_posts() {
        let conn = two_posts();
        let [a, b, c] = ["A", "B", "C"].map(|n| create_collection(&conn, n).unwrap());
        let names = |conn: &Connection| collections(conn).unwrap().into_iter().map(|c| c.item.name).collect::<Vec<_>>();
        assert_eq!(names(&conn), ["A", "B", "C"]);
        move_collection(&conn, c.id, 0).unwrap();
        assert_eq!(names(&conn), ["C", "A", "B"]);
        move_collection(&conn, c.id, 99).unwrap();
        assert_eq!(names(&conn), ["A", "B", "C"], "una posición demasiado grande va al final");
        set_collection(&conn, "20", Some(b.id)).unwrap();
        assert_eq!(rename_collection(&conn, b.id, "Bis").unwrap(), ["20"]);
        assert_eq!(collections(&conn).unwrap()[1], Counted { item: Collection { id: b.id, name: "Bis".into() }, count: 1 });
        assert_eq!(delete_collection(&conn, b.id).unwrap(), ["20"]);
        assert_eq!(db::get(&conn, "20").unwrap().unwrap().post.collection_id, None);
        assert_eq!(names(&conn), ["A", "C"]);
        set_collection(&conn, "20", Some(a.id)).unwrap();
        set_collection(&conn, "20", None).unwrap();
        assert_eq!(db::get(&conn, "20").unwrap().unwrap().post.collection_id, None);
    }

    #[test]
    fn filters_and_counts_by_collection_tag_and_favorite() {
        let conn = two_posts();
        let c = create_collection(&conn, "Espacio").unwrap();
        set_collection(&conn, NASA, Some(c.id)).unwrap();
        let t = add_tag(&conn, "20", "historia").unwrap();
        set_favorite(&conn, "20", true).unwrap();
        assert_eq!(ids(&conn, Filter { collection_id: Some(c.id), ..Default::default() }), [NASA]);
        assert_eq!(ids(&conn, Filter { tag_id: Some(t.id), ..Default::default() }), ["20"]);
        assert_eq!(ids(&conn, Filter { favorite: Some(true), ..Default::default() }), ["20"]);
        let f = db::facets(&conn).unwrap();
        assert_eq!(f.favorites, 1);
        assert_eq!(f.collections[0].count, 1);
        assert_eq!(f.tags, [Counted { item: t, count: 1 }]);
    }

    #[test]
    fn user_data_is_by_name_for_post_json() {
        let conn = two_posts();
        assert_eq!(user_data(&conn, "20").unwrap(), UserData::default());
        let c = create_collection(&conn, "Recetas").unwrap();
        set_collection(&conn, "20", Some(c.id)).unwrap();
        add_tag(&conn, "20", "pan").unwrap();
        add_tag(&conn, "20", "Cocina").unwrap();
        set_favorite(&conn, "20", true).unwrap();
        set_note(&conn, "20", "nota").unwrap();
        let want = UserData { collection: Some("Recetas".into()), tags: vec!["Cocina".into(), "pan".into()], favorite: true, note: "nota".into() };
        assert_eq!(user_data(&conn, "20").unwrap(), want);
        assert_eq!(user_data(&conn, "999"), Err("not_found".into()));
    }
}
