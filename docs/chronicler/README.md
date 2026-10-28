# XVault

> Bóveda local para posts de X (Twitter): pegas la URL de un post y la app guarda en tu disco el texto, el autor, la fecha, las fotos, los vídeos y los documentos enlazados.

**Stack:** Tauri 2 (Rust) + Svelte 5 + Tailwind 4 + SQLite
**Nivel de dificultad:** Intermedio
**Documentado con:** metodología PBL

---

## ¿Qué es esto?

Una app de escritorio que funciona sin conexión a ninguna nube. Copias el enlace de un post de X, lo pegas y la app descarga todo su contenido a una carpeta de tu ordenador. Si el post se borra mañana, tú lo sigues teniendo.

## ¿Por qué existe?

Es la versión para X de **ThreadsVault-desktop**. ThreadsVault a veces guardaba texto que no era del post (respuestas, textos de la interfaz) porque lo sacaba del HTML de la página con expresiones regulares. XVault nace con una regla: el contenido solo se lee de datos estructurados (JSON). Lo que se aprenda aquí se llevará después a ThreadsVault.

## ¿Qué vas a aprender si sigues esta documentación?

- [ ] Por qué el *scraping* de HTML con regex es frágil y qué hacer en su lugar
- [ ] Cómo funciona el endpoint de *syndication* que usan los embeds de X
- [ ] Escribir un parser como función pura y probarlo con *fixtures*
- [ ] Descargas seguras: escritura atómica, hash SHA-256 y reintentos
- [ ] Búsqueda de texto completo con SQLite FTS5
- [ ] Cómo se comunican Svelte (frontend) y Rust (backend) en Tauri

## Cómo seguir esta documentación

1. `00_GLOSARIO.md` · 2. `01_SETUP.md` · 3. `02_ARQUITECTURA.md` · 4. `03_PASO_A_PASO.md` · 5. `04_CONCEPTOS.md` · 6. `05_ERRORES_Y_SOLUCIONES.md` · 7. `06_SIGUIENTE_NIVEL.md` · 8. `07_COMANDOS_Y_MANUAL.md` · 9. `informes/`

> Hay una segunda carpeta, `docs/profesoria/`, con la documentación que genera ProfesorIA al cerrar cada sesión. Esta (`docs/chronicler/`) se escribe en tiempo real.

---

*Documentación generada con metodología PBL — "Aprendo mientras creo, creo mientras aprendo."*
