# Llevar a ThreadsVault-desktop lo aprendido en XVault

> Plan para una sesión futura. En esta sesión **no se ha tocado ThreadsVault**: solo se ha leído (versión `1ba92f5`, v2.3.0).

## 1. El fallo que se quiere arreglar

ThreadsVault a veces guarda texto que no es del post, o no saca el texto. La causa está en `src/lib/utils/post-extractor.ts` (1.433 líneas) y en `src-tauri/src/main.rs`:

1. Junta hasta 7 candidatos de texto (oEmbed, JSON incrustado, estado de React, Jina, `og:description`…) y se queda con el que pasa unos filtros heurísticos (`isGenericThreadsText`, `isInvalidExtractedText`, `isThreadPositionNoise`, `isImageAltNoise`).
2. Varios de esos candidatos se sacan con **regex sobre HTML** (`extractJsonValue`, `extractTextFromThreadsJson`, `extractFallbackTextFromSource`, y en Rust `extract_first` / `extract_video_candidate`). Un `"text":"…"` de una respuesta o de la interfaz se parece demasiado al del post.
3. El último recurso (`og:description`) viene truncado y a veces es de otro post del hilo.
4. No hay tests: cada arreglo de un caso rompe otro sin que nadie se entere.

XVault evita esto con cuatro piezas. El orden de abajo es el orden recomendado: cada paso deja ThreadsVault funcionando.

## 2. Pasos, en orden

### Paso 1: fixtures reales y un test que falle (antes de tocar código)

- Guardar en `tests/fixtures/` las respuestas reales de Threads para 8–10 casos: solo texto, 1 foto, carrusel, vídeo, respuesta en hilo, post con enlace, post con emojis, post borrado, post que hoy sale mal.
- Test invariante, como `no_fixture_leaks_tco_links_into_text` de XVault (`src-tauri/src/parser.rs`): recorre **todos** los fixtures y falla si el texto contiene algo que no es del post (texto de otra respuesta, "Log in", "Translate", etc.).
- Sin esto, el resto del plan no se puede verificar.

> Por verificar con los fixtures: qué bloque JSON incrustado contiene el post. ThreadsVault ya lee el estado de React (`extractTextFromThreadsReactState`); el paso 2 debe leer ese mismo JSON entero con un parser, no con regex.

### Paso 2: parser puro en Rust

- Nuevo `src-tauri/src/parser.rs` con `fn parse_post(json: &str, expected_code: &str) -> Result<Post, ParseError>`: sin red ni disco, testeable con `cargo test`.
- Recibe el JSON ya extraído del `<script type="application/json">` (localizar la etiqueta sí es aceptable; leer su contenido con regex no). Lo recorre con `serde_json`, busca el nodo cuyo código coincide con el del post pedido y lee de ahí texto, autor, fecha y medios. **Si el código no coincide, error**: así es imposible coger el texto de otra respuesta.
- Copiar de XVault: estructura de `Post`/`Media`/`ParseError`, `decode_entities`, tests en `parser.rs`.
- Borrar al terminar: las funciones de candidatos y filtros heurísticos de `post-extractor.ts` y `extract_first`, `extract_video_candidate`, `extract_video_candidate_from_embed`, `normalize_html_value` de `main.rs`.

### Paso 3: cadena de fuentes con la fuente apuntada

- Nuevo `src-tauri/src/sources.rs` como el de XVault: fuentes en orden fijo; la primera que da un post válido gana; se guarda `source` y `partial`.
- Si ninguna da texto fiable: error claro con lo que pasó en cada fuente (mensaje de `sources::extract`). Nunca "lo que se pille".
- Mover las peticiones de red del frontend (`tryFetchJson`, `tryFetchText`, Jina) a Rust. Regla de XVault: red y disco solo en Rust.
- `post-extractor.ts` queda como una llamada `invoke('extract_post', { url })`.

### Paso 4: validación de URL y SSRF

- Sustituir `is_supported_threads_url` (regex) por una función como `x_url::status_id`: parsear con `reqwest::Url`, lista blanca exacta de hosts (`threads.net`, `www.threads.net`, `threads.com`, `www.threads.com`) y devolver **solo el código del post**. El backend construye las URLs a partir del código.
- `open_url` hoy abre cualquier cadena que mande el frontend. Cambiarlo por comandos con propósito (`open_on_threads(id)`, `open_folder(id)`) que saquen la ruta o URL de la BD y la validen, como `open_on_x` / `open_folder` en `src-tauri/src/main.rs` de XVault.

### Paso 5: descargas atómicas con SHA-256

- `download_threads_video` y `save_backup` escriben con `std::fs::write` directo: si se corta, queda un archivo roto con su nombre final.
- Copiar de `src-tauri/src/vault.rs`: `fetch_to` (escritura a `.part`, `rename` al terminar, SHA-256 mientras se descarga, 3 intentos solo en 429/5xx/corte), `write_atomic`, `part_path`, lista blanca de hosts de medios (los CDN de Threads/Instagram, a comprobar con los fixtures) y `is_public_ip` para enlaces.
- Copiar el test con servidor local (`download_writes_part_then_renames_with_sha256`, `documents_are_checked_before_downloading`).

### Paso 6 (opcional): búsqueda y estados

- ThreadsVault guarda en SQLite (`plugin-sql`) en PC y en Dexie en Android, con selector. **No migrar a rusqlite**: rompería Android.
- Sí vale la pena añadir en SQLite la tabla FTS5 con triggers de `src-tauri/src/db.rs` (en el esquema de `src/lib/storage/sqlite.ts`) y las columnas `source` y `status` (ok/parcial/error) para mostrar y reintentar.

## 3. Qué NO llevar

- El endpoint de syndication y FxTwitter: son de X.
- `PRAGMA user_version` solo si se añaden columnas en el paso 6.
- El diseño Bandeja: ThreadsVault tiene su propia interfaz.

## 4. Cómo saber que ha funcionado

- `cargo test`: el test invariante pasa con todos los fixtures, incluido el que antes salía mal.
- Ninguna regex en el código lee contenido del post (`grep -n "Regex::new\|new RegExp" src-tauri/src src/lib/utils` solo debe encontrar validaciones, no extracción).
- Un post borrado da un error claro, no un post vacío ni texto de otra cosa.
