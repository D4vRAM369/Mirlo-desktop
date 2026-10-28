# Errores y soluciones

## Error: ThreadsVault guarda texto que no es del post

**Fecha:** 2026-10-28 (análisis; el bug viene de versiones anteriores)
**Contexto:** Guardar un post de Threads en ThreadsVault-desktop.
**El error:** A veces el texto guardado incluye respuestas, textos de la interfaz o un `og:description` cortado; otras veces sale vacío.
**Por qué ocurrió:**
1. `extractPostData` lanza 3 peticiones (oEmbed, HTML directo y Jina) y mezcla hasta 7 candidatos de texto con una cadena de `??`.
2. Los extractores usan regex sobre HTML: `extractTextFromThreadsJson` coge **cualquier** `"text":"..."` de la página y se queda con el más largo con saltos de línea, que puede ser una respuesta.
3. `extractTextFromThreadsReactState` también devuelve el candidato más largo, suponiendo que "el post principal es el más largo".
4. La sección de Jina se recorta buscando el ID del post en Markdown; si el corte falla, entra el hilo o el feed.
5. Cuando todo falla, se usa `og:description`, que Threads trunca o rellena con el hilo.

**Cómo se resolverá:** En XVault no se adivina: el texto sale del campo `text` del JSON de syndication, recortado con `display_text_range`. Si ninguna fuente fiable responde, se muestra un error, nunca "lo que pille". Los tests con fixtures fallarán si entra texto ajeno.
**Qué aprendimos:** Los filtros de ruido no arreglan una fuente ruidosa. Hay que cambiar de fuente.

## Error: el endpoint responde 200 pero vacío

**Fecha:** 2026-10-28
**Contexto:** Probar `tweet-result?id=20` sin `token`.
**El error exacto:**
```
HTTP 200
{}
```
**Por qué ocurrió:** El endpoint exige `token`; sin él no da error, devuelve un objeto vacío.
**Cómo se resolvió:** Calcular el token a partir del ID (ver Glosario). El parser tratará `{}` o la ausencia de `id_str` como fallo de la fuente.
**Qué aprendimos:** Validar el contenido, no solo el código HTTP.

## Error: scroll horizontal en móvil (mockups A y C)

**Fecha:** 2026-10-28
**Contexto:** Revisar los mockups a 360 px de ancho.
**El error:** La página medía 704 px (A) y 365 px (C) en una pantalla de 341 px.
**Por qué ocurrió:** (A, C) columnas de grid con `1fr`, que no encogen por debajo de su contenido; (C) además una URL larga sin puntos de corte.
**Cómo se resolvió:** `grid-template-columns: minmax(0, 1fr)` y `overflow-wrap: anywhere` en el texto del post.
**Qué aprendimos:** Probar siempre el ancho mínimo; el contenido real (URLs, tiras de imágenes) es lo que rompe el diseño.

## Error: el t.co de la foto aparece dentro del texto visible

**Fecha:** 2026-10-28
**Contexto:** Fixture `photo_1` (post de 2014).
**El error:** `display_text_range` es `[0, 81]`, todo el texto, incluido `http://t.co/C9U5NOtGap`, que es el enlace de la foto.
**Por qué ocurrió:** En posts antiguos X no excluía el enlace de la foto del rango.
**Cómo se resolvió:** Además de recortar por el rango, se borran las URLs de `entities.media`.
**Qué aprendimos:** Un campo de una API puede cambiar de comportamiento con los años. Por eso los fixtures cubren posts de épocas distintas (2006, 2014, 2021, 2024, 2026).

## Error: posts largos truncados a 280 caracteres

**Fecha:** 2026-10-28
**Contexto:** Fixtures `video` y `reply_mention`.
**El error:** El texto acaba a mitad de frase: `"…a foundational piece of"`.
**Por qué ocurrió:** Son posts de más de 280 caracteres. Syndication y yt-dlp solo dan el principio; el resto está en `note_tweet`, que aquí solo trae un ID.
**Cómo se resolvió (de momento):** Se marca `text_truncated = true` y el post se guarda como parcial. FxTwitter (API de terceros) sí da el texto completo, pero usarlo es una decisión de privacidad pendiente.
**Qué aprendimos:** Mejor un "parcial" honesto que un texto a medias que parece completo.

## Error: el test de yt-dlp devolvía otro ID

**Fecha:** 2026-10-28
**El error exacto:**
```
assertion `left == right` failed
  left: "1969513398892445696"
 right: "1969515038512926823"
```
**Por qué ocurrió:** En la salida de yt-dlp, `id` es el ID del **vídeo**; el del **post** está en `display_id`.
**Cómo se resolvió:** Leer `display_id`.
**Qué aprendimos:** Este bug habría guardado posts con un enlace roto. Lo cazó un test escrito antes del código.

## Error: oEmbed responde vacío

**Fecha:** 2026-10-28
**El error:** `json.decoder.JSONDecodeError: Expecting value` al leer `publish.twitter.com/oembed`.
**Por qué ocurrió:** El dominio redirige (`301`) a `publish.x.com` con cuerpo vacío, y `curl` sin `-L` no sigue redirecciones.
**Cómo se resolvió:** Usar directamente `publish.x.com/oembed`.

## Actualización: posts largos (2026-10-28)

Resuelto con FxTwitter como fuente extra solo del texto (Decisión 6). Si FxTwitter no responde, el post sigue marcado como parcial.

## Error: el PDF enlazado no termina en .pdf

- **Síntoma:** `https://data.ntsb.gov/.../GenerateNewestReport/202401/pdf` es un PDF, pero su URL acaba en `/pdf`.
- **Causa:** muchas webs generan documentos sin extensión en la URL.
- **Solución:** pedir el enlace, leer la cabecera `Content-Type` y cortar antes de bajar el cuerpo si no es un documento de la lista blanca.
- **Lección:** la URL miente; la cabecera la manda el servidor que sirve el archivo.

## Error: los tests dejaban carpetas en /tmp

- **Síntoma:** carpetas `xvault-test-*` en `/tmp` tras cada ejecución, con el PID en el nombre.
- **Solución:** los tests escriben en `src-tauri/target/test-tmp/`, ignorado por git y limpiado por `cargo clean`.

## Error: Ctrl+V fuera de un campo no hacía nada (WebKitGTK)

- **Síntoma:** en la ventana real, pegar un enlace sin tener el cursor en un campo no guardaba ni mostraba aviso.
- **Causa:** Tauri en Linux usa WebKitGTK (el motor de Safari). WebKit no dispara el evento `paste` si no hay un elemento editable enfocado; Chrome sí. El backend nunca recibió nada.
- **Solución:** en `keydown` con Ctrl+V se enfoca el campo de enlace antes de que el navegador pegue; `onInput` detecta `inputType === 'insertFromPaste'` y guarda si es un enlace de X.
- **Lección:** los eventos del portapapeles se prueban en el motor real de la app, no en Chrome.
