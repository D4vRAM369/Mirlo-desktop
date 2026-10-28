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
