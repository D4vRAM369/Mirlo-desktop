-- Esquema de mirlo.db con PRAGMA user_version = 1 (antes de organizar). Solo para el test de migración.
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
    alt     TEXT,
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
