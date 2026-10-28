# Fase 5: empaquetado, CI y backport

> Informe de task para alguien que sabe algo de programación (por ejemplo Python) pero no domina Rust ni Tauri.

## 1. Resumen para situarse

- **Qué se ha construido:** paquetes instalables (.deb, .rpm, AppImage) con yt-dlp dentro, workflows de CI y release, README y el plan (local) para llevar las mejoras a ThreadsVault.
- **Dónde está:** `scripts/fetch-yt-dlp.mjs`, `src-tauri/tauri.conf.json` (`externalBin`), `.github/workflows/`, `README.md`.
- **Estado final:** 32 tests y 6 pruebas en vivo en verde (incluida la del respaldo de vídeo con yt-dlp, pendiente desde la fase 2), clippy, svelte-check y actionlint sin avisos, `npm run tauri build` genera los 3 paquetes.

## 2. Flujo del empaquetado

```
npm run tauri build
  → beforeBuildCommand: npm run sidecar (baja yt-dlp, verifica SHA-256) && npm run build (frontend)
  → cargo build --release
  → bundle: /usr/bin/xvault + /usr/bin/xvault-yt-dlp + XVault.desktop
```

## 3. Qué NO está verificado

- Instalar el .deb en el sistema (no se ha instalado: se ha comprobado su contenido extrayéndolo).
- Los workflows en GitHub: el repo no tiene remoto todavía. actionlint los da por buenos, pero no han corrido.
- Los paquetes de Windows y macOS: solo se compilarán en `release.yml`.
- Los iconos siguen siendo los copiados de ThreadsVault.
