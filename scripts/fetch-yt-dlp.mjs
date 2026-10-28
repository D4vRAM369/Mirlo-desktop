// Descarga yt-dlp para empaquetarlo como sidecar de Tauri (bundle.externalBin) y comprueba su SHA-256.
// Tauri exige el nombre `mirlo-yt-dlp-<target-triple>` y al empaquetar lo deja junto al ejecutable como
// `mirlo-yt-dlp`, que es donde lo busca `sources::ytdlp_binary()`. El prefijo evita chocar con el paquete
// `yt-dlp` del sistema, que ya es dueño de /usr/bin/yt-dlp (el .deb no se podría instalar).
// Uso: `npm run sidecar`. Si el archivo ya está y su hash cuadra, no descarga nada.
import { createHash } from 'node:crypto'
import { execFileSync } from 'node:child_process'
import { chmodSync, existsSync, mkdirSync, readFileSync, renameSync, writeFileSync } from 'node:fs'

// Sube la versión aquí cuando X cambie algo y yt-dlp publique el arreglo.
const VERSION = '2026.10.26'

const triple = execFileSync('rustc', ['-vV'], { encoding: 'utf8' }).match(/^host: (.+)$/m)[1]
const windows = triple.includes('windows')
const asset = windows
  ? 'yt-dlp.exe'
  : triple.includes('apple')
    ? 'yt-dlp_macos'
    : triple.startsWith('aarch64')
      ? 'yt-dlp_linux_aarch64'
      : 'yt-dlp_linux'
const dest = `src-tauri/binaries/mirlo-yt-dlp-${triple}${windows ? '.exe' : ''}`
const base = `https://github.com/yt-dlp/yt-dlp/releases/download/${VERSION}/`
const sha256 = (bytes) => createHash('sha256').update(bytes).digest('hex')

async function get(url) {
  const res = await fetch(url)
  if (!res.ok) throw new Error(`${url}: HTTP ${res.status}`)
  return Buffer.from(await res.arrayBuffer())
}

// La lista de hashes viene de la misma release: protege de descargas corruptas o cortadas,
// no de una release comprometida en GitHub.
const sums = (await get(base + 'SHA2-256SUMS')).toString('utf8')
const want = sums
  .split('\n')
  .map((line) => line.trim().split(/\s+/))
  .find(([, name]) => name === asset)?.[0]
if (!want) throw new Error(`${asset} no aparece en SHA2-256SUMS de ${VERSION}`)

if (existsSync(dest) && sha256(readFileSync(dest)) === want) {
  console.log(`yt-dlp ${VERSION} ya está en ${dest}`)
} else {
  console.log(`Descargando yt-dlp ${VERSION} (${asset})…`)
  const bytes = await get(base + asset)
  const got = sha256(bytes)
  if (got !== want) throw new Error(`SHA-256 no coincide: esperado ${want}, recibido ${got}`)
  mkdirSync('src-tauri/binaries', { recursive: true })
  // misma regla que la app: .part y rename, nunca un binario a medias con su nombre final
  writeFileSync(dest + '.part', bytes)
  chmodSync(dest + '.part', 0o755)
  renameSync(dest + '.part', dest)
  console.log(`yt-dlp ${VERSION} verificado (${got.slice(0, 12)}…) en ${dest}`)
}
