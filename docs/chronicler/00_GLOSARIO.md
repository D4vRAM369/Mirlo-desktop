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

## FTS5

**Definición simple:** El buscador de texto completo que trae SQLite: un índice de palabras, como el de un libro.
**En este proyecto:** tabla `posts_fts` sobre el texto, autor y texto citado. Se consulta con `MATCH`.

## Trigger

**Definición simple:** Código SQL que la base de datos ejecuta sola cuando pasa algo (un INSERT, UPDATE o DELETE).
**En este proyecto:** `posts_ai`, `posts_au` y `posts_ad` mantienen `posts_fts` sincronizada con `posts`.

## Mutex

**Definición simple:** Un candado: solo un hilo a la vez puede usar lo que protege.
**En este proyecto:** la conexión SQLite vive en `Mutex<Connection>` porque varios comandos pueden llegar a la vez.

## Protocolo asset:

**Definición simple:** La forma en que Tauri deja al WebView mostrar archivos del disco, solo dentro de las carpetas permitidas.
**En este proyecto:** `assetProtocol.scope = ["$HOME/XVault/**"]`; `api.fileUrl` construye la URL.

## Sidecar

**Definición simple:** Un programa aparte que la app lleva dentro y ejecuta cuando lo necesita.
**En este proyecto:** `xvault-yt-dlp`, empaquetado con `bundle.externalBin` y descargado por `npm run sidecar`.

## actionlint

**Definición simple:** Un corrector para los archivos de GitHub Actions. Encuentra errores (una clave mal escrita, una expresión `${{ }}` inválida, un `run:` con un fallo de shell) antes de hacer push, en lugar de descubrirlos cuando el workflow falla en GitHub.

## i18n

**Definición simple:** "Internationalization" (i + 18 letras + n). Preparar la app para mostrarse en varios idiomas sin tocar el código de cada pantalla.
**En este proyecto:** `t('clave')` busca el texto en el diccionario del idioma activo (`src/lib/i18n.svelte.ts`).

## WebKitGTK

**Definición simple:** El "navegador sin barra de direcciones" que Tauri usa en Linux para pintar la interfaz de la app. Es WebKit (el motor de Safari) adaptado al escritorio GTK de Linux.
**Por qué importa:** Tauri no lleva un navegador propio dentro (por eso los paquetes pesan ~45 MB y no 150 MB como una app Electron): usa el que ya trae cada sistema. En Windows es WebView2 (motor de Edge/Chrome), en macOS WKWebView (Safari) y en Linux WebKitGTK. Mismo HTML, **tres motores distintos**: lo que funciona en uno puede fallar en otro.
**En este proyecto:** tres errores salieron de aquí, y en Chrome no se ven:
- `paste` no se dispara sin un campo de texto enfocado → Ctrl+V global ([05](05_ERRORES_Y_SOLUCIONES.md)).
- `<video>` no reproduce archivos servidos por `asset:` → los vídeos van como `blob:`.
- El icono de ventana de 512 px se descarta en X11 → se usa el de 256 px.
**Lección:** probar en la ventana real de la app, no solo en el navegador de desarrollo.

## GStreamer

**Definición simple:** La biblioteca de Linux que reproduce audio y vídeo. WebKitGTK le pasa cada `<video>` para que lo decodifique.
**En este proyecto:** GStreamer no sabe abrir el esquema `asset:` de Tauri; por eso Mirlo lee el vídeo en Rust y se lo entrega como `blob:`.

## URL blob: (object URL)

**Definición simple:** Una dirección inventada (`blob:…`) que apunta a datos que ya están en la memoria de la página, no a un archivo ni a un servidor.
**En este proyecto:** `api.videoUrl` convierte los bytes del vídeo en un `Blob` y crea su URL con `URL.createObjectURL`; al cambiar de post se libera con `URL.revokeObjectURL` para no acumular memoria.

## X11

**Definición simple:** El sistema clásico de ventanas de Linux: dibuja las ventanas y comunica a las apps con el escritorio. Su sucesor es Wayland.
**En este proyecto:** el icono de la ventana viaja como píxeles sin comprimir en la propiedad `_NET_WM_ICON`; uno de 512 px (1 MB) no cabe y GTK lo descarta.

## Transacción

**Definición simple:** Un grupo de cambios en la base de datos que se aplican todos o ninguno, como un punto de guardado en un videojuego.
**En este proyecto:** la migración a v2 y cada cambio de etiquetas van en `unchecked_transaction()` … `commit()`: si algo falla a mitad, la BD queda como estaba.

## Migración de esquema (`PRAGMA user_version`)

**Definición simple:** Cambiar la forma de una base de datos que ya tiene datos (tablas, columnas) sin perderlos, llevando la cuenta de qué versión tiene.
**En este proyecto:** `db::migrate` lee `user_version` y aplica los pasos pendientes: v1 añade `alt` a `files`; v2 añade colecciones, etiquetas, favorito y nota, y rehace el índice.

## Desnormalizar

**Definición simple:** Guardar una copia de un dato en otro sitio para leerlo más rápido, a cambio de mantener las dos copias iguales.
**En este proyecto:** `posts.tags_text` repite los nombres de las etiquetas del post para que la búsqueda FTS5 los encuentre; `organize.rs` lo rehace en cada cambio.

## Rueda de color HSV

**Definición simple:** Una forma de elegir color con tres números: tono (el ángulo en la rueda), saturación (lo lejos del centro) y brillo.
**En este proyecto:** `ColorWheel.svelte` la dibuja con dos degradados CSS y convierte la posición del puntero a `#rrggbb`.

## xdotool

**Definición simple:** Programa de Linux (X11) que mueve el ratón y pulsa teclas en una ventana como si lo hiciera una persona.
**En este proyecto:** se usa para probar la ventana real de Mirlo (WebKitGTK) junto con `import -window` para las capturas.

## Zip-slip

**Definición simple:** Ataque con un `.zip` que trae rutas como `../../algo` para que, al descomprimir, se escriba fuera de la carpeta elegida.
**En este proyecto:** `backup::is_safe` solo acepta rutas relativas normales y se comprueba todo el zip antes de escribir nada.

## VACUUM INTO

**Definición simple:** Orden de SQLite que escribe una copia compacta y consistente de la base de datos en otro archivo, aunque esté abierta.
**En este proyecto:** exportar la usa para meter `mirlo.db` en el `.zip`.
