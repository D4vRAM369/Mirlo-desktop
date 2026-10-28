# ROADMAP: Mirlo

> Qué tiene Mirlo y qué viene después, por versiones. Los cambios publicados están en [`CHANGELOG.md`](../CHANGELOG.md).

## ✅ Hecho (sin etiqueta todavía)
- Fases 1–5: extractor con TDD, descargas atómicas con SHA-256, SQLite + FTS5, diseño Bandeja, empaquetado con yt-dlp, CI y workflow de release.
- Icono propio, nombre Mirlo, banner y README en español e inglés.
- Interfaz en español e inglés con errores de Rust como códigos.
- `.deb` probado (extraído, sin instalar): abre, icono 256 px, sidecar correcto.

## 🔨 v0.1.0: primera versión pública (en curso)
1. ✅ **Organizar:** favoritos, colecciones, etiquetas (paleta de 8 tonos o color HEX propio) y notas, con migración `user_version = 2` y bloque `user` en `post.json`.
2. ✅ **Exportar / importar:** `.zip` con manifiesto, importación que fusiona y nunca sobrescribe, protección zip-slip.
3. **Release:** CHANGELOG, README, etiqueta `v0.1.0`, revisar el borrador y publicar.

## 🔜 v0.2.0
- **Reconstruir el índice desde disco** (recorrer `~/Mirlo/*/*/post.json`): recuperación si se pierde `mirlo.db`.
- **Ruta de la bóveda configurable** (hoy fija en `~/Mirlo`) y límite de documentos configurable (hoy 50 MB).
- **Paginación** de la lista (hoy tope de 500 filas, comentario `ponytail:` en `db::list`).
- **Probar Windows y macOS** de verdad y valorar la firma de los instaladores.
- **Icono simplificado a 16 px** (a ese tamaño hoy solo se lee "cajón azul").
- Arrastrar una fila de la lista a una colección del sidebar.
- Enlaces del texto del post clicables (abrir en el navegador del sistema con validación).

## 💡 Más adelante (sin compromiso)
- Hilos completos.
- Posts protegidos con `--cookies-from-browser` de yt-dlp (opcional, desactivado por defecto).
- Exportar una colección a HTML estático para compartirla.
- Servir vídeos grandes por rangos en lugar de cargarlos enteros en memoria (`read_media`).
- Cerrar la ventana de *DNS rebinding* en descargas de documentos (comentario `ponytail:` en `vault::is_public_url`).

## Fuera del proyecto
- **Llevar el extractor a ThreadsVault** (proyecto hermano): se hará en su propio repositorio.
