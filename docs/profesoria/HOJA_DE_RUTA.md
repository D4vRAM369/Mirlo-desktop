# 🗺️ Hoja de ruta de misión: Mirlo

> El mapa de todo lo que hay dentro de Mirlo, zona por zona, con el estado de cada concepto para ti.
> Úsalo para repasar, para elegir qué estudiar después, o para explicarle el proyecto a otra persona.

**Leyenda**

| Estado | Significa |
|---|---|
| ✅ | Dominado: lo has usado o explicado tú sin ayuda |
| 🟡 | Explicado al menos una vez en una nota del profesor |
| ❌ | Ha salido en el código, pero aún no se ha explicado: zona sin explorar |

Cada concepto apunta al sitio de **tu** código donde vive. Ábrelo mientras lees: el código es el libro de texto.

---

## 🏰 Zona 1: el extractor (leer un post sin inventarse nada)

*Misión: pegar un enlace y obtener solo el contenido de ese post, o un error claro.*

| Concepto | Dónde vive | Estado | En una frase |
|---|---|---|---|
| Datos estructurados (JSON) vs raspar HTML | `parser.rs` | 🟡 | Leer campos con nombre en vez de adivinar con regex sobre una página |
| HTTP 200 ≠ éxito | `sources.rs` | ❌ | Un servidor puede responder "todo bien" con un `{}` vacío |
| Función pura | `parser::parse_tweet` | ❌ | Recibe datos, devuelve datos; sin red ni disco, así se prueba fácil |
| TDD (test en rojo primero) | `parser.rs`, tests | 🟡 | Escribir qué debe pasar antes del código; el bug de `display_id` lo pilló así |
| Fixtures | `tests/fixtures/*.json` | ❌ | Respuestas reales guardadas para probar sin internet |
| Test invariante | `no_fixture_leaks_tco_links_into_text` | ❌ | Un test que recorre todos los casos y protege los futuros |
| `Result<T, E>` | todo `src-tauri/src` | ❌ | La forma de Rust de decir "esto puede salir bien o mal" sin excepciones |
| Unicode: code points vs bytes | `clean_text` | ❌ | Un emoji es 1 carácter para X pero 4 bytes para la memoria |
| serde (`Deserialize`) | `RawTweet` | ❌ | Convertir JSON en structs de Rust automáticamente |
| Cadena de fuentes con respaldo | `sources::extract` | ❌ | Si la primera fuente falla, se prueba la siguiente y se apunta cuál funcionó |
| SSRF y validación en la frontera | `x_url.rs` | ❌ | Nunca dejar que un texto de fuera decida a qué servidor llama tu app |

## ⚔️ Zona 2: las descargas (archivos completos o ninguno)

*Misión: guardar fotos, vídeos y documentos sin dejar nunca un archivo a medias.*

| Concepto | Dónde vive | Estado | En una frase |
|---|---|---|---|
| Escritura atómica (`.part` + rename) | `vault::fetch_to`, `write_atomic` | ❌ | El nombre final solo aparece cuando el archivo está entero |
| SHA-256 | `vault.rs`, `post.json` | ❌ | La huella dactilar de un archivo: cambia entera si cambia un byte |
| Content-Type | `vault::doc_ext` | ❌ | Lo que dice el servidor que envía; manda más que la extensión de la URL |
| Reintentos con backoff | `vault::fetch_to` | ❌ | Reintentar esperando cada vez más, y solo lo que puede arreglarse esperando |
| Test double (servidor de mentira) | `serve()` en los tests de `vault.rs` | 🟡 | Un `TcpListener` local que hace de internet en los tests |
| IPs privadas y DNS | `vault::is_public_ip` | ❌ | Por qué un enlace a `192.168.1.1` no debe tocarse nunca |
| `async`/`await` y `spawn_blocking` | `sources::ytdlp_json` | ❌ | Esperar a la red sin congelar la app; aislar lo que sí bloquea |

## 🗝️ Zona 3: la base de datos y la interfaz básica

*Misión: buscar en miles de posts al instante y reintentar los que fallaron.*

| Concepto | Dónde vive | Estado | En una frase |
|---|---|---|---|
| Índice invertido (FTS5) | `db.rs`, `posts_fts` | 🟡 | El índice al final de un libro: palabra → posts donde sale |
| Triggers | `db.rs`, `posts_ai/au/ad` | ❌ | SQL que la base de datos ejecuta sola cuando cambia una tabla |
| Consultas parametrizadas | `db::list` (`?1`, `?2`) | ❌ | Los datos van aparte del SQL: así no hay inyección SQL |
| Mutex | `main.rs`, `App.db` | ❌ | Un candado: un comando a la vez usa la conexión |
| Comando Tauri (`#[tauri::command]` + `invoke`) | `main.rs`, `src/lib/api.ts` | ❌ | El puente entre el botón de Svelte y la función de Rust |
| Runes de Svelte 5 (`$state`, `$derived`, `$effect`) | `App.svelte` | ❌ | Variables que repintan la pantalla solas cuando cambian |
| Eventos del navegador según el motor | `App.svelte`, `onKey` | ❌ | WebKitGTK no lanza `paste` sin un campo enfocado (ver glosario) |

## 🎨 Zona 4: el diseño Bandeja

*Misión: una interfaz que se maneja con teclado y no parece hecha por una IA.*

| Concepto | Dónde vive | Estado | En una frase |
|---|---|---|---|
| CSS Grid y `minmax(0, 1fr)` | `App.svelte`, `.panes` | 🟡 | Por qué una columna no encoge y cómo obligarla |
| Tokens de diseño (variables CSS) | `src/app.css` | ❌ | Los colores con nombre en un solo sitio; el tema oscuro solo cambia sus valores |
| `prefers-color-scheme` | `src/app.css` | ❌ | El sistema le dice a la página si quieres modo oscuro |
| `<dialog>` nativo | `Gallery.svelte` | ❌ | El visor de fotos sin librerías: foco, Esc y fondo gratis |
| URL `blob:` (object URL) | `api.videoUrl`, `Gallery.svelte` | 🟡 | Una dirección inventada que apunta a datos ya en memoria |
| Migración de esquema (`user_version`) | `db::migrate` | 🟡 | Un contador dentro del archivo que dice qué cambios ya se aplicaron |
| Roving tabindex | `App.svelte`, filas de la lista | ❌ | Solo la fila activa es "tabulable"; las flechas mueven el foco |

## 📦 Zona 5: el empaquetado

*Misión: que cualquiera instale Mirlo con un clic, con yt-dlp dentro.*

| Concepto | Dónde vive | Estado | En una frase |
|---|---|---|---|
| Sidecar | `tauri.conf.json` (`externalBin`) | ❌ | Un programa aparte que la app lleva dentro |
| Verificar descargas con checksum | `scripts/fetch-yt-dlp.mjs` | ❌ | Comparar el SHA-256 con el publicado antes de fiarse de un binario |
| Conflicto de archivos entre paquetes | `mirlo-yt-dlp` | 🟡 | Dos paquetes no pueden ser dueños del mismo `/usr/bin/x` |
| CI (integración continua) | `.github/workflows/ci.yml` | 🟡 | Un robot que compila y pasa los tests en cada push |
| CD / release automática | `.github/workflows/release.yml` | ❌ | Una etiqueta `v0.1.0` y GitHub compila los instaladores de 3 sistemas |

## 🐦 Zona 6: la identidad (icono y nombre)

| Concepto | Dónde vive | Estado | En una frase |
|---|---|---|---|
| Icono de ventana en X11 (`_NET_WM_ICON`) | `tauri.conf.json` (`bundle.icon`) | 🟡 | Por qué 32 px pixela y 512 px desaparece |
| Marcas registradas y uso descriptivo | `README.md`, Decisión 12 | ❌ | "Mirlo, para posts de X" sí; "TwitterVault" no |
| Legibilidad por tamaños | `src-tauri/icons/` | ❌ | Un icono se diseña para 16 px tanto como para 512 |

## 🌍 Zona 7: dos idiomas (i18n)

| Concepto | Dónde vive | Estado | En una frase |
|---|---|---|---|
| Separación de responsabilidades / códigos de mensaje | `error.rs` + `i18n.svelte.ts` | 🟡 | Rust dice QUÉ pasó; la interfaz decide CÓMO contarlo |
| Diccionario comprobado por tipos | `Record<Key, string>` en `i18n.svelte.ts` | ❌ | Si falta una traducción, TypeScript no compila |
| `Intl` (fechas, números, plurales) | `formatDate`, `formatSize`, `tn` | ❌ | El navegador ya sabe que en inglés es "May 4" y en español "4 may" |
| Comprobaciones propias en el CI | `scripts/check-i18n.mjs` | ❌ | Un script de 20 líneas que vigila algo que ningún compilador ve |

---

## 🧭 Próximas misiones (orden recomendado)

Tu stack habitual es Kotlin. Muchas de estas piezas tienen equivalente directo allí, y te lo indico para que conectes lo nuevo con lo que ya conoces.

1. **`Result<T, E>` y el operador `?`.** Es la base de todo el Rust de Mirlo. *En Kotlin: `Result` y `runCatching`, pero en Rust no hay excepciones.*
2. **Comandos Tauri de punta a punta.** Seguir un clic desde `App.svelte` → `api.ts` → `main.rs` → `vault.rs` → disco. *Como un ViewModel llamando a un Repository.*
3. **Runes de Svelte 5.** `$state` / `$derived` / `$effect`. *Equivalen a `mutableStateOf` / `derivedStateOf` / `LaunchedEffect` en Compose.*
4. **`async`/`await` en Rust.** *Las corrutinas de Kotlin con otro nombre; `spawn_blocking` es `Dispatchers.IO`.*
5. **Escritura atómica + SHA-256.** Dos ideas pequeñas que sirven en cualquier app que guarde archivos, también en Android.
6. **Consultas parametrizadas y triggers.** *Room hace las dos cosas por ti; aquí ves cómo funcionan por dentro.*
7. **CD con `release.yml`.** La misión que viene ahora: la primera release.

## 🏆 Jefe final de esta temporada

**Publicar Mirlo v0.1.0.** Es la etiqueta de versión, los instaladores compilados por GitHub, las notas de la release y la prueba del `.deb`. Toca SemVer, CI/CD y verificación de paquetes: tres zonas a la vez.
