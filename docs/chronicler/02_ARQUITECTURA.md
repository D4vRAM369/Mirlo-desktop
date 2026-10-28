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
