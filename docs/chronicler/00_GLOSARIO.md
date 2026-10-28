# Glosario

Términos técnicos del proyecto. Si un término no se entiende en otro archivo, está aquí.

---

## Scraping (raspado de HTML)

**Definición simple:** Leer una página web "a mano" buscando trozos de texto dentro del código de la página.
**Definición técnica:** Descargar el HTML de una URL y extraer datos con selectores o expresiones regulares, sin una API que los entregue ya ordenados.
**Analogía:** Buscar el número de teléfono de alguien leyendo una guía entera con lupa, en vez de preguntárselo directamente.
**En este proyecto lo usamos para:** Nada. Es justo lo que **no** hacemos (ver `05_ERRORES_Y_SOLUCIONES.md`).

## Regex (expresión regular)

**Definición simple:** Un patrón para buscar texto, como un "buscar" con comodines muy potentes.
**Definición técnica:** Lenguaje de patrones que describe conjuntos de cadenas. Sirve para texto plano; no entiende estructuras anidadas como HTML o JSON.
**Analogía:** Un colador: deja pasar lo que tiene la forma del agujero, aunque no sea lo que buscabas.
**Ejemplo:**
```ts
// Captura cualquier "text":"..." del HTML. Problema: hay decenas en la página
// (respuestas, menús, posts relacionados) y la regex no sabe cuál es el del post.
/"text"\s*:\s*"((?:[^"\\]|\\.)*)"/g
```

## JSON

**Definición simple:** Un formato de texto para guardar datos con etiquetas, como una ficha con campos.
**Definición técnica:** *JavaScript Object Notation*: objetos `{clave: valor}`, listas `[...]`, textos, números y booleanos.
**Analogía:** Un formulario rellenado: cada dato está en su casilla con nombre.
**En este proyecto lo usamos para:** Leer los posts. Pedimos `data.text` y obtenemos exactamente el texto del post, sin adivinar.
```json
{ "id_str": "20", "text": "just setting up my twttr", "display_text_range": [0, 24] }
```

## Endpoint de syndication (`cdn.syndication.twimg.com/tweet-result`)

**Definición simple:** Una dirección pública de X que devuelve los datos de un post en JSON, sin iniciar sesión.
**Definición técnica:** Endpoint que usan los *embeds* oficiales de X (los posts incrustados en otras webs). Recibe `id` y `token` y devuelve texto, entidades, medios con todas sus calidades, tweet citado y autor.
**Analogía:** La ventanilla de "información" del edificio: no es la puerta principal, pero te dan la ficha completa del post.
**En este proyecto lo usamos para:** Fuente principal de datos (prioridad 1).

## Token de syndication

**Definición simple:** Un código calculado a partir del ID del post que el endpoint pide para responder.
**Definición técnica:** `((id / 1e15) * π)` en base 36, quitando ceros y el punto. Verificado el 2026-10-28: sin token el endpoint responde `200` con `{}` vacío.
**Ejemplo:**
```js
// id 20 → token "6dq1a2xwd93"
((Number(id) / 1e15) * Math.PI).toString(36).replace(/(0+|\.)/g, '')
```

## `display_text_range`

**Definición simple:** Dos números que dicen qué parte del campo `text` es el texto "visible" del post.
**Definición técnica:** `[inicio, fin]` en code points. Fuera del rango quedan las @menciones automáticas de una respuesta y el enlace `t.co` de la foto o el vídeo.
**Analogía:** El marco de un cuadro: lo que queda dentro es la obra; lo de fuera es el embalaje.

## t.co

**Definición simple:** El acortador de enlaces de X: todos los enlaces de un post se convierten en `https://t.co/xxxx`.
**En este proyecto:** Sustituimos cada `t.co` por la URL real que viene en `entities.urls[].expanded_url`.

## oEmbed

**Definición simple:** Un estándar para pedir a una web "dame este contenido listo para incrustar".
**En este proyecto:** Fuente de respaldo (prioridad 3) en `publish.twitter.com/oembed`. Solo da texto, así que el post se marca como "parcial".

## Sidecar

**Definición simple:** Un programa externo que tu app lleva al lado y ejecuta cuando lo necesita.
**En este proyecto:** `yt-dlp`, un descargador de vídeo de línea de comandos (fuente de prioridad 2 y respaldo para vídeos).

## Variante de vídeo (`video_info.variants`)

**Definición simple:** El mismo vídeo en varias calidades.
**En este proyecto:** Elegimos la de tipo `video/mp4` con mayor `bitrate`. Ejemplo real: 256k, 832k, 2176k y 10368k → nos quedamos con 10368k.

## Fixture

**Definición simple:** Un archivo de ejemplo real que se guarda para usarlo en los tests.
**En este proyecto:** `tests/fixtures/*.json` son respuestas reales de X descargadas el 2026-10-28 (texto, fotos, vídeo, GIF, cita, PDF, respuesta, emoji). Los tests las leen sin red, así que dan el mismo resultado hoy y dentro de un año.
**Analogía:** La muestra de pintura que te llevas a la tienda: comparas siempre contra la misma.

## TDD (Test-Driven Development)

**Definición simple:** Escribir primero la prueba, verla fallar y después escribir el código que la hace pasar.
**Por qué:** Si el test nunca falló, no sabes si de verdad comprueba algo.
**En este proyecto:** 14 tests del parser en rojo con una función vacía → implementación → 16 en verde.

## Función pura

**Definición simple:** Una función que con la misma entrada siempre devuelve lo mismo y no toca nada de fuera (ni red, ni disco, ni reloj).
**En este proyecto:** `parse_tweet(json) -> Post`. Por eso se puede probar con fixtures.
**Comparación con Python:** `def parse(texto: str) -> dict` sin `requests` ni `open()` dentro.

## Code point

**Definición simple:** Un "carácter" tal como lo cuenta Unicode. 🌎 es 1 code point pero 4 bytes en UTF-8.
**En este proyecto:** `display_text_range` cuenta code points; por eso en Rust recortamos con `.chars()` y no con índices de bytes.

## `note_tweet`

**Definición simple:** La marca que pone X en los posts de más de 280 caracteres.
**En este proyecto:** syndication solo da los primeros ~280 caracteres y un `note_tweet` con un ID. Lo usamos para marcar el post como `text_truncated` en lugar de guardar medio texto como si fuera completo.

## Clippy

**Definición simple:** El "corrector de estilo y errores" de Rust.
**Comando:** `cargo clippy --all-targets -- -D warnings` (cualquier aviso cuenta como error).

## SSRF

**Definición simple:** Engañar a un programa para que haga peticiones a sitios a los que no debería.
**En este proyecto:** `x_url::status_id` solo acepta enlaces de x.com o twitter.com con `/status/<número>`, y el backend construye él mismo las URLs a partir del número.

## Escritura atómica (.part + rename)

**Definición simple:** Escribir en un nombre provisional y renombrar al final. Renombrar dentro del mismo disco es instantáneo: el archivo aparece entero o no aparece.
**En este proyecto:** `vault::fetch_to` escribe `01.jpg.part`; `get_file` lo renombra a `01.jpg`.

## SHA-256

**Definición simple:** La "huella dactilar" de un archivo: 64 caracteres que cambian por completo si cambia un solo byte.
**En este proyecto:** se guarda en `post.json` por cada archivo. Sirve para detectar corrupción y, más adelante, duplicados.

## Content-Type

**Definición simple:** Cabecera HTTP donde el servidor dice qué tipo de archivo envía (`application/pdf`, `text/html`…).
**En este proyecto:** decide si un enlace es un documento que se guarda o una web que se ignora.

## Backoff

**Definición simple:** Esperar cada vez más entre reintentos (2 s, 4 s…) para no machacar un servidor que ya va mal.
