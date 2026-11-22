# Changelog

Formato: [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/) · Versiones: [SemVer](https://semver.org/lang/es/).

## [0.3.0] — 2026-11-22

Mirlo trae el script de X y te guía para instalarlo, sin buscarlo en el README. · *Mirlo brings the X script and guides you through installing it, no README digging.*

### Español

- **Integrar con X**: Mirlo lo ofrece una vez y queda siempre en la barra superior. Guía la instalación del gestor y del script en cada navegador.
- El userscript va dentro de la app: se instala desde una dirección local y temporal o se copia.
- **Comprobar**: «Conectado con X» solo aparece cuando llega el saludo del script, con un código de un solo uso.
- La AppImage puede registrar los enlaces `mirlo://`, pidiendo permiso antes.
- Userscript 0.2.0, con el saludo. Si tienes el 0.1.0, reinstálalo con **Instalar script**: se actualiza.

#### Cómo empezar

1. Sal de Mirlo desde el icono de la bandeja si está abierto e instala la 0.3.0. Tu bóveda sigue en `~/Mirlo`.
2. Pulsa **Integrar con X**, elige tu navegador y sigue los pasos. Lo que se instala lo confirmas tú en el navegador.
3. Pulsa **Comprobar** y, en X, **Conectar con Mirlo**. Después, en cualquier post: **Compartir → Enviar a Mirlo**.

### English

- **Integrate with X**: Mirlo offers it once and it always stays in the top bar. It guides you through installing the manager and the script in each browser.
- The userscript ships inside the app: install it from a temporary local address or copy it.
- **Check**: "Connected to X" only appears when the script's greeting arrives, with a single-use code.
- The AppImage can register `mirlo://` links, asking for permission first.
- Userscript 0.2.0, with the greeting. If you have 0.1.0, reinstall it with **Install script**: it gets updated.

#### Getting started

1. Quit Mirlo from its tray icon if it is running, then install 0.3.0. Your vault stays in `~/Mirlo`.
2. Click **Integrate with X**, pick your browser and follow the steps. You confirm every install in the browser.
3. Click **Check** and, on X, **Connect to Mirlo**. Then, on any post: **Share → Send to Mirlo**.

### Limitaciones conocidas · *Known limitations*

- Probado en Linux con X sin iniciar sesión: el `.deb`, de principio a fin en Firefox con Violentmonkey; la AppImage, el registro de enlaces y el saludo; en Chromium, la pantalla de instalación de Tampermonkey y la comprobación cargando el script a mano. · *Tested on Linux with X signed out: the `.deb` end to end on Firefox with Violentmonkey; the AppImage, link registration and the greeting; on Chromium, Tampermonkey's install screen and the check with the script loaded by hand.*
- Sin probar: X con sesión iniciada, Tampermonkey con su permiso activado, `.rpm`, Windows y macOS. · *Untested: signed-in X, Tampermonkey with its permission turned on, `.rpm`, Windows and macOS.*
- Con la AppImage en una ruta con espacios, algunos escritorios sin GNOME ni KDE no abren `mirlo://`. · *With the AppImage in a path with spaces, some desktops other than GNOME and KDE do not open `mirlo://`.*
- La comprobación necesita Mirlo abierto, y al terminar algunos escritorios solo lo resaltan en la barra en vez de traerlo al frente. · *The check needs Mirlo running, and when it finishes some desktops only highlight it in the taskbar instead of bringing it to the front.*

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
