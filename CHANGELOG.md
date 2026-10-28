# Changelog

Formato: [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/) · Versiones: [SemVer](https://semver.org/lang/es/).

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
