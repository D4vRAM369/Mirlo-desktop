// Comprueba que cada código de error que puede devolver Rust tiene su traducción en src/lib/i18n.svelte.ts.
// (Que es y en tienen las mismas claves ya lo garantiza TypeScript.) Se ejecuta dentro de `npm run check`.
import { readFileSync, readdirSync } from 'node:fs'

const rust = readdirSync('src-tauri/src')
  .filter((f) => f.endsWith('.rs'))
  .map((f) => readFileSync(`src-tauri/src/${f}`, 'utf8'))
  .join('\n')
const codes = new Set([...rust.matchAll(/fail(?:_with)?\("([a-z_]+)"/g)].map((m) => m[1]))
// códigos que se construyen con una condición: fail_with(if offline { "offline" } else { "post_unavailable" }, …)
for (const m of rust.matchAll(/fail(?:_with)?\(if [^{]+\{ "([a-z_]+)" \} else \{ "([a-z_]+)" \}/g)) {
  codes.add(m[1])
  codes.add(m[2])
}

const dict = readFileSync('src/lib/i18n.svelte.ts', 'utf8')
const missing = [...codes].filter((c) => !dict.includes(`'err.${c}':`))
if (missing.length) {
  console.error(`Códigos de error sin traducir en src/lib/i18n.svelte.ts: ${missing.join(', ')}`)
  process.exit(1)
}
console.log(`i18n: ${codes.size} códigos de error de Rust, todos traducidos`)
