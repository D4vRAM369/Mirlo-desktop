# Prompt — XVault (X/Twitter) desktop

> Pégalo tal cual en una sesión nueva de Claude Code abierta en esta carpeta.

---

Vamos a construir **XVault**: la versión para X/Twitter de mi app **ThreadsVault-desktop**
(`ThreadsVault-desktop`). Bóveda local: pego la URL de un post de X,
la app guarda texto, autor, fecha y **todos los archivos (fotos, vídeos, GIFs, documentos enlazados) en disco**.
Sin nube, sin telemetría, sin cuenta obligatoria.

## 0. Antes de escribir código

1. Lee ThreadsVault-desktop para reutilizar lo que funciona: stack (Tauri 2 + Svelte 5 + Tailwind 4 +
   SQLite vía `@tauri-apps/plugin-sql`), estructura de `src/routes`, `src/lib/storage`, el sidecar
   `yt-dlp` en `src-tauri/src/main.rs` y el sistema i18n.
2. Lee `src/lib/utils/post-extractor.ts` y las funciones de regex sobre HTML de `main.rs`
   (`extract_video_candidate`, `extract_first`…). **Ahí está el fallo conocido**: a veces mete texto que
   no es del post (respuestas, UI, og:description truncado) o no extrae bien. Escríbeme en 5 líneas
   la causa raíz que veas. XVault **no** repetirá ese patrón.
3. Hazme las preguntas que bloqueen de verdad (máx. 3) y propón el plan antes de implementar.

## 1. Regla de oro de la extracción

**Nunca extraer contenido con regex sobre HTML.** Solo de fuentes estructuradas (JSON), en este orden:

| Prioridad | Fuente | Qué da |
|---|---|---|
| 1 | `https://cdn.syndication.twimg.com/tweet-result?id=<ID>&token=<t>` (endpoint del embed oficial, sin login) | JSON: `text`, `entities`, `display_text_range`, `mediaDetails` (fotos + `video_info.variants`), `quoted_tweet`, `card`, `user`, `created_at` |
| 2 | `yt-dlp --dump-json <url>` (sidecar ya existente) | metadatos + vídeo en mejor calidad |
| 3 | `https://publish.twitter.com/oembed?url=<url>` | solo texto de respaldo, marcado como "parcial" |

- Investiga primero cómo se calcula `token` hoy (cambia; verifica con una petición real, no de memoria).
- Texto del post = `text` recortado con `display_text_range` (quita @menciones iniciales y el t.co final
  de la media) y con los `t.co` sustituidos por `entities.urls[].expanded_url`. Nada más.
- Si una fuente falla, se pasa a la siguiente y **se guarda qué fuente se usó** (`source` en la BD).
  Si ninguna da texto fiable → error claro en la UI, nunca texto inventado o "lo que pille".

## 2. Archivos locales

- Carpeta por post: `~/XVault/<usuario>/<YYYY-MM-DD>_<tweetId>/`
  - `post.json` (JSON crudo de la fuente, para poder re-parsear en el futuro)
  - `post.md` (texto limpio + metadatos legibles)
  - `media/01.jpg`, `media/02.mp4`… en orden original
- Fotos: URL con `?name=orig` (máxima resolución).
- Vídeos/GIFs: variante `video/mp4` de mayor `bitrate`; si falla, `yt-dlp`. Los GIFs de X son mp4: guárdalos como mp4.
- Documentos: enlaces del post (`entities.urls`) que apunten a PDF/DOCX/ZIP/etc. → descargar a `attachments/`
  tras comprobar `Content-Type` y tamaño máx. configurable. Lo demás se guarda solo como enlace.
- Tweet citado (`quoted_tweet`): guardarlo anidado con sus propios medios.
- Descargas: escritura a `.part` y rename al terminar; hash SHA-256 en BD; reintento con backoff; nunca dejar
  un archivo a medias como si estuviera completo.
- Fuera de alcance v1: hilos completos y posts protegidos/NSFW con login. Diséñalo para que `--cookies-from-browser`
  de yt-dlp se pueda añadir después como opción, sin implementarlo ahora.

## 3. Diseño (mejor que ThreadsVault)

- Antes de UI, carga la skill `frontend-design` y proponme **2 direcciones visuales** con mockup estático.
- Requisitos: tema claro/oscuro, galería de medios con visor (vídeo inline), búsqueda full-text (SQLite FTS5),
  filtros por autor/fecha/tipo de medio, estado de descarga por post (ok / parcial / error + reintentar),
  pegar URL con Ctrl+V desde cualquier pantalla, accesible con teclado.
- Nada de estética genérica de "IA" (gradientes morados, emojis en botones).

## 4. Calidad (menos fallos)

- **Fixtures primero (TDD):** guarda 8–10 JSON reales en `tests/fixtures/` cubriendo: solo texto, 1 foto,
  4 fotos, vídeo, GIF, tweet citado, enlace a PDF, respuesta que empieza por @mención, texto con emojis/URLs,
  post borrado. El parser es una **función pura** `JSON → Post` testeada contra esos fixtures
  (`cargo test` y/o `vitest`). El test debe fallar si entra texto que no sea del post.
- Red y disco solo en la capa Tauri (Rust); el frontend llama comandos tipados.
- Validar que la URL es `x.com`/`twitter.com`/`mobile.twitter.com` + `/status/<id numérico>`; nada de URLs arbitrarias
  al backend (SSRF), nombres de archivo saneados (path traversal).
- `npm run check` y `cargo clippy` sin warnings antes de dar algo por terminado.

## 5. Retroportar a ThreadsVault

Al terminar, escribe `docs/BACKPORT-THREADSVAULT.md`: qué piezas del extractor (parser puro + fixtures +
cadena de fuentes con fallback + escritura atómica) se llevan a ThreadsVault-desktop, qué archivos cambiar
y en qué orden. **No toques ThreadsVault en esta sesión.**

## 6. Cómo trabajamos

- Modo mentor PBL: explícame cada concepto nuevo (syndication API, FTS5, escritura atómica, sidecars…) con el
  "por qué" antes del "cómo", y para en checkpoints.
- Entregas pequeñas y que funcionen: (1) extractor + tests, (2) descarga de medios, (3) BD + UI básica,
  (4) diseño final, (5) empaquetado. Commit al cerrar cada fase (git init si no existe).
- Termina cada sesión con el resumen exportable.
