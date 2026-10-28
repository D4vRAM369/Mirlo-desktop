# Paso a paso

## Sesión 1 — 2026-10-28

### Paso 0: Investigar antes de construir

**Qué vamos a hacer:** Leer ThreadsVault-desktop para saber qué reutilizar y por qué falla su extractor, y comprobar que la fuente principal de XVault funciona de verdad.

**Por qué lo hacemos así:** Si copiamos el extractor de ThreadsVault, copiamos su bug. Y si diseñamos la app alrededor de un endpoint que no funciona, perdemos días.

**Resultado esperado:** Causa raíz del bug escrita y endpoint de syndication probado con peticiones reales.

#### Instrucciones

1. Leer `src/lib/utils/post-extractor.ts` de ThreadsVault (1433 líneas), en especial `extractPostData` (línea 1140) y los extractores de texto.
2. Probar el endpoint con el post nº 20 de X (el primer tweet público) y con un post con vídeo:

```bash
# Calculamos el token a partir del ID (fórmula de los embeds oficiales)
ID=20
TOKEN=$(node -e "console.log(((Number('$ID')/1e15)*Math.PI).toString(36).replace(/(0+|\.)/g,''))")
# Pedimos el JSON del post; -w imprime el código HTTP al final
curl -s -w "\nHTTP %{http_code}\n" "https://cdn.syndication.twimg.com/tweet-result?id=$ID&token=$TOKEN"
```

> 💡 **Nota para aprendices:** Sin `token` el servidor responde `200 OK` pero con `{}` vacío. Un 200 no significa "todo bien": hay que comprobar que el contenido tiene lo que esperas.

**Verificación:** El JSON contiene `"text":"just setting up my twttr"` y, en el post con vídeo, `mediaDetails[0].video_info.variants` con varias calidades MP4.

### Paso 1: Mockups visuales antes de la UI

**Qué vamos a hacer:** Tres mockups estáticos en HTML (`mockups/`) con enfoques distintos, para elegir dirección antes de escribir componentes Svelte.

**Por qué lo hacemos así:** Cambiar un HTML suelto cuesta minutos; cambiar una UI ya conectada a la base de datos cuesta días. Decidir el diseño con algo que se puede tocar (clic, teclado, tema oscuro) es más fiable que con una descripción.

| Mockup | Idea | Tipografía | Visor de medios |
|---|---|---|---|
| `a-mesa-de-luz.html` | Medios primero: cada post es una tira de negativos; el estado se marca con "lápiz graso" | Archivo (juega con su anchura) | Panel lateral "lupa" |
| `b-bandeja.html` | Herramienta de lectura de tres paneles, atajos de teclado visibles | Atkinson Hyperlegible Next | Lightbox (`<dialog>`) |
| `c-registro.html` | Texto primero: río cronológico por meses, cada post se lee como una cita | Literata + Schibsted Grotesk | Lightbox con reproductor |

Para verlos: `python3 -m http.server 4317 --directory mockups` y abrir `http://localhost:4317`.

> 💡 **Nota para aprendices:** Probamos los tres a 360 px de ancho buscando scroll horizontal. Dos fallaban por lo mismo: en CSS Grid, `1fr` significa en realidad `minmax(auto, 1fr)`, así que una columna nunca encoge por debajo de su contenido más ancho. La solución es `minmax(0, 1fr)`.

**Verificación:** `scrollWidth - clientWidth = 0` en los tres a 360 px y 1280 px.

### Paso 2: Fase 1, extractor con TDD

**Qué vamos a hacer:** Crear el proyecto y un extractor que convierta un enlace de X en datos limpios, probado con respuestas reales.

**Por qué lo hacemos así:** El fallo de ThreadsVault estaba en la extracción. Si esta pieza está probada contra casos reales, todo lo demás (descargas, BD, UI) se apoya en algo fiable.

**Resultado esperado:** `cargo test` con 17 tests en verde y 2 pruebas en vivo contra X.

#### Instrucciones

1. **Scaffold a mano.** Copiamos de ThreadsVault solo la config (`tsconfig*`, `vite.config.ts`, `svelte.config.js`, `build.rs`, iconos) y escribimos `package.json`, `Cargo.toml` y `tauri.conf.json` mínimos. `git init`.
2. **Fixtures reales.** Descargamos 9 posts con el endpoint de syndication y los guardamos en `tests/fixtures/`. Añadimos `empty.json` (`{}`), la respuesta sin token o de un post borrado, y fixtures de yt-dlp y oEmbed.
3. **Tests en rojo.** `parser.rs` con los tipos (`Post`, `Media`, `Link`) y `parse_tweet` devolviendo siempre error. `cargo test` → 14 fallos. Correcto: aún no hay código.
4. **Implementación.** El corazón es `clean_text`:

```rust
// 1) recortar al rango visible (en code points, no bytes: los emoji cuentan 1)
let chars: Vec<char> = text.chars().collect();
let mut out: String = chars[from..to].iter().collect();
// 2) quitar el t.co de las fotos: en posts antiguos está DENTRO del rango
for m in &entities.media { out = out.replace(&m.url, ""); }
// 3) t.co → URL real
for u in &entities.urls { out = out.replace(&u.url, &u.expanded_url); }
// 4) &amp; → &, etc.
decode_entities(out.trim())
```

5. **Cadena de fuentes** (`sources.rs`): syndication → yt-dlp → oEmbed. Cada intento fallido se apunta y, si fallan todos, el error dice qué pasó con cada fuente.
6. **Comprobación final:** `cargo clippy -- -D warnings`, `npm run check` y `npm run build`, todo limpio.

> 💡 **Nota para aprendices:** Un test "invariante" vale por diez: `no_fixture_leaks_tco_links_into_text` recorre TODOS los fixtures y falla si se cuela un `t.co`. Cada fixture nuevo queda protegido sin escribir un test nuevo.

> ⚠️ **Errores comunes:** Suponer que un campo significa lo que su nombre parece. En yt-dlp, `id` es el ID del vídeo, no del post (ver `05_ERRORES_Y_SOLUCIONES.md`).

**Verificación:**
```bash
cd src-tauri
cargo test                 # 17 passed, 2 ignored
cargo test -- --ignored    # 2 pruebas en vivo contra X
```


## Paso 3: texto completo con FxTwitter (2026-10-28)

1. Descargar la respuesta real de FxTwitter para el post largo de rauchg y guardar un fixture recortado (`fxtwitter_long.json`).
2. Test primero: `fxtwitter_gives_full_text_of_long_post` exige el texto completo, enlaces expandidos y rechazo si el ID no coincide o llega HTML. Rojo.
3. `parser::parse_fxtwitter_text` y su uso en `sources::extract` solo cuando `text_truncated`. Verde.
4. Prueba en vivo `live_long_post_gets_full_text`: `source = "syndication+fxtwitter"`, `partial = false`.

## Paso 4: guardar en disco (fase 2)

1. Mirar los datos reales: el PDF del fixture `pdf_link` no termina en `.pdf`. Decisión: Content-Type manda.
2. Módulo nuevo `vault.rs`. Funciones puras con test: `post_dir`, `media_name`, `attachment_name`, `doc_ext`, `is_public_ip`, `render_markdown`.
3. La descarga se prueba contra un **servidor HTTP de usar y tirar** en `127.0.0.1` dentro del propio test: comprueba el `.part`, el SHA-256 de `"abc"`, el límite de tamaño y que un 404 no se reintenta.
4. Comando `save_post` en `main.rs` y botón "Guardar post" en la pantalla provisional.
5. Prueba en vivo con 5 posts reales (4 fotos, vídeo, GIF, cita, PDF): todos `ok`, ningún `.part` sobrante, `sha256sum` coincide con `post.json`, la foto conserva 5568x3712.

> 💡 **Nota para aprendices:** No hace falta internet para probar una descarga. Un `TcpListener` en el puerto 0 (el sistema elige uno libre) que responde texto fijo es un servidor real para `reqwest`.

> ⚠️ **Errores comunes:** Escribir directamente en el nombre final. Si la descarga se corta, queda un archivo roto que parece bueno. Con `.part` + `rename`, el nombre final solo aparece cuando está completo.

**Verificación:**
```bash
cd src-tauri
cargo test                                      # 26 passed, 4 ignored
cargo test live_save -- --ignored --nocapture   # guarda 5 posts reales en target/test-tmp/live
```


## Paso 5: BD + UI básica (fase 3)

1. `db.rs` con tests en memoria: búsqueda por prefijo, sin acentos, texto del citado, sintaxis hostil (`"`, `OR`, `NEAR(`), filtros por tipo/autor/estado/fecha, re-guardar reemplaza archivos e índice, un error no pisa una copia guardada.
2. Comandos en `main.rs`: `save_post`, `retry_post`, `list_posts`, `get_post`, `facets`, `open_folder`, `open_on_x`. Estado compartido `App { root, db: Mutex<Connection> }`.
3. `src/lib/api.ts`: un wrapper tipado por comando. El frontend no llama a `invoke` en ningún otro sitio.
4. `App.svelte` con el esqueleto Bandeja: un solo campo (un enlace guarda, lo demás busca), filtros, lista navegable con J/K, lectura con fotos y vídeos locales (protocolo `asset:`), tabla de archivos y fuente.
5. Ctrl+V fuera de un campo de texto guarda el enlace directamente.
6. `npm run tauri dev`: la ventana abre y crea `~/XVault/xvault.db`.

> 💡 **Nota para aprendices:** FTS5 no busca dentro del texto como `LIKE '%x%'`: mantiene un índice de palabras, como el índice al final de un libro. Por eso es rápido con miles de posts.

> ⚠️ **Errores comunes:** pasar lo que escribe la persona directamente a `MATCH`. `"` o `NEAR(` son sintaxis de FTS5 y romperían la consulta. `fts_query` mete cada palabra entre comillas.

**Verificación:**
```bash
cd src-tauri && cargo test     # 31 passed, 4 ignored
npm run tauri dev               # pega un enlace de X con Ctrl+V en la ventana
```


## Paso 6: diseño final (fase 4)

1. Tokens de color, tema oscuro y fuente local en `src/app.css`; Tailwind desinstalado.
2. `src/lib/Gallery.svelte`: rejilla 2fr/1fr con "+N", visor `<dialog>` con ←/→, contador y texto alternativo; vídeos inline como blob.
3. `App.svelte` reescrito con las clases del mockup: cola de guardado, título según filtro, filtro de fecha plegable con "Quitar fechas", estados vacíos que explican qué hacer.
4. Columna `alt` en `files` + migración `user_version = 1` (test `old_database_gets_alt_column_and_keeps_rows`).
5. Revisión visual con capturas headless a 1280x840 y 960x640, en claro y oscuro (ver "Cómo revisar el diseño" en `07_COMANDOS_Y_MANUAL.md`).

> 💡 **Nota para aprendices:** `PRAGMA user_version` es un número que SQLite guarda dentro del archivo y no usa para nada: es tuyo. Sirve para saber qué cambios de esquema ya se aplicaron.


## Paso 7: empaquetado (fase 5)

1. `scripts/fetch-yt-dlp.mjs` + `npm run sidecar`; `src-tauri/binaries/` en `.gitignore`.
2. `externalBin` en `tauri.conf.json` y el script en `beforeDevCommand` y `beforeBuildCommand`.
3. Test en vivo `live_ytdlp_fallback_downloads_video`: la rama de respaldo de vídeo, sin probar desde la fase 2, descarga un MP4 válido con su SHA-256 y sin dejar el temporal.
4. `npm run tauri build` → paquetes en `src-tauri/target/release/bundle/`.
5. Plan para llevar el extractor a ThreadsVault (documento de trabajo local, no versionado) y `README.md`.


## Paso 8: interfaz en dos idiomas (2026-10-29)

1. Inventario: ~55 textos en Svelte y ~35 mensajes en Rust.
2. Tests primero: `errors_are_codes_not_sentences` (x_url) y `markdown_follows_the_app_language` (vault); los tests de descarga pasan a esperar `http: 404 Not Found` y `too_big: 1`.
3. `src-tauri/src/error.rs` con `fail` / `fail_with`; todos los `Err("frase")` pasan a códigos.
4. `src/lib/i18n.svelte.ts`: diccionarios, `t()`, `tn()` (plurales), `errorParts()`, `formatDate()`, `formatSize()`, `withKeys()`.
5. `App.svelte` y `Gallery.svelte` sin textos fijos; botón de idioma junto al de tema.
6. `scripts/check-i18n.mjs` en `npm run check`; probado con un código inventado (falla con salida 1).
7. Revisión con capturas en es/en, claro/oscuro, visor y post con error.

## Paso 9: preparar la release y limpiar el historial (2026-10-29)

1. `CHANGELOG.md` bilingüe; texto de la Release con aviso de instaladores sin firmar.
2. Prueba del `.deb` sin instalarlo: `dpkg-deb -x` a una carpeta temporal y ejecutar `usr/bin/mirlo` → ventana, icono 256 px (`xprop`), `mirlo-yt-dlp --version`.
3. Etiqueta `v0.1.0` subida → se decidió añadir funciones antes → `gh run cancel`, borrar la etiqueta en GitHub (`git push origin :refs/tags/v0.1.0`) y en local.
4. Limpieza de datos personales: primero los archivos actuales, después el historial con `git filter-repo` (ver `07_COMANDOS_Y_MANUAL.md`).
5. `docs/ROADMAP.md` (público) y documentos de trabajo locales, no versionados, para retomar en otra sesión.

---

## Sesión 1 — resumen (2026-10-28 / 2026-10-29)

📋 **RESUMEN DE SESIÓN**

**PROYECTO:** Mirlo (antes XVault)

✅ **LO QUE LOGRAMOS:**
- Extractor de posts de X con TDD, sin regex sobre HTML, con cadena de fuentes y texto completo de posts largos.
- Descargas atómicas con SHA-256, documentos por Content-Type y protección SSRF.
- SQLite con búsqueda FTS5, filtros y reintento; interfaz "Bandeja" con tema claro/oscuro y teclado.
- Empaquetado `.deb`/`.rpm`/AppImage con yt-dlp, CI y workflow de release.
- Icono y nombre propios, banner y README en dos idiomas, interfaz en español e inglés.
- Historial de Git limpio de datos personales.

📚 **CONCEPTOS APRENDIDOS:** ver `../profesoria/CONCEPTOS.md` (10 conceptos) y `../profesoria/HOJA_DE_RUTA.md` (mapa completo).

🔜 **PRÓXIMOS PASOS:** organizar (favoritos, colecciones, etiquetas, notas) → exportar/importar → release v0.1.0 (`../ROADMAP.md`).

❓ **PENDIENTES:** botón de idioma sin probar en la ventana real; Windows y macOS sin probar.

## Paso 10: organizar los posts (2026-10-28)

1. Esquema v1 congelado en `tests/fixtures/schema_v1.sql` y test de migración en rojo (`v1_database_migrates_to_v2_and_keeps_rows`).
2. Esquema v2: tablas `collections`, `tags`, `post_tags`; columnas `collection_id`, `favorite`, `note`, `tags_text`; FTS rehecho con 6 columnas.
3. `organize.rs` con los tests primero (funciones en `todo!()`): re-guardar no pisa nada, nombres únicos sin mayúsculas, errores por id desconocido, renombrar/borrar actualiza la búsqueda, orden manual de colecciones, filtros y recuentos, bloque `user` por nombre.
4. `vault::write_user` (atómico) y `vault::save` conserva el bloque `user` del post.json anterior.
5. 12 comandos Tauri; cada cambio va a la BD y luego a post.json (`sync_user`).
6. Migración probada contra una **copia** de la bóveda real: `MIRLO_DB=<copia> cargo test real_db -- --ignored` (`user_version 1 → 2`, filas intactas, `integrity-check` del FTS correcto).
7. Interfaz: sidebar (Favoritos, Colecciones, Etiquetas), panel de lectura (estrella, colección, etiquetas, nota), atajos F/T/M/N, editor en línea con doble clic o F2, Alt+↑/↓.
8. Rueda de color HEX (9.º círculo) para colores fuera de la paleta.
9. Verificación en la ventana real (WebKitGTK) con una bóveda de pruebas: `HOME=<carpeta de pruebas> src-tauri/target/release/mirlo`, `xdotool` para teclear y `import -window` para capturar.
