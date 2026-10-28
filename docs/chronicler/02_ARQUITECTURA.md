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
