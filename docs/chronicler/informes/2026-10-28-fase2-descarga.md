# Fase 2: guardar posts en disco

> Informe de task para alguien que sabe algo de programación (por ejemplo Python) pero no domina Rust ni Tauri.

## 1. Resumen para situarse

- **Qué se ha construido:** el botón "Guardar post" descarga el post a `~/XVault/<usuario>/<fecha>_<id>/` con `post.json`, `post.md`, `media/` y `attachments/`.
- **Dónde está:** `src-tauri/src/vault.rs` (nuevo), `parser.rs` (`parse_fxtwitter_text`), `sources.rs` y `main.rs` (`save_post`).
- **Estado final:** 26 tests en verde, 4 pruebas en vivo en verde (5 posts reales guardados), clippy y svelte-check sin avisos.

## 2. Qué problema resolvía

Tener el post **en local** de forma fiable: archivos completos o marcados como fallidos, nunca a medias, y comprobables con su huella SHA-256.

## 3. Archivos principales

- `vault.rs`: carpetas, nombres, descargas, anti-SSRF, `post.json` y `post.md`.
- `parser.rs`: texto completo de posts largos desde FxTwitter.
- `tests/fixtures/fxtwitter_long.json`: respuesta real recortada.

## 4. Flujo

```
URL pegada → x_url (¿es un post?) → sources (JSON del post)
          → vault::save → media/NN (twimg o yt-dlp)
                        → attachments/ (si el Content-Type es documento)
                        → post.json + post.md (escritura atómica)
```

## 5. Decisiones clave

Ver Decisiones 6 y 7 en `02_ARQUITECTURA.md`.

## 6. Qué NO está verificado

- El respaldo de vídeo con yt-dlp: en las pruebas reales la descarga directa funcionó siempre, así que esa rama no se ha ejecutado.
- La ventana real de Tauri (`npm run tauri dev`).
- Rebinding de DNS (límite conocido, anotado en el código).

## 7. Siguiente

Fase 3: SQLite + FTS5, guardar `source` y `status`, y lista con reintento de los parciales.
