<p align="center">
  <img src="assets/banner.png" width="900" alt="Mirlo: tu bóveda local de posts de X. Icono de un cajón de fichero azul con un post en modo oscuro y un mirlo posado encima.">
</p>

<p align="center"><b>Español</b> · <a href="README-EN.md">English</a></p>

Bóveda local para posts de X (Twitter). Pegas el enlace de un post y Mirlo guarda en tu disco el texto, el autor, la fecha y todos sus archivos: fotos a resolución original, vídeos, GIF (como MP4) y documentos enlazados (PDF, DOCX…).

También puedes [enviarlo desde el menú Compartir de X](#enviar-desde-x), sin cambiar de ventana.

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
| <kbd>F</kbd> | Marcar o quitar favorito |
| <kbd>T</kbd> / <kbd>M</kbd> / <kbd>N</kbd> | Etiquetas, colección y nota del post |

## Enviar desde X

Guarda un post con **Compartir → Enviar a Mirlo**, esté Mirlo abierto, oculto o cerrado.

<p align="center">
  <img src="assets/share-menu-es.png" width="248" alt="Menú Compartir de X con tres opciones: Enviar a Mirlo, Enviar vía Chat y Copiar vínculo.">
</p>

Con X en español el botón dice «Enviar a Mirlo»; en cualquier otro idioma, «Send to Mirlo».

En las versiones posteriores a la 0.2.0, Mirlo te guía: lo ofrece la primera vez y lo tienes siempre en **Integrar con X**, en la barra superior.

1. Elige tu navegador e instala el gestor de userscripts que te indica (una extensión que añade pequeños programas a las webs): Violentmonkey en Firefox o Tampermonkey en Chrome, Edge, Brave y Chromium.
2. Solo en los navegadores Chromium: activa «Permitir secuencias de comandos del usuario» en los detalles de la extensión.
3. Pulsa **Instalar script** y confírmalo en el navegador. El script va dentro de Mirlo; también puedes copiarlo.
4. Pulsa **Comprobar** y, en X, **Conectar con Mirlo**. Cuando Mirlo recibe el saludo muestra «✓ Conectado con X».

Con la AppImage, el asistente te pide antes permiso para registrar los enlaces `mirlo://` con el botón **Permitir enlaces de Mirlo**. Si la mueves de carpeta, vuelve a pulsarlo.

<details>
<summary>Instalarlo a mano (o con la 0.2.0, que no trae asistente)</summary>

1. Instala Mirlo con un [instalador](https://github.com/D4vRAM369/Mirlo-desktop/releases/latest): la AppImage de la 0.2.0 no registra los enlaces `mirlo://`.
2. Instala [Violentmonkey](https://violentmonkey.github.io/) en Firefox o [Tampermonkey](https://www.tampermonkey.net/) en Chromium, con «Permitir secuencias de comandos del usuario» activado.
3. Abre **[mirlo-x.user.js](https://github.com/D4vRAM369/Mirlo-desktop/raw/master/userscript/mirlo-x.user.js)** y confirma la instalación.
4. Recarga X y usa **Compartir → Enviar a Mirlo**.

</details>

La primera vez el navegador pregunta si puede abrir Mirlo. Si un envío sale bien, Mirlo guarda sin quitarte el foco. Si falla, muestra su ventana con el error.

## Organizar

- **Favoritos** (★) para lo que quieres tener a mano.
- **Colecciones**, como carpetas: cada post va en una o en ninguna. Se crean con **+** en la barra lateral, se renombran o borran con doble clic (o <kbd>F2</kbd>) y se reordenan con <kbd>Alt</kbd>+<kbd>↑</kbd>/<kbd>↓</kbd>.
- **Etiquetas**, todas las que quieras por post, con un color de la paleta o cualquier otro desde la rueda de color.
- **Notas** personales en cada post.

Las notas y las etiquetas entran en la búsqueda. Volver a guardar o reintentar un post nunca borra lo que añadiste: va en la BD y en el bloque `user` de su `post.json`.

## Copias de seguridad

En **Copia de seguridad** (barra superior):

- **Exportar bóveda…** crea un `.zip` con todo. Desmarca *Incluir fotos, vídeos y documentos* para una copia ligera con solo el texto.
- **Importar copia…** lo trae de vuelta, en este equipo o en otro. **Fusiona, nunca sobrescribe:** los posts que ya tienes no cambian, se suman etiquetas y favoritos, y tus notas y colecciones se quedan. Cada archivo se comprueba con su huella SHA-256, y una copia con rutas sospechosas se rechaza entera.

Si la copia era ligera, los posts importados quedan como parciales: <kbd>R</kbd> baja lo que falta.

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

Las carpetas son la copia de verdad: puedes copiarlas o leerlas sin Mirlo.

## Qué sale de tu equipo

Mirlo solo hace estas peticiones, y solo cuando guardas o reintentas un post:

- **X** (`cdn.syndication.twimg.com`, `publish.x.com`, `pbs.twimg.com`, `video.twimg.com`): el post y sus archivos.
- **yt-dlp** (incluido en la app): pide el post a X cuando la vía anterior falla.
- **FxTwitter** (`api.fxtwitter.com`): **solo** para posts de más de 280 caracteres, porque X los entrega cortados. Recibe el número del post.
- **Los sitios enlazados en el post**: Mirlo mira si el enlace es un documento (por su `Content-Type`) y, si lo es y pesa menos de 50 MB, lo descarga. Nunca toca direcciones de tu red local.

El userscript no hace peticiones ni lee cookies: solo pasa a Mirlo el número del post o, al comprobar la conexión, el código de un solo uso que creó Mirlo. Cualquier web podría intentar abrir un enlace `mirlo://`, por eso el navegador pide permiso y Mirlo solo acepta esas dos formas, y el saludo solo con su código. Si marcas «permitir siempre», hazlo solo para x.com.

El asistente solo actúa cuando pulsas sus botones: abre en tu navegador la tienda de extensiones o X, y para **Instalar script** escucha hasta 10 minutos en una dirección de tu propio equipo (`127.0.0.1`) que únicamente entrega ese script.

La tipografía va dentro de la app: abrir Mirlo no llama a nadie.

## Limitaciones

- No guarda hilos completos ni posts protegidos o que requieren iniciar sesión.
- «Enviar a Mirlo» depende de la página de X y puede dejar de salir cuando X la cambie. Se ha probado en Linux con X sin iniciar sesión: de principio a fin en Firefox con Violentmonkey; en Chromium, la pantalla de instalación de Tampermonkey y la comprobación cargando el script a mano. Faltan por probar la sesión iniciada, Tampermonkey con su permiso activado, Windows y macOS.
- Con la AppImage en una ruta con espacios, algunos escritorios sin GNOME ni KDE no abren los enlaces `mirlo://`: muévela a una ruta sin espacios.
- La comprobación necesita Mirlo abierto: si lo cerraste, pulsa **Comprobar** otra vez.
- Si FxTwitter no responde, un post largo queda como parcial (texto cortado por X).

## Desarrollo

Requisitos: Rust, Node 20+ y las [dependencias de Tauri 2](https://tauri.app/start/prerequisites/) del sistema.

```bash
npm install
npm run sidecar        # descarga yt-dlp para tu plataforma y comprueba su SHA-256
npm run tauri dev      # sal antes del Mirlo instalado: solo puede haber una instancia
```

```bash
npm run check                                   # tipos, traducciones y userscript
cd src-tauri && cargo test                      # tests con respuestas reales guardadas
cd src-tauri && cargo test -- --ignored         # pruebas en vivo contra X
cd src-tauri && cargo clippy --all-targets -- -D warnings
npm run tauri build                             # paquetes .deb, .rpm y AppImage
```

También he publicado la doc pública de la skill [ProfesorIA](https://github.com/D4vRAM369/ProfesorIA). Espero que resulte productiva y didáctica.

Cambios por versión: [`CHANGELOG.md`](CHANGELOG.md).

## Licencia

[GPL-3.0 o posterior](LICENSE). Puedes usar, estudiar, cambiar y compartir Mirlo; si distribuyes una versión modificada, su código tiene que ser también libre con esta misma licencia.
