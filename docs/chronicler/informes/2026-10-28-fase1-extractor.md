# Fase 1: extractor de posts de X con TDD

> Informe de task para alguien que sabe algo de programación (por ejemplo Python) pero no domina Rust ni Tauri.

## 1. Resumen para situarse

- **Qué se ha construido:** Un extractor que recibe un enlace de X y devuelve el post limpio: texto, autor, fecha, fotos, vídeos, GIF, enlaces y post citado.
- **Dónde está:** `src-tauri/src/` (`x_url.rs`, `parser.rs`, `sources.rs`, `main.rs`) y `tests/fixtures/`.
- **Por qué era necesario:** ThreadsVault mezclaba texto ajeno al post porque extraía con regex sobre HTML.
- **Estado final:** 17 tests en verde, 2 pruebas en vivo en verde, clippy y svelte-check sin avisos. Todavía no descarga archivos (fase 2).

## 2. Qué problema resolvía

Pegar un enlace y obtener **solo** el contenido de ese post, de forma verificable. Si no se puede, un error claro, no texto inventado.

## 3. Archivos principales

- `src-tauri/src/x_url.rs`: decide si lo pegado es un post de X y saca su número. Es la puerta de entrada.
- `src-tauri/src/parser.rs`: convierte JSON en `Post`. No usa red: es una función pura.
- `src-tauri/src/sources.rs`: hace las peticiones y prueba las fuentes en orden.
- `src-tauri/src/main.rs`: expone `extract_post` al frontend.
- `tests/fixtures/*.json`: 12 respuestas reales de X, yt-dlp y oEmbed.
- `src/App.svelte`: pantalla provisional para probar a mano.

## 4. Paso a paso

1. Probar la API a mano con `curl` antes de escribir código (token, respuesta vacía).
2. Crear el proyecto copiando solo la config de ThreadsVault.
3. Descargar fixtures reales y mirar su estructura con Python.
4. Escribir los tests con el comportamiento esperado y verlos fallar.
5. Implementar hasta que pasen. Un test pilló un bug real (`display_id`).
6. Añadir la cadena de fuentes con red y dos pruebas en vivo marcadas `#[ignore]`.
7. Pasar clippy, svelte-check y build.

## 5. Conceptos

- **Datos estructurados vs. scraping:** se pregunta por el campo `text` en vez de adivinar en el HTML.
- **Función pura:** el parser no toca red ni disco, así que se prueba con archivos.
- **Code points:** los índices de X cuentan caracteres Unicode, no bytes.
- **Errores como valores:** `Result<Post, ParseError>` obliga a decidir qué hacer si falla (en Python sería lanzar o devolver `None`; Rust no te deja olvidarlo).

## 6. Comparación mental con Python

```python
def parse_tweet(raw: str) -> Post:          # parser.rs
    d = json.loads(raw)
    if "id_str" not in d:
        raise EmptyResponse()
    a, b = d.get("display_text_range", [0, len(d["text"])])
    text = d["text"][a:b]                   # Python ya indexa por code points
    for m in d["entities"].get("media", []):
        text = text.replace(m["url"], "")
    for u in d["entities"].get("urls", []):
        text = text.replace(u["url"], u["expanded_url"])
    return Post(text=html.unescape(text.strip()), ...)

def extract(id):                            # sources.rs
    for fuente in (syndication, ytdlp, oembed):
        try:
            return fuente(id)
        except Exception as e:
            errores.append(e)
    raise NoSePudoLeer(errores)
```

## 7. Decisiones técnicas

- **Parser separado de la red.** Alternativa rechazada: parsear dentro de la función que descarga (imposible de probar sin red).
- **Sin `chrono` ni crates de HTML.** Alternativa rechazada: añadir dependencias para 20 líneas de código.
- **Posts largos marcados como parciales.** Alternativa pendiente: FxTwitter (texto completo, pero es un tercero).
- **Token de syndication aproximado.** Hoy X no lo valida; está anotado con `ponytail:` en el código.

## 8. Pruebas y verificación

```bash
cd src-tauri && cargo test && cargo test -- --ignored
cargo clippy --all-targets -- -D warnings
cd .. && npm run check && npm run build
```

## 9. Pendiente

- Fase 2: descargar medios y escribir `post.json` y `post.md`.
- Decidir si se usa FxTwitter para posts largos.
- No se ha probado la ventana real de Tauri (`npm run tauri dev`): el comando se probó por debajo, con los tests en vivo.
