# Conceptos explicados en Mirlo

> Los conceptos que salieron en las notas del profesor durante la construcción de Mirlo, explicados desde cero.
> El mapa completo de todos los conceptos del proyecto, con su estado, está en [`HOJA_DE_RUTA.md`](HOJA_DE_RUTA.md).

## Sesión 1 — 2026-10-28 / 2026-10-29

## Datos estructurados frente a raspar HTML

**Qué es:** leer la información de un formato pensado para máquinas (JSON, con campos con nombre) en vez de buscarla dentro de una página web con patrones de texto (regex).
**Para qué sirve:** que el programa coja *exactamente* el texto del post, no el de una respuesta o un menú que se le parece.
**Analogía:** pedir la ficha de un libro en el mostrador de la biblioteca frente a buscarlo leyendo los lomos de todas las estanterías: con la ficha no te equivocas de libro.
**Dónde aparece en el proyecto:** `src-tauri/src/parser.rs` lee el JSON de X campo por campo.
**Se conecta con:** el fallo original de ThreadsVault, que extraía con regex sobre HTML.

## TDD (escribir el test antes que el código)

**Qué es:** primero escribes un test con el resultado correcto, lo ves fallar (rojo) y después escribes el código hasta que pasa (verde).
**Para qué sirve:** el test es el mapa del tesoro antes de cavar. En Mirlo pilló que en yt-dlp el campo `id` era el del vídeo, no el del post, antes de que llegara a guardarse ningún post con un enlace roto.
**Analogía:** apuntar en la receta cómo tiene que quedar el plato antes de cocinar; si no queda así, sabes que algo falló.
**Dónde aparece en el proyecto:** `parse_ytdlp` y su test `ytdlp_fallback_keeps_video_and_drops_media_link`.
**Se conecta con:** fixtures (datos reales guardados para los tests).

## Test double: un servidor de mentira

**Qué es:** sustituir algo real y lento (internet) por una pieza falsa que se comporta igual en lo que importa.
**Para qué sirve:** probar descargas, límites de tamaño y errores 404 en milisegundos, sin depender de X ni de la conexión.
**Analogía:** un muñeco de pruebas de choque: no es una persona, pero sirve para comprobar el airbag.
**Dónde aparece en el proyecto:** la función `serve()` de los tests de `vault.rs` abre un `TcpListener` en el puerto 0 y responde textos fijos.
**Se conecta con:** TDD y escritura atómica (los tests comprueban que no queda ningún `.part`).

## Índice invertido (búsqueda FTS5)

**Qué es:** una tabla que guarda, para cada palabra, en qué posts aparece. Buscar no es leer todos los posts: es consultar esa tabla.
**Para qué sirve:** que buscar entre miles de posts sea instantáneo, sin distinguir acentos y aceptando el principio de una palabra ("sqli" encuentra "sqlite").
**Analogía:** el índice alfabético al final de un libro de cocina: "tomate → páginas 12, 40, 87".
**Dónde aparece en el proyecto:** `posts_fts` en `src-tauri/src/db.rs`; los triggers la mantienen al día solos.
**Se conecta con:** triggers SQL y consultas parametrizadas.

## Object URL (`blob:`)

**Qué es:** una dirección inventada (`blob:…`) que apunta a datos que ya están en la memoria de la página.
**Para qué sirve:** reproducir los vídeos en Linux. El reproductor (GStreamer) no sabe abrir el protocolo `asset:` de Tauri, pero sí unos bytes que ya tiene delante.
**Analogía:** en vez de darle a alguien la dirección de un almacén al que no sabe llegar, le pones la caja en las manos y le das un número de ticket.
**Dónde aparece en el proyecto:** `api.videoUrl` crea el `blob:`; `Gallery.svelte` lo libera con `URL.revokeObjectURL` al cambiar de post.
**Se conecta con:** WebKitGTK y GStreamer (glosario de Chronicler).

## Migración de esquema (`PRAGMA user_version`)

**Qué es:** un número que SQLite guarda dentro del archivo de la base de datos y que no usa para nada propio: es tuyo. Sirve para saber qué cambios de estructura ya se aplicaron.
**Para qué sirve:** añadir una columna (`alt`, el texto alternativo de las fotos) a una base de datos que ya existe sin perder lo guardado. `CREATE TABLE IF NOT EXISTS` no añade columnas a una tabla que ya está.
**Analogía:** la pegatina de "revisión hecha" en un coche: el taller mira el número y solo hace las revisiones que faltan.
**Dónde aparece en el proyecto:** `db::migrate`; si `user_version` es 0, añade la columna y lo pone a 1.
**Se conecta con:** Room en Android hace lo mismo con su número de versión y sus `Migration`.

## Conflicto de archivos entre paquetes

**Qué es:** en Linux, cada archivo instalado tiene un paquete dueño; dos paquetes no pueden instalar el mismo archivo.
**Para qué sirve:** entender por qué el `.deb` de Mirlo no podía llevar `/usr/bin/yt-dlp`: ese archivo ya era del paquete `yt-dlp` del sistema. Solución: llamarlo `mirlo-yt-dlp`.
**Analogía:** dos inquilinos no pueden tener la misma plaza de garaje; el segundo necesita otra con su nombre.
**Dónde aparece en el proyecto:** `bundle.externalBin` en `tauri.conf.json` y `scripts/fetch-yt-dlp.mjs`.
**Se conecta con:** sidecar y empaquetado.

## CI (integración continua)

**Qué es:** un robot que, en cada `push`, arranca un ordenador limpio, compila el proyecto y pasa todos los tests.
**Para qué sirve:** que nada roto llegue a GitHub sin que te enteres, y comprobar que el proyecto no depende de algo que solo existe en tu ordenador.
**Analogía:** el control de calidad de una fábrica: cada pieza pasa la misma inspección, siempre igual.
**Dónde aparece en el proyecto:** `.github/workflows/ci.yml` (check, tests, clippy); su hermano `release.yml` compila los instaladores al crear una etiqueta.
**Se conecta con:** `npm run check` y `scripts/check-i18n.mjs`, que corren dentro del CI.

## El icono de ventana en X11 (`_NET_WM_ICON`)

**Qué es:** en Linux con X11, el icono de una ventana viaja como una lista de píxeles sin comprimir dentro de una propiedad de la ventana.
**Para qué sirve:** entender por qué el de 32 px se veía pixelado (el dock lo ampliaba) y por qué el de 512 px desaparecía (1 MB no cabe y GTK lo descarta sin avisar). El punto dulce es 256 px.
**Analogía:** un paquete que pasa por una ventanilla: demasiado pequeño y no se ve; demasiado grande y no entra.
**Dónde aparece en el proyecto:** el primer PNG de `bundle.icon` en `tauri.conf.json`; se comprueba con `xprop`.
**Se conecta con:** WebKitGTK y X11 (glosario de Chronicler).

## Separación de responsabilidades: códigos de mensaje

**Qué es:** cada pieza decide solo lo que le toca. Rust dice **qué** pasó (`invalid_link`, `too_big: 50`) y la interfaz decide **cómo** contarlo y en qué idioma.
**Para qué sirve:** traducir la app sin tocar Rust; que un error guardado en español se lea en inglés al cambiar de idioma; que el frontend reaccione al código (`offline`) y no a una frase que puede cambiar.
**Analogía:** la cocina pasa una nota `SIN RODABALLO` y el camarero decide cómo decírselo a cada mesa; si el cocinero escribiera la frase en su idioma, la mesa de turistas no la entendería.
**Dónde aparece en el proyecto:** `src-tauri/src/error.rs` (`fail`, `fail_with`) y `src/lib/i18n.svelte.ts` (`errorParts`, `t`).
**Se conecta con:** los `strings.xml` de Android: tu Kotlin dice `R.string.error_x` y Android elige `values/` o `values-en/`.
