# Fase 4: diseño final Bandeja

> Informe de task para alguien que sabe algo de programación (por ejemplo Python) pero no domina Rust ni Tauri.

## 1. Resumen para situarse

- **Qué se ha construido:** la interfaz final según el mockup B "Bandeja": tres paneles, tema claro y oscuro, galería con visor, vídeo inline, cola de guardado y atajos a la vista.
- **Dónde está:** `src/app.css`, `src/App.svelte`, `src/lib/Gallery.svelte` (nuevo); `db.rs` y `vault.rs` para el texto alternativo.
- **Estado final:** 32 tests en verde, clippy y svelte-check sin avisos, revisado con capturas en claro, oscuro, visor, error y ancho mínimo.

## 2. Decisiones clave

Decisión 9 en `02_ARQUITECTURA.md`: CSS propio, fuente local, alt text con migración.

## 3. Qué NO está verificado

- Las capturas se hicieron en Chromium con el puente de Tauri simulado. En la ventana real (WebKitGTK) hay que mirar el visor y el tema oscuro.
- Lectores de pantalla: la estructura es accesible (roles, etiquetas, alt), pero no se ha probado con Orca.

## 4. Siguiente

Fase 5: empaquetado con yt-dlp como sidecar y `docs/BACKPORT-THREADSVAULT.md`.
