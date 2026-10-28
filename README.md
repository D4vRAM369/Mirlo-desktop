<p align="center">
  <img src="docs/assets/banner.png" width="900" alt="Mirlo: tu bóveda local de posts de X. Icono de un cajón de fichero azul con un post en modo oscuro y un mirlo posado encima.">
</p>

<p align="center"><b>Español</b> · <a href="README-EN.md">English</a></p>

Bóveda local para posts de X (Twitter). Pegas el enlace de un post y Mirlo guarda en tu disco el texto, el autor, la fecha y todos sus archivos: fotos a resolución original, vídeos, GIF (como MP4) y documentos enlazados (PDF, DOCX…).

Sin cuenta, sin nube y sin telemetría.

> La interfaz está en español y en inglés. La primera vez Mirlo usa el idioma del sistema; puedes cambiarlo cuando quieras con el botón que hay junto al del tema.

## Cómo se usa

1. Copia el enlace de un post en X.
2. Pulsa <kbd>Ctrl</kbd>+<kbd>V</kbd> en cualquier parte de la ventana de Mirlo.
3. El post aparece en la lista. Si algo no se pudo bajar, se marca como **◐ Parcial** o **✕ Error** y puedes reintentarlo con <kbd>R</kbd>.

| Atajo | Acción |
|---|---|
| <kbd>J</kbd> / <kbd>K</kbd> | Siguiente y anterior |
| <kbd>/</kbd> | Buscar (texto completo, sin distinguir acentos) |
| <kbd>Ctrl</kbd>+<kbd>V</kbd> | Guardar el enlace copiado |
| <kbd>O</kbd> | Abrir la carpeta del post |
| <kbd>R</kbd> | Reintentar un post parcial o con error |

## Dónde se guarda

```
~/Mirlo/
├── mirlo.db                          índice para buscar y filtrar
└── <usuario>/<YYYY-MM-DD>_<id>/
    ├── post.json                     post, archivos con su SHA-256 y la respuesta original
    ├── post.md                       el post en texto legible
    ├── media/01.jpg, 02.mp4…         en el orden del post (q01… = post citado)
    └── attachments/                  documentos enlazados
```

Las carpetas son la copia de verdad: puedes copiarlas, leerlas sin Mirlo o hacer copia de seguridad de `~/Mirlo` entera.

## Qué sale de tu equipo

Mirlo solo hace estas peticiones, y solo cuando guardas o reintentas un post:

- **X** (`cdn.syndication.twimg.com`, `publish.x.com`, `pbs.twimg.com`, `video.twimg.com`): el post y sus archivos.
- **yt-dlp** (incluido en la app): pide el post a X cuando la vía anterior falla.
- **FxTwitter** (`api.fxtwitter.com`): **solo** para posts de más de 280 caracteres, porque X los entrega cortados. Recibe el número del post.
- **Los sitios enlazados en el post**: Mirlo mira si el enlace es un documento (por su `Content-Type`) y, si lo es y pesa menos de 50 MB, lo descarga. Nunca toca direcciones de tu red local.

La tipografía va dentro de la app: abrir Mirlo no llama a nadie.

## Limitaciones

- No guarda hilos completos ni posts protegidos o que requieren iniciar sesión.
- Si FxTwitter no responde, un post largo queda como parcial (texto cortado por X).

## Desarrollo

Requisitos: Rust, Node 20+ y las [dependencias de Tauri 2](https://tauri.app/start/prerequisites/) del sistema.

```bash
npm install
npm run sidecar        # descarga yt-dlp para tu plataforma y comprueba su SHA-256
npm run tauri dev
```

```bash
npm run check                                   # tipos de TypeScript y Svelte
cd src-tauri && cargo test                      # tests con respuestas reales guardadas
cd src-tauri && cargo test -- --ignored         # pruebas en vivo contra X
cd src-tauri && cargo clippy --all-targets -- -D warnings
npm run tauri build                             # paquetes .deb, .rpm y AppImage
```

La documentación del proceso está en `docs/chronicler/` (decisiones, errores y soluciones, paso a paso).
