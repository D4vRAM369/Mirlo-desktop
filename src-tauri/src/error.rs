//! Errores como códigos: Rust dice QUÉ pasó y el frontend lo cuenta en el idioma de quien usa la app
//! (`src/lib/i18n.ts`). Formato: `codigo` o `codigo: detalle`. El detalle es técnico (estado HTTP,
//! rutas, errores del sistema) y no se traduce. `npm run check` falla si un código no tiene traducción.
//! Los mismos textos se guardan en la BD y en post.json, así que un error guardado en español se lee
//! en inglés si cambias de idioma.

pub fn fail(code: &str) -> String {
    code.to_string()
}

pub fn fail_with(code: &str, detail: impl std::fmt::Display) -> String {
    format!("{code}: {detail}")
}
