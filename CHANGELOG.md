# Changelog

Formato: [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/) · Versiones: [SemVer](https://semver.org/lang/es/).

## [0.2.0] — 2026-11-22

Enviar desde X. · *Send from X.*

### Español

- **Enviar a Mirlo** en el menú Compartir de X, con el userscript `userscript/mirlo-x.user.js`.
- Enlaces `mirlo://save/<id>`: guardan con Mirlo abierto, oculto o cerrado. Solo aceptan un número de post.
- Los envíos se guardan de uno en uno, sin quitarte el foco ni cambiar el post que tienes abierto. Si uno falla, aparece la ventana con el error.
- Una sola instancia: volver a abrir Mirlo muestra la ventana que ya había.
- Icono en la bandeja del sistema: al minimizar, Mirlo se esconde ahí.
- Para desarrollo: una PR semanal avisa cuando sale un yt-dlp nuevo.
- La documentación interna (`docs/`) deja de publicarse en el repositorio.

### English

- **Send to Mirlo** in X's Share menu, through the `userscript/mirlo-x.user.js` userscript.
- `mirlo://save/<id>` links: they save with Mirlo open, hidden or closed, and accept nothing but a post number.
- Incoming posts are saved one at a time, without stealing focus or changing the post you have open. If one fails, the window appears with the error.
- Single instance: opening Mirlo again shows the window that was already running.
- System tray icon: minimizing hides Mirlo there.
- For development: a weekly PR flags new yt-dlp versions.
- Internal documentation (`docs/`) is no longer published in the repository.

### Limitaciones conocidas · *Known limitations*

- El userscript depende de la página de X y puede necesitar ajustes cuando X cambie. · *The userscript depends on X's page and may need fixes when X changes.*
- Probado en Linux con Chromium y X sin iniciar sesión; sin probar con sesión iniciada, con gestores de userscripts ni en Firefox. · *Tested on Linux with Chromium and X signed out; untested while signed in, with userscript managers, or on Firefox.*
- La AppImage todavía no registra `mirlo://`: usa `.deb` o `.rpm`. · *The AppImage does not register `mirlo://` yet: use `.deb` or `.rpm`.*
- Windows y macOS sin probar, con instaladores sin firmar. · *Windows and macOS are untested and their installers are unsigned.*

## [0.1.0] — 2026-10-28

Primera versión pública. · *First public release.*

### Español

- Guarda posts de X pegando su enlace (<kbd>Ctrl</kbd>+<kbd>V</kbd> en cualquier parte de la ventana): texto, autor, fecha, fotos a resolución original, vídeos, GIF (como MP4), el post citado y documentos enlazados (PDF, DOCX…, hasta 50 MB).
- Texto completo de los posts largos (más de 280 caracteres).
- Todo en tu disco, en `~/Mirlo/<usuario>/<fecha>_<id>/`, con `post.json`, `post.md` y la huella SHA-256 de cada archivo.
- Búsqueda de texto completo sin distinguir acentos, filtros por tipo, autor, estado y fecha.
- Posts parciales o con error marcados y con reintento (<kbd>R</kbd>).
- Organiza tus posts: favoritos (<kbd>F</kbd>), colecciones (<kbd>M</kbd>), etiquetas con color (<kbd>T</kbd>; 8 tonos o cualquier color con la rueda) y notas personales (<kbd>N</kbd>). Las notas y etiquetas entran en la búsqueda, y nada de esto se pierde al volver a guardar un post.
- Copias de seguridad: exporta la bóveda a un `.zip` (con o sin fotos y vídeos) e impórtala en otro equipo. Importar fusiona y nunca sobrescribe: suma etiquetas y favoritos, conserva tus notas y colecciones, y comprueba la huella de cada archivo.
- Interfaz en español e inglés, tema claro y oscuro, manejo completo con teclado.
- yt-dlp incluido.
- Licencia GPL-3.0 o posterior.

### English

- Saves X posts by pasting their link (<kbd>Ctrl</kbd>+<kbd>V</kbd> anywhere in the window): text, author, date, photos at original resolution, videos, GIFs (as MP4), the quoted post and linked documents (PDF, DOCX…, up to 50 MB).
- Full text of long posts (over 280 characters).
- Everything on your disk, in `~/Mirlo/<user>/<date>_<id>/`, with `post.json`, `post.md` and the SHA-256 fingerprint of every file.
- Accent-insensitive full-text search, filters by type, author, status and date.
- Partial or failed posts are flagged and can be retried (<kbd>R</kbd>).
- Organize your posts: favorites (<kbd>F</kbd>), collections (<kbd>M</kbd>), colored tags (<kbd>T</kbd>; 8 tones or any color from the wheel) and personal notes (<kbd>N</kbd>). Notes and tags are searchable, and saving a post again never loses them.
- Backups: export the vault to a `.zip` (with or without photos and videos) and import it on another computer. Importing merges and never overwrites: it adds tags and favorites, keeps your notes and collections, and checks the fingerprint of every file.
- Spanish and English interface, light and dark themes, full keyboard control.
- yt-dlp bundled.
- Licensed under GPL-3.0 or later.

### Limitaciones conocidas · *Known limitations*

- No guarda hilos completos ni posts protegidos. · *No full threads or protected posts.*
- Probado en Linux (Ubuntu, X11). Windows y macOS se compilan automáticamente pero no se han probado, y los instaladores no están firmados: el sistema mostrará un aviso de seguridad. · *Tested on Linux (Ubuntu, X11). Windows and macOS builds are automatic but untested, and installers are unsigned: the OS will show a security warning.*
