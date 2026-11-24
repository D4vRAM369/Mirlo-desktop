// Textos de la interfaz en español e inglés.
// `en` se declara con las claves de `es`: si a un idioma le falta una clave, TypeScript no compila.
// Los errores llegan de Rust como `codigo` o `codigo: detalle` (src-tauri/src/error.rs) y se traducen
// aquí con las claves `err.<codigo>`; `npm run check` comprueba que ningún código de Rust se queda sin traducir.
// Sintaxis de los textos: {nombre} = dato que se rellena; [Ctrl] = tecla (se pinta como <kbd>).

const es = {
  'cmd.placeholder': 'Pega un enlace de X para guardarlo, o escribe para buscar',
  'cmd.label': 'Enlace o búsqueda',
  'cmd.save': 'Guardar post',
  'cmd.theme_light': 'Tema claro',
  'cmd.theme_dark': 'Tema oscuro',
  'cmd.other_lang': 'English',

  'side.label': 'Filtros',
  'side.all': 'Todos',
  'side.partial': 'Parciales',
  'side.error': 'Con error',
  'side.by_type': 'Por tipo',
  'side.authors': 'Autores',
  'side.date': 'Fecha del post: {range}',
  'side.date_any': 'cualquiera',
  'side.date_range': '{from} a {to}',
  'side.from': 'Desde',
  'side.to': 'Hasta',
  'side.clear_dates': 'Quitar fechas',
  'side.favorites': 'Favoritos',
  'side.collections': 'Colecciones',
  'side.new_collection': 'Nueva colección',
  'side.tags': 'Etiquetas',
  'side.no_tags': 'Añade etiquetas desde el post abierto con [T].',
  'side.edit_hint': 'Doble clic o [F2]: renombrar. [Alt] [↑] [↓]: mover.',
  'side.edit_hint_tag': 'Doble clic o [F2]: renombrar, color o borrar.',

  'edit.name': 'Nombre',
  'edit.delete': 'Borrar',
  'edit.confirm_delete': 'Sí, borrar',
  'edit.delete_collection_hint': 'Los posts no se borran: se quedan sin colección.',
  'edit.delete_tag_hint': 'Se quita de todos sus posts.',
  'edit.cancel': 'Cancelar',
  'edit.done': 'Hecho',
  'edit.color': 'Color',
  'edit.no_color': 'Sin color',
  'edit.custom_color': 'Otro color…',
  'edit.brightness': 'Brillo',
  'edit.hex': 'Color en HEX',
  'edit.use_color': 'Usar',
  'color.red': 'Rojo',
  'color.orange': 'Naranja',
  'color.yellow': 'Amarillo',
  'color.green': 'Verde',
  'color.teal': 'Turquesa',
  'color.blue': 'Azul',
  'color.purple': 'Morado',
  'color.pink': 'Rosa',
  'org.favorite': 'Favorito',
  'org.add_favorite': 'Añadir a favoritos',
  'org.remove_favorite': 'Quitar de favoritos',
  'org.collection': 'Colección',
  'org.no_collection': 'Sin colección',
  'org.tags': 'Etiquetas',
  'org.tag_input': 'Añadir etiqueta',
  'org.tag_placeholder': 'Escribe y pulsa Intro',
  'org.remove_tag': 'Quitar {name}',
  'org.note': 'Nota',
  'org.note_placeholder': 'Solo para ti. También entra en la búsqueda.',

  'kind.video': 'Con vídeo',
  'kind.photo': 'Con fotos',
  'kind.gif': 'Con GIF',
  'kind.document': 'Con documentos',
  'kind.text': 'Solo texto',

  'list.label': 'Posts',
  'list.items_label': 'Posts guardados',
  'list.newest': 'Más recientes primero',
  'list.searching': 'Buscando «{q}»',
  'list.saving_user': 'Guardando el post de @{user}…',
  'list.saving': 'Guardando post…',
  'list.no_match': 'Nada coincide con estos filtros.',
  'list.empty_collection': 'Esta colección está vacía. Abre un post y pulsa [M] para elegir su colección.',
  'list.empty': 'Tu bóveda está vacía. Copia el enlace de un post de X y pulsa [Ctrl] [V] en cualquier parte de esta ventana.',
  'list.post_id': 'Post {id}',
  'list.no_date': 'Sin fecha',

  'media.photo': 'Fotos',
  'media.video': 'Vídeo',
  'media.gif': 'GIF',
  'media.document': 'Documento',
  'media.text': 'Texto',

  'status.ok': '✓ Completo',
  'status.partial': '◐ Parcial',
  'status.error': '✕ Error',

  'read.label': 'Post abierto',
  'read.retry': 'Reintentar',
  'read.open_folder': 'Abrir carpeta',
  'read.view_on_x': 'Ver en X',
  'read.quoting': 'Cita a @{user}',
  'read.files': 'Archivos en disco',
  'read.file': 'Archivo',
  'read.state': 'Estado',
  'read.size': 'Tamaño',
  'read.downloaded': '✓ Descargado',
  'read.skipped': 'Omitido: {why}',
  'read.source': 'Fuente: {source}',
  'read.source_none': 'ninguna',
  'read.details': 'Detalles: {detail}',
  'read.pick': 'Elige un post de la lista con [J] y [K], o con el ratón.',
  'read.how_to_save': 'Para guardar uno nuevo, copia su enlace en X y pulsa [Ctrl] [V] aquí.',

  'keys.label': 'Atajos de teclado',
  'keys.nav': 'siguiente y anterior',
  'keys.search': 'buscar',
  'keys.save': 'guardar enlace',
  'keys.open': 'abrir carpeta',
  'keys.retry': 'reintentar',
  'keys.favorite': 'favorito',
  'keys.tags': 'etiquetas',
  'keys.collection': 'colección',
  'keys.note': 'nota',

  'msg.saved': 'Guardado: @{user}',
  'msg.saved_partial': 'Guardado con avisos: @{user}',
  'msg.save_failed': 'No se pudo guardar: {error}',
  'msg.retrying': 'Reintentando…',
  'msg.retry_ok': 'Reintento correcto: ya está completo',
  'msg.retry_partial': 'Reintentado, pero sigue incompleto',
  'msg.retry_failed': 'Sigue fallando: {error}',

  'gallery.loading_video': 'Cargando vídeo…',
  'gallery.view_full': 'Ver a pantalla completa: {alt}',
  'gallery.photo_alt': 'Foto {n} de {total}, sin descripción del autor',
  'gallery.hint.one': '1 foto a resolución original. Pulsa para verla a pantalla completa.',
  'gallery.hint.other': '{n} fotos a resolución original. Pulsa una para verla a pantalla completa.',
  'gallery.viewer': 'Visor de fotos',
  'gallery.counter': '{n} de {total}',
  'gallery.prev': 'Anterior',
  'gallery.next': 'Siguiente',
  'gallery.close': 'Cerrar',

  'err.invalid_link': 'No es un enlace válido.',
  'err.link_scheme': 'El enlace debe empezar por https://',
  'err.host_not_allowed': 'Solo se aceptan enlaces de x.com o twitter.com.',
  'err.not_a_post': 'El enlace no apunta a un post (falta /status/).',
  'err.invalid_post_id': 'El número del post no es válido.',
  'err.offline': 'Sin conexión: no se pudo contactar con X.',
  'err.post_unavailable': 'No se pudo leer el post. Puede estar borrado o ser privado.',
  'err.network': 'Se cortó la conexión.',
  'err.http': 'El servidor respondió con un error.',
  'err.too_big': 'Supera {detail} MB',
  'err.media_host_not_allowed': 'El archivo no está en los servidores de X.',
  'err.video_unavailable': 'No se pudo descargar el vídeo, ni directamente ni con yt-dlp.',
  'err.disk_write': 'No se pudo escribir en el disco.',
  'err.db_locked': 'La base de datos quedó bloqueada. Reinicia Mirlo.',
  'err.database': 'Error de la base de datos.',
  'err.not_found': 'Ese post no está en la bóveda.',
  'err.no_folder': 'Este post no tiene carpeta todavía.',
  'err.folder_missing': 'La carpeta ya no existe.',
  'err.unknown_file': 'Ese archivo no pertenece al post.',
  'err.outside_vault': 'Esa ruta está fuera de la bóveda.',
  'err.read_file': 'No se pudo leer el archivo.',
  'err.open_failed': 'No se pudo abrir.',
  'err.name_empty': 'Escribe un nombre.',
  'err.name_taken': 'Ya hay una con ese nombre.',
  'err.collection_not_found': 'Esa colección ya no existe.',
  'err.tag_not_found': 'Esa etiqueta ya no existe.',
  'err.invalid_color': 'Ese color no es válido.',
  'backup.menu': 'Copia de seguridad',
  'backup.with_media': 'Incluir fotos, vídeos y documentos',
  'backup.export': 'Exportar bóveda…',
  'backup.import': 'Importar copia…',
  'backup.exporting': 'Exportando…',
  'backup.importing': 'Importando…',
  'backup.exported': 'Copia creada: {posts} posts, {files} archivos.',
  'backup.imported': 'Importación: {added} nuevos, {merged} fusionados.',
  'backup.bad_hash': '{n} archivos no coincidían con el original y no se guardaron (Reintentar los baja).',
  'backup.errors': '{n} posts no se pudieron importar. Primero: {first}',
  'err.export_failed': 'No se pudo crear la copia.',
  'err.import_invalid': 'Ese archivo no es una copia de Mirlo.',
  'err.import_unsafe_path': 'La copia tiene rutas peligrosas: no se ha importado nada.',
  'err.import_too_big': 'La copia es demasiado grande para importarla.',
  'err.not_in_export': 'No venía en la copia (exportada sin medios).',
  'err.hash_mismatch': 'El archivo de la copia no coincide con el original.',
  'err.internal': 'Error interno de Mirlo.',
  'x.menu': 'Integrar con X',
  'x.offer': '¿Integrar Mirlo con X?',
  'x.offer_why': 'Guarda posts desde el menú Compartir, sin cambiar de aplicación.',
  'x.integrate': 'Integrar',
  'x.not_now': 'Ahora no',
  'x.title': 'Integrar Mirlo con X',
  'x.close': 'Cerrar',
  'x.links_checking': 'Comprobando los enlaces de Mirlo…',
  'x.links_here': '✓ Los enlaces de Mirlo se abren en esta app',
  'x.links_other': '◐ Los enlaces de Mirlo abren otra copia o aplicación',
  'x.links_nobody': '◐ Los enlaces de Mirlo todavía no abren esta app',
  'x.links_unknown': '○ No se puede saber quién abre los enlaces de Mirlo: lo dirá la comprobación final',
  'x.register': 'Permitir enlaces de Mirlo',
  'x.register_note': 'Se creará un acceso en tu carpeta de usuario. No pide contraseña.',
  'x.register_note_other': 'Se creará un acceso en tu carpeta de usuario y los enlaces dejarán de abrir la otra copia. No pide contraseña.',
  'x.use_installer': 'Para que abran esta copia, instala Mirlo con el instalador de tu sistema.',
  'x.browser': '¿En qué navegador usas X?',
  'x.manager': 'Instala Violentmonkey o Tampermonkey, las extensiones que ejecutan scripts como el de Mirlo. Cada botón abre su web oficial, con el enlace para tu navegador.',
  'x.manager_note': 'Algunos navegadores restringen Violentmonkey: si en el tuyo no se puede instalar o aparece desactivado, usa Tampermonkey.',
  'x.allow_scripts': 'En Extensiones, abre los detalles de Tampermonkey (o Violentmonkey) y activa «Permitir secuencias de comandos del usuario».',
  'x.allow_scripts_note': 'Sin ese permiso el script se instala, pero no funciona. En versiones antiguas se llama «Modo de desarrollador».',
  'x.install_step': 'Instala el script de Mirlo y confírmalo en tu navegador. Si ya tenías uno anterior, se actualiza.',
  'x.install': 'Instalar script',
  'x.copy': 'Copiar script',
  'x.copied': '✓ Copiado: pégalo en un script nuevo del gestor y guárdalo.',
  'x.install_url': 'Si se abrió otro navegador, pega esta dirección en el tuyo. Vale unos minutos:',
  'x.check_step': 'Comprueba que funciona: se abre X y allí pulsas «Conectar con Mirlo». Si ya tenías X abierto, se usa una pestaña nueva.',
  'x.check': 'Comprobar',
  'x.check_url': 'Si se abrió otro navegador, pega esta dirección en el tuyo. Si no ves el botón, revisa los pasos anteriores.',
  'x.unchecked': '○ Sin comprobar',
  'phone.menu': 'Conectar móvil',
  'phone.title': 'Conectar el móvil',
  'phone.intro': 'Para guardar desde la app de X en Android con «Send with Mirlo». Mirlo prepara Syncthing en este ordenador; en el móvil necesitas Syncthing-Fork.',
  'phone.step_scan': 'En Syncthing-Fork: pestaña Dispositivos › + › icono de QR, y escanea este código. En la misma WiFi también vale tocar este ordenador en «Dispositivos descubiertos».',
  'phone.qr_alt': 'Código QR con el ID de Syncthing de este ordenador',
  'phone.step_accept': 'Cuando el móvil aparezca aquí, acéptalo.',
  'phone.waiting': '○ Esperando a que el móvil intente conectar…',
  'phone.accept': 'Aceptar',
  'phone.step_folder': 'En el móvil, acepta la carpeta «Mirlo» en modo «Enviar y recibir» y elígela en la app Send with Mirlo.',
  'phone.shared': '{name} recibe la bandeja de Mirlo',
  'err.syncthing_missing': 'No se encuentra Syncthing en este ordenador. Instálalo desde syncthing.net y ábrelo una vez.',
  'err.syncthing_off': 'Syncthing no responde. ¿Está abierto?',
  'err.syncthing_refused': 'Syncthing rechazó la petición.',
  'x.waiting': '○ Esperando la comprobación',
  'x.connected': '✓ Conectado con X',
  'err.register_failed': 'No se pudieron registrar los enlaces de Mirlo.',
  'err.delivery_failed': 'No se pudo preparar la instalación. Usa «Copiar script».',
  'err.copy_failed': 'No se pudo copiar el script.',
}

type Key = keyof typeof es

const en: Record<Key, string> = {
  'cmd.placeholder': 'Paste an X link to save it, or type to search',
  'cmd.label': 'Link or search',
  'cmd.save': 'Save post',
  'cmd.theme_light': 'Light theme',
  'cmd.theme_dark': 'Dark theme',
  'cmd.other_lang': 'Español',

  'side.label': 'Filters',
  'side.all': 'All',
  'side.partial': 'Partial',
  'side.error': 'With errors',
  'side.by_type': 'By type',
  'side.authors': 'Authors',
  'side.date': 'Post date: {range}',
  'side.date_any': 'any',
  'side.date_range': '{from} to {to}',
  'side.from': 'From',
  'side.to': 'To',
  'side.clear_dates': 'Clear dates',
  'side.favorites': 'Favorites',
  'side.collections': 'Collections',
  'side.new_collection': 'New collection',
  'side.tags': 'Tags',
  'side.no_tags': 'Add tags from the open post with [T].',
  'side.edit_hint': 'Double-click or [F2]: rename. [Alt] [↑] [↓]: move.',
  'side.edit_hint_tag': 'Double-click or [F2]: rename, color or delete.',

  'edit.name': 'Name',
  'edit.delete': 'Delete',
  'edit.confirm_delete': 'Yes, delete',
  'edit.delete_collection_hint': 'Posts are not deleted: they just leave the collection.',
  'edit.delete_tag_hint': 'It is removed from all its posts.',
  'edit.cancel': 'Cancel',
  'edit.done': 'Done',
  'edit.color': 'Color',
  'edit.no_color': 'No color',
  'edit.custom_color': 'Another color…',
  'edit.brightness': 'Brightness',
  'edit.hex': 'Color as HEX',
  'edit.use_color': 'Use',
  'color.red': 'Red',
  'color.orange': 'Orange',
  'color.yellow': 'Yellow',
  'color.green': 'Green',
  'color.teal': 'Teal',
  'color.blue': 'Blue',
  'color.purple': 'Purple',
  'color.pink': 'Pink',
  'org.favorite': 'Favorite',
  'org.add_favorite': 'Add to favorites',
  'org.remove_favorite': 'Remove from favorites',
  'org.collection': 'Collection',
  'org.no_collection': 'No collection',
  'org.tags': 'Tags',
  'org.tag_input': 'Add tag',
  'org.tag_placeholder': 'Type and press Enter',
  'org.remove_tag': 'Remove {name}',
  'org.note': 'Note',
  'org.note_placeholder': 'Just for you. It is searchable too.',

  'kind.video': 'With video',
  'kind.photo': 'With photos',
  'kind.gif': 'With GIFs',
  'kind.document': 'With documents',
  'kind.text': 'Text only',

  'list.label': 'Posts',
  'list.items_label': 'Saved posts',
  'list.newest': 'Newest first',
  'list.searching': 'Searching “{q}”',
  'list.saving_user': 'Saving @{user}’s post…',
  'list.saving': 'Saving post…',
  'list.no_match': 'Nothing matches these filters.',
  'list.empty_collection': 'This collection is empty. Open a post and press [M] to pick its collection.',
  'list.empty': 'Your vault is empty. Copy the link of an X post and press [Ctrl] [V] anywhere in this window.',
  'list.post_id': 'Post {id}',
  'list.no_date': 'No date',

  'media.photo': 'Photos',
  'media.video': 'Video',
  'media.gif': 'GIF',
  'media.document': 'Document',
  'media.text': 'Text',

  'status.ok': '✓ Complete',
  'status.partial': '◐ Partial',
  'status.error': '✕ Error',

  'read.label': 'Open post',
  'read.retry': 'Retry',
  'read.open_folder': 'Open folder',
  'read.view_on_x': 'View on X',
  'read.quoting': 'Quoting @{user}',
  'read.files': 'Files on disk',
  'read.file': 'File',
  'read.state': 'Status',
  'read.size': 'Size',
  'read.downloaded': '✓ Downloaded',
  'read.skipped': 'Skipped: {why}',
  'read.source': 'Source: {source}',
  'read.source_none': 'none',
  'read.details': 'Details: {detail}',
  'read.pick': 'Pick a post from the list with [J] and [K], or with the mouse.',
  'read.how_to_save': 'To save a new one, copy its link on X and press [Ctrl] [V] here.',

  'keys.label': 'Keyboard shortcuts',
  'keys.nav': 'next and previous',
  'keys.search': 'search',
  'keys.save': 'save link',
  'keys.open': 'open folder',
  'keys.retry': 'retry',
  'keys.favorite': 'favorite',
  'keys.tags': 'tags',
  'keys.collection': 'collection',
  'keys.note': 'note',

  'msg.saved': 'Saved: @{user}',
  'msg.saved_partial': 'Saved with warnings: @{user}',
  'msg.save_failed': 'Could not save: {error}',
  'msg.retrying': 'Retrying…',
  'msg.retry_ok': 'Retry worked: it is complete now',
  'msg.retry_partial': 'Retried, but it is still incomplete',
  'msg.retry_failed': 'Still failing: {error}',

  'gallery.loading_video': 'Loading video…',
  'gallery.view_full': 'View full screen: {alt}',
  'gallery.photo_alt': 'Photo {n} of {total}, no description from the author',
  'gallery.hint.one': '1 photo at original resolution. Click it to view it full screen.',
  'gallery.hint.other': '{n} photos at original resolution. Click one to view it full screen.',
  'gallery.viewer': 'Photo viewer',
  'gallery.counter': '{n} of {total}',
  'gallery.prev': 'Previous',
  'gallery.next': 'Next',
  'gallery.close': 'Close',

  'err.invalid_link': 'That is not a valid link.',
  'err.link_scheme': 'The link must start with https://',
  'err.host_not_allowed': 'Only x.com and twitter.com links are accepted.',
  'err.not_a_post': 'The link does not point to a post (missing /status/).',
  'err.invalid_post_id': 'The post number is not valid.',
  'err.offline': 'No connection: could not reach X.',
  'err.post_unavailable': 'Could not read the post. It may have been deleted or be private.',
  'err.network': 'The connection dropped.',
  'err.http': 'The server answered with an error.',
  'err.too_big': 'Larger than {detail} MB',
  'err.media_host_not_allowed': 'The file is not on X’s servers.',
  'err.video_unavailable': 'Could not download the video, directly or with yt-dlp.',
  'err.disk_write': 'Could not write to disk.',
  'err.db_locked': 'The database got locked. Restart Mirlo.',
  'err.database': 'Database error.',
  'err.not_found': 'That post is not in the vault.',
  'err.no_folder': 'This post has no folder yet.',
  'err.folder_missing': 'The folder no longer exists.',
  'err.unknown_file': 'That file does not belong to the post.',
  'err.outside_vault': 'That path is outside the vault.',
  'err.read_file': 'Could not read the file.',
  'err.open_failed': 'Could not open it.',
  'err.name_empty': 'Type a name.',
  'err.name_taken': 'That name is already taken.',
  'err.collection_not_found': 'That collection no longer exists.',
  'err.tag_not_found': 'That tag no longer exists.',
  'err.invalid_color': 'That color is not valid.',
  'backup.menu': 'Backup',
  'backup.with_media': 'Include photos, videos and documents',
  'backup.export': 'Export vault…',
  'backup.import': 'Import backup…',
  'backup.exporting': 'Exporting…',
  'backup.importing': 'Importing…',
  'backup.exported': 'Backup created: {posts} posts, {files} files.',
  'backup.imported': 'Import: {added} new, {merged} merged.',
  'backup.bad_hash': '{n} files did not match the original and were not kept (Retry downloads them).',
  'backup.errors': '{n} posts could not be imported. First: {first}',
  'err.export_failed': 'Could not create the backup.',
  'err.import_invalid': 'That file is not a Mirlo backup.',
  'err.import_unsafe_path': 'The backup has unsafe paths: nothing was imported.',
  'err.import_too_big': 'The backup is too big to import.',
  'err.not_in_export': 'Not in the backup (exported without media).',
  'err.hash_mismatch': 'The file in the backup does not match the original.',
  'err.internal': 'Mirlo internal error.',
  'x.menu': 'Integrate with X',
  'x.offer': 'Integrate Mirlo with X?',
  'x.offer_why': 'Save posts from the Share menu, without switching apps.',
  'x.integrate': 'Integrate',
  'x.not_now': 'Not now',
  'x.title': 'Integrate Mirlo with X',
  'x.close': 'Close',
  'x.links_checking': 'Checking Mirlo links…',
  'x.links_here': '✓ Mirlo links open in this app',
  'x.links_other': '◐ Mirlo links open another copy or app',
  'x.links_nobody': '◐ Mirlo links do not open this app yet',
  'x.links_unknown': '○ Cannot tell what opens Mirlo links: the final check will show it',
  'x.register': 'Allow Mirlo links',
  'x.register_note': 'A launcher will be created in your user folder. No password needed.',
  'x.register_note_other': 'A launcher will be created in your user folder and links will stop opening the other copy. No password needed.',
  'x.use_installer': 'To make them open this copy, install Mirlo with the installer for your system.',
  'x.browser': 'Which browser do you use X in?',
  'x.manager': 'Install Violentmonkey or Tampermonkey, the extensions that run scripts like Mirlo’s. Each button opens its official website, with the link for your browser.',
  'x.manager_note': 'Some browsers restrict Violentmonkey: if yours will not install it or shows it disabled, use Tampermonkey.',
  'x.allow_scripts': 'In Extensions, open the details of Tampermonkey (or Violentmonkey) and turn on “Allow User Scripts”.',
  'x.allow_scripts_note': 'Without it the script installs but does not work. Older versions call it “Developer mode”.',
  'x.install_step': 'Install the Mirlo script and confirm it in your browser. An earlier one gets updated.',
  'x.install': 'Install script',
  'x.copy': 'Copy script',
  'x.copied': '✓ Copied: paste it into a new script in the manager and save it.',
  'x.install_url': 'If another browser opened, paste this address into yours. It works for a few minutes:',
  'x.check_step': 'Check that it works: X opens and you click “Connect to Mirlo” there. If X was already open, a new tab is used.',
  'x.check': 'Check',
  'x.check_url': 'If another browser opened, paste this address into yours. If you do not see the button, review the steps above.',
  'phone.menu': 'Connect phone',
  'phone.title': 'Connect your phone',
  'phone.intro': 'To save from the X app on Android with "Send with Mirlo". Mirlo sets up Syncthing on this computer; on the phone you need Syncthing-Fork.',
  'phone.step_scan': 'In Syncthing-Fork: Devices tab › + › QR icon, and scan this code. On the same WiFi you can also tap this computer under "Discovered devices".',
  'phone.qr_alt': 'QR code with the Syncthing ID of this computer',
  'phone.step_accept': 'When the phone shows up here, accept it.',
  'phone.waiting': '○ Waiting for the phone to try to connect…',
  'phone.accept': 'Accept',
  'phone.step_folder': 'On the phone, accept the "Mirlo" folder as "Send & Receive" and choose it in the Send with Mirlo app.',
  'phone.shared': '{name} receives the Mirlo inbox',
  'err.syncthing_missing': 'Syncthing is not installed on this computer. Get it from syncthing.net and open it once.',
  'err.syncthing_off': 'Syncthing does not answer. Is it running?',
  'err.syncthing_refused': 'Syncthing refused the request.',
  'x.unchecked': '○ Not checked',
  'x.waiting': '○ Waiting for the check',
  'x.connected': '✓ Connected to X',
  'err.register_failed': 'Could not register Mirlo links.',
  'err.delivery_failed': 'Could not prepare the installation. Use “Copy script”.',
  'err.copy_failed': 'Could not copy the script.',
}

export type Lang = 'es' | 'en'
const dictionaries: Record<Lang, Record<Key, string>> = { es, en }

function initialLang(): Lang {
  try {
    const saved = localStorage.getItem('lang')
    if (saved === 'es' || saved === 'en') return saved
  } catch {}
  // primera vez: el idioma del sistema; todo lo que no es español va en inglés
  return navigator.language.toLowerCase().startsWith('es') ? 'es' : 'en'
}

export const i18n = $state<{ lang: Lang }>({ lang: initialLang() })

export function setLang(lang: Lang) {
  i18n.lang = lang
  document.documentElement.lang = lang
  try {
    localStorage.setItem('lang', lang)
  } catch {}
}
document.documentElement.lang = i18n.lang

type Params = Record<string, string | number>

export function t(key: Key, params: Params = {}): string {
  return dictionaries[i18n.lang][key].replace(/\{(\w+)\}/g, (m, name) => (name in params ? String(params[name]) : m))
}

/** Texto con plural: usa `<key>.one` / `<key>.other` según las reglas del idioma. */
export function tn(key: 'gallery.hint', n: number): string {
  const form = new Intl.PluralRules(i18n.lang).select(n) === 'one' ? 'one' : 'other'
  return t(`${key}.${form}`, { n })
}

/** Trozos de un texto con teclas marcadas como [Ctrl], para pintarlas con <kbd> sin usar HTML. */
export function withKeys(text: string): { text: string; key: boolean }[] {
  return text.split(/(\[[^\]]+\])/).filter(Boolean).map((part) =>
    part.startsWith('[') ? { text: part.slice(1, -1), key: true } : { text: part, key: false },
  )
}

/**
 * Error de Rust (`codigo` o `codigo: detalle`) → frase en el idioma actual y detalle técnico aparte.
 * Si la frase lleva {detail} (too_big: "Supera 50 MB"), el detalle va dentro y no se repite.
 * Un texto antiguo sin código conocido se devuelve tal cual.
 */
export function errorParts(raw: unknown): { text: string; detail: string } {
  const s = String(raw)
  const sep = s.indexOf(': ')
  const code = sep === -1 ? s : s.slice(0, sep)
  const detail = sep === -1 ? '' : s.slice(sep + 2)
  const key = `err.${code}` as Key
  if (!(key in es)) return { text: s, detail: '' }
  const template = dictionaries[i18n.lang][key]
  return template.includes('{detail}') ? { text: t(key, { detail }), detail: '' } : { text: template, detail }
}

/** Frase y detalle en una línea, para sitios con poco espacio (tabla de archivos). */
export function errorText(raw: unknown): string {
  const { text, detail } = errorParts(raw)
  return detail ? `${text} (${detail})` : text
}

export function formatDate(iso: string | null, long = false): string {
  if (!iso) return t('list.no_date')
  const opts: Intl.DateTimeFormatOptions = long ? { dateStyle: 'long', timeStyle: 'short' } : { day: 'numeric', month: 'short', year: 'numeric' }
  return new Intl.DateTimeFormat(i18n.lang, opts).format(new Date(iso))
}

export function formatSize(bytes: number): string {
  if (bytes < 1048576) return `${Math.ceil(bytes / 1024)} KB`
  return `${new Intl.NumberFormat(i18n.lang, { maximumFractionDigits: 1 }).format(bytes / 1048576)} MB`
}
