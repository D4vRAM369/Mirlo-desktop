# Arquitectura y decisiones de diseño

## Decisión 1: el contenido solo se lee de JSON, nunca de HTML

**Fecha:** 2026-10-28
**Contexto:** ThreadsVault-desktop extrae el texto con 7 fuentes distintas (oEmbed, estado React, Jina, JSON genérico, `og:description`…) y regex sobre HTML. A veces guarda texto que no es del post.
**Decisión:** XVault lee cada post de una cadena de fuentes estructuradas:

| Prioridad | Fuente | Qué da | Si falla |
|---|---|---|---|
| 1 | Syndication (`tweet-result`) | JSON completo: texto, medios, citado, autor | pasa a la 2 |
| 2 | `yt-dlp --dump-json` | metadatos + vídeo | pasa a la 3 |
| 3 | oEmbed | solo texto → post "parcial" | error claro en la UI |

**Por qué:** En JSON el texto del post está en un campo con nombre (`text`); no hay que adivinar cuál de los cien textos de la página es el bueno.
**Alternativa rechazada:** Mejorar los filtros de ruido de ThreadsVault. Cada filtro tapa un caso y abre otro; el problema está en la fuente, no en el filtro.
**Se guarda qué fuente se usó** (`source` en la base de datos) para saber de dónde vino cada post.

## Decisión 2: red y disco solo en Rust

**Por qué:** El frontend (Svelte) no puede hacer peticiones a X por CORS, y concentrar red y disco en un único sitio facilita validar URLs (evitar SSRF) y nombres de archivo (evitar *path traversal*).

## Decisión 3: arranque del proyecto, docs y yt-dlp (2026-10-28)

| Tema | Elegido | Por qué |
|---|---|---|
| Scaffold | Proyecto Tauri nuevo; de ThreadsVault solo se copia config, i18n y patrones de storage | No arrastrar las 1433 líneas del extractor con su bug |
| `docs/` | Se versiona en git | Material PBL útil para otros aprendices |
| yt-dlp | Empaquetado como *sidecar* (`externalBin` de Tauri) | Funciona sin instalar nada. Coste: actualizar el binario en cada release (yt-dlp cambia a menudo porque X cambia) |

## Decisión 4: dirección visual B "Bandeja" (2026-10-28)

**Elegido:** `mockups/b-bandeja.html`: tres paneles (filtros, lista y lectura), Atkinson Hyperlegible Next, cobalto como único acento, atajos de teclado visibles y estados con forma + palabra (no solo color).
**Por qué:** Es la más legible y la más rápida con teclado para una herramienta que se usa a diario. La tabla "Archivos en disco" enseña lo que de verdad importa en una bóveda: qué está guardado y qué falta.
**Descartadas:** A (Mesa de luz) prioriza los medios y esconde el texto; C (Registro) es bonita para leer pero lenta para gestionar cientos de posts.

## Decisión 5: estructura del backend (fase 1)

```
src-tauri/src/
├── main.rs      # comando Tauri extract_post(url): valida → extrae
├── x_url.rs     # URL pegada → ID numérico (o error). Defensa contra SSRF
├── parser.rs    # PURO: JSON → Post (syndication, yt-dlp, oEmbed). Todos los tests aquí
└── sources.rs   # red y procesos: cadena syndication → yt-dlp → oEmbed
```

Flujo: `pegar URL → x_url::status_id → sources::extract → parser::parse_* → Extracted { post, source, partial }`.

**Por qué separar parser y sources:** El parser no sabe de dónde viene el JSON. Así se prueba con fixtures (rápido, sin red) y las fuentes se pueden cambiar sin tocarlo.
**Sin dependencias nuevas:** fechas ISO con 15 líneas (algoritmo de Hinnant) en lugar de `chrono`; entidades HTML con 5 `replace`.

## Decisión 6: FxTwitter solo para el texto de posts largos (2026-10-28)

- **Problema:** syndication y yt-dlp cortan los posts de más de 280 caracteres.
- **Opciones:** dejarlos como parciales / pedir el texto a FxTwitter (`api.fxtwitter.com`), un servicio de terceros.
- **Elegida:** FxTwitter, pero **solo** cuando `text_truncated` es true y **solo** para el texto. Autor, fecha y medios siguen saliendo de X. Se comprueba que el ID de la respuesta coincide.
- **Coste aceptado:** el ID de cada post largo guardado se envía a un tercero. Si FxTwitter falla, el post queda como parcial, igual que antes.
- **Rastro:** `source = "syndication+fxtwitter"`.

## Decisión 7: cómo se guardan los archivos (fase 2)

- **Escritura atómica:** todo se escribe como `nombre.part` y se renombra al terminar. Un archivo con su nombre final está siempre completo.
- **Reintentos:** hasta 3 intentos con esperas de 2 s y 4 s, solo para conexión caída, 429 y 5xx. Un 404 no se reintenta.
- **Medios:** solo de `pbs.twimg.com` y `video.twimg.com` por https. Si un vídeo falla, se reintenta con yt-dlp.
- **Documentos:** decide el **Content-Type**, no la extensión de la URL (el PDF de la NTSB acaba en `/pdf`). Lista blanca: pdf, epub, doc(x), xls(x), ppt(x), odt, csv. Máximo 50 MB, comprobado antes y durante la descarga.
- **Anti-SSRF en enlaces:** antes de tocar un enlace del post se resuelve su DNS y se rechaza cualquier IP privada, localhost, enlace local, CGNAT o multicast. Cada salto de una redirección pasa el mismo control. Límite conocido: rebinding de DNS (ver comentario `ponytail:` en `vault.rs`).
- **Reintento barato:** un medio que ya existe en disco no se vuelve a bajar; se recalcula su SHA-256.
- **post.json** guarda el post normalizado, la lista de archivos con su SHA-256 y el JSON crudo de la fuente (`raw`) para poder reinterpretarlo en el futuro.

## Decisión 8: SQLite con rusqlite en Rust, no plugin-sql (fase 3)

| | plugin-sql (ThreadsVault) | rusqlite (XVault) |
|---|---|---|
| Quién escribe SQL | el frontend | solo Rust |
| Tests | necesita la app abierta | `cargo test` con BD en memoria |
| Regla "red y disco solo en Rust" | la rompe | la cumple |

- **Elegida:** rusqlite con `bundled` (trae su propio SQLite con FTS5, no depende del del sistema).
- **Dónde:** `~/XVault/xvault.db`. Es un **índice**: la verdad son las carpetas con `post.json`. Va dentro de la bóveda para que copiar `~/XVault` lo lleve todo.
- **Búsqueda:** tabla virtual FTS5 en modo *external content* + triggers que la mantienen al día. `remove_diacritics 2`: "cancion" encuentra "canción".
- **Un fallo nunca pisa una copia buena:** `record_error` solo actualiza el mensaje de error si el post ya estaba guardado.
- **Candado (Mutex):** la conexión se bloquea solo después de las descargas, nunca durante.

## Decisión 9: diseño final Bandeja (fase 4)

- **CSS propio con tokens, sin Tailwind.** El mockup elegido ya era CSS con variables (`--bg`, `--accent`…). Tailwind no aportaba nada y se quitó: una dependencia menos.
- **Fuente empaquetada en local** (`@fontsource-variable/atkinson-hyperlegible-next`, 34 KB). Cargarla de Google Fonts haría que la app llamase a Google cada vez que se abre, y XVault promete no llamar a nadie. La CSP ya no permite fuentes externas.
- **Tema:** sigue al sistema (`prefers-color-scheme`) hasta que eliges uno con el botón; la elección se recuerda en `localStorage`.
- **Estados con forma + palabra** (✓ Completo, ◐ Parcial, ✕ Error), nunca solo color.
- **Texto alternativo de las fotos:** se guarda en `files.alt` (lo pone el autor en X) y lo usa el visor. Migración con `PRAGMA user_version` para BD de la fase 3.
- **Cola sin porcentaje falso:** la barra de "Guardando…" va y viene; no hay progreso real que mostrar y no se inventa uno.

## Decisión 10: yt-dlp como sidecar oficial de Tauri (fase 5)

- **Mecanismo:** `bundle.externalBin = ["binaries/xvault-yt-dlp"]`. Tauri exige el archivo `src-tauri/binaries/xvault-yt-dlp-<target-triple>` y al empaquetar lo deja junto al ejecutable como `xvault-yt-dlp`, que es donde ya lo buscaba `sources::ytdlp_binary()`. ThreadsVault lo metía como recurso y lo buscaba a mano.
- **El binario no va a git** (41 MB). `npm run sidecar` (`scripts/fetch-yt-dlp.mjs`) baja la versión fijada de la release oficial y comprueba su SHA-256 contra `SHA2-256SUMS`. Corre antes de `tauri dev` y `tauri build`; si ya está y el hash cuadra, no descarga.
- **Versión fijada** (`VERSION` en el script). Subirla cuando X rompa yt-dlp.
- **Límite conocido:** la lista de hashes viene de la misma release; protege de descargas corruptas, no de una release comprometida.
- **Sin ffmpeg:** se pide `best[ext=mp4]`, un único archivo con vídeo y audio.

## Decisión 11: icono propio (2026-10-29)

- **Concepto:** un cajón de fichero de biblioteca (archivo) con un post en modo oscuro siendo fichado y un mirlo azul posado encima.
- **Conexión con X sin copiar marcas:** la tarjeta oscura con la fila de acciones (responder, repostear, me gusta) y una "X" con serifa clásica en la etiqueta de latón, inicial del nombre de la app. Ni el logo de X ni el pájaro de Twitter (marcas registradas de X Corp): el mirlo tiene proporciones de mirlo real, cola larga y pico naranja.
- **Proceso:** generado con GPT en cuatro iteraciones (genérico → fichero → pájaro fotográfico → pájaro de juguete → punto intermedio satinado). Esquinas recortadas a transparente y bordes limpiados con ImageMagick; `npx tauri icon app-icon.png` genera todos los tamaños.
- **Fuente:** `app-icon.png` (1024×1024) en la raíz del proyecto.
- **Límite conocido:** a 16 px solo se lee "cajón azul con algo encima". Si molesta, versión simplificada solo para 16 y 32 px.

## Decisión 12: la app se llama Mirlo (2026-10-29)

- **Por qué no XVault:** "X" + "vault" puede leerse como contenido +18 ("XXX"). "TwitterVault" usa una marca registrada de X Corp.
- **Por qué Mirlo:** es el protagonista del icono, corto, en español y sin connotaciones. Guiña al pájaro de Twitter sin usar su marca.
- **La compatibilidad con X va en el subtítulo y la descripción** ("Tu bóveda local de posts de X"): uso descriptivo, como "compatible con iPhone".
- **Qué cambió:** nombre del paquete y del binario (`mirlo`), `productName`, identificador `com.mirlo.desktop`, carpeta `~/Mirlo`, base de datos `mirlo.db`, sidecar `mirlo-yt-dlp`, marca `"mirlo": 1` en `post.json`. Los documentos históricos de Chronicler y los mockups conservan "XVault".
- **Migración de datos (una sola vez, a mano):** `mv ~/XVault ~/Mirlo`, `xvault.db` → `mirlo.db`, `UPDATE posts SET dir = …` para las rutas absolutas y la marca de `post.json`. No se dejó código de migración en la app: solo había una bóveda, la del autor.
- **Banner del README:** compuesto en HTML (`docs/assets/banner.html`) con el icono real y la tipografía de la app, exportado con Chromium a 2560×800. Así el texto sale perfecto, cosa que un generador de imágenes no garantiza.

## Decisión 13: interfaz en español e inglés (i18n) antes de la primera release

- **Errores de Rust como códigos** (`src-tauri/src/error.rs`): `codigo` o `codigo: detalle`. Rust dice qué pasó; el frontend lo cuenta en el idioma de la app. El detalle técnico (HTTP, rutas, errores del sistema) no se traduce y se muestra aparte como "Detalles".
- **Mismo formato en la BD y en post.json:** un error guardado en español se lee en inglés al cambiar de idioma. Un texto antiguo sin código conocido se muestra tal cual (sin migración).
- **Diccionario** en `src/lib/i18n.svelte.ts`: `en` se declara con las claves de `es`, así que TypeScript no compila si falta una traducción.
- **`scripts/check-i18n.mjs`** (dentro de `npm run check`) falla si Rust usa un código de error sin traducción. Probado con un código inventado: falla con salida 1.
- **Plurales** con `Intl.PluralRules`; **fechas y tamaños** con `Intl` en el idioma elegido.
- **Teclas dentro de frases** (`[Ctrl] [V]`) se pintan como `<kbd>` con un snippet, sin `{@html}`: los textos llevan datos del post y `{@html}` permitiría inyectar HTML.
- **Idioma inicial:** el del sistema (`es*` → español, el resto → inglés); se recuerda en `localStorage`. `post.md` se escribe en el idioma de la app al guardar.
- **Por qué antes de publicar:** después habría mensajes en español guardados en la BD de cada usuario.

## Decisión 14: organizar (favoritos, colecciones, etiquetas y notas) (2026-10-28)

| Tema | Elegido | Por qué |
|---|---|---|
| Colección | 0 o 1 por post, como una carpeta, con orden manual | Es lo que se espera de "mover a…"; para clasificar en varios ejes ya están las etiquetas |
| Etiquetas | Muchas por post, nombre único sin distinguir mayúsculas | `Rust` y `rust` son la misma; añadir una que existe la reutiliza |
| Color de etiqueta | 8 tonos de la paleta (cambian con el tema) **o** un `#rrggbb` propio desde una rueda | La paleta asegura contraste en claro y oscuro; la rueda da libertad. Rust solo acepta esas dos formas: el valor acaba en el CSS de la página |
| Búsqueda | `tags_text` (nombres desnormalizados) y `note` entran en el FTS | El índice FTS5 solo lee columnas de `posts`; Rust rehace `tags_text` en la misma transacción que el cambio de etiquetas |
| Dónde vive | Columnas que `db::upsert` no toca + bloque `"user"` en `post.json` | Son datos de la persona, no de X: ni re-guardar ni reintentar los pisa. En disco van por **nombre** (los ids cambian de una bóveda a otra) para que importar no pierda nada |
| Migración | `user_version = 2` en una transacción: columnas nuevas, se tira el índice viejo y se rehace (`rebuild`) | Si falla a mitad, la BD sigue en v1. Probado contra una copia de una bóveda real antes de abrir la app |
| Borrar | Dos pulsaciones ("Borrar" → "Sí, borrar") | No hay papelera; borrar una colección no borra posts (`ON DELETE SET NULL`) |

**Descartado:** colecciones anidadas (más complejidad que valor por ahora) y arrastrar filas a una colección (opcional en la SPEC; el `M` + flechas cubre el caso).
**Código:** `src-tauri/src/organize.rs` (lógica y tests), `db.rs` (esquema y migración), `vault::write_user`, `src/lib/Organizer.svelte`, `NameEditor.svelte`, `ColorWheel.svelte`.

## Decisión 15: copias de seguridad en .zip (2026-10-28)

| Tema | Elegido | Por qué |
|---|---|---|
| Formato | `.zip` con `manifest.json` (`mirlo_export: 1`), `mirlo.db` y las carpetas tal cual | Se abre con cualquier programa; el manifest permite rechazar lo que no es de Mirlo y cambiar el formato en el futuro |
| Copia de la BD | `VACUUM INTO` | Copia consistente aunque la BD esté abierta; copiar el archivo a pelo podría pillarla a medias |
| Compresión | Fotos y vídeos `Stored`, json/md/db `Deflated` | Los medios ya vienen comprimidos: recomprimir solo gasta tiempo |
| Copia ligera | Casilla "Incluir fotos, vídeos y documentos" | Sin medios, lo que falta se marca `not_in_export` y el post queda parcial: Reintentar lo baja |
| Importar | Fusiona, nunca sobrescribe: un post que ya está no cambia archivos ni texto; datos de la persona con `merge_user` (unión de etiquetas, favorito OR, colección y nota locales si las hay) | Importar una copia vieja nunca puede deshacer trabajo |
| De dónde salen las filas | De cada `post.json` (`vault::load`), no del `mirlo.db` exportado | La verdad está en disco. Del `mirlo.db` solo se toma lo que no está en ningún post.json: orden de colecciones y colores |
| Seguridad | Todo el zip se valida antes de escribir: rutas solo relativas normales (zip-slip), tope de 20 GB, SHA-256 de cada archivo | Un zip es entrada no fiable. Un archivo con huella distinta se borra y se marca `hash_mismatch` |
| Ventana | Comandos `async` con `spawn_blocking` y conexión propia | Un export de varios GB no congela la ventana ni bloquea los demás comandos |

**Descartado:** exportar a otros formatos (HTML estático queda para v0.2.0) y restaurar sobrescribiendo (contradice "nunca pisar").
**Código:** `src-tauri/src/backup.rs` (lógica y tests), `organize::merge_user`/`apply_user`, `vault::load`, `src/lib/Backup.svelte`.
