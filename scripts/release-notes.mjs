// Descripción de la release en inglés y español con selector, sacada de CHANGELOG.md.
//   node scripts/release-notes.mjs v0.2.0   la imprime (en Actions, además, output `body`)
//   node scripts/release-notes.mjs --test   comprueba todas las versiones del changelog
import assert from 'node:assert/strict'
import { randomUUID } from 'node:crypto'
import { appendFileSync, readFileSync } from 'node:fs'

const REPO = process.env.GITHUB_REPOSITORY ?? 'D4vRAM369/Mirlo-desktop'
const changelog = readFileSync(new URL('../CHANGELOG.md', import.meta.url), 'utf8')

function notes(text, tag) {
  const sections = text.replace(/\r\n/g, '\n').split(/^## /m).filter((s) => /^\[\d/.test(s))
  const version = (s) => s.slice(1, s.indexOf(']'))
  const i = sections.findIndex((s) => `v${version(s)}` === tag)
  if (i < 0) throw new Error(`CHANGELOG.md no tiene la versión ${tag}`)

  const [head, ...parts] = sections[i].split(/^### /m)
  const part = (title) => {
    const found = parts.find((p) => p.split('\n', 1)[0].trim() === title)
    const body = found?.slice(found.indexOf('\n')).trim()
    if (!body) throw new Error(`${tag}: falta «### ${title}» en CHANGELOG.md`)
    // Un apartado propio de la versión (#### Instalar…) queda al nivel de los demás
    return body.replace(/^#### /gm, '### ')
  }
  // Las líneas comunes a los dos idiomas se escriben «español · *English*»
  const pair = (line) => {
    const found = line.match(/^(.+) · \*(.+)\*$/)
    if (!found) throw new Error(`${tag}: línea sin «español · *English*»: ${line}`)
    return found.slice(1)
  }

  const intro = head.slice(head.indexOf('\n')).trim().split('\n').filter(Boolean).map(pair)
  const limitsTitle = 'Limitaciones conocidas · *Known limitations*'
  const limits = parts.some((p) => p.startsWith(limitsTitle))
    ? part(limitsTitle).split('\n').filter(Boolean).map((line) => pair(line.replace(/^- /, '')))
    : []

  const block = (name, lang, changes, limitations, open = '') => {
    const body = [
      ...intro.map((p) => p[lang]),
      `### ${changes}`,
      part(name),
      ...(limits.length ? [`### ${limitations}`, limits.map((p) => `- ${p[lang]}`).join('\n')] : []),
    ].join('\n\n')
    // La línea en blanco tras </summary> hace falta para que GitHub interprete el Markdown
    return `<details name="lang"${open}>\n<summary><b>${name}</b></summary>\n\n${body}\n\n</details>`
  }

  const previous = sections[i + 1]
  return [
    '**Language · Idioma**',
    block('English', 1, 'Changelog', 'Known limitations', ' open'),
    block('Español', 0, 'Cambios', 'Limitaciones conocidas'),
    [
      '> [!WARNING]',
      '> Windows and macOS are untested and their installers are unsigned, so the OS will show a security warning.',
      '>',
      '> Windows y macOS no se han probado y sus instaladores no están firmados: el sistema mostrará un aviso de seguridad.',
    ].join('\n'),
    ...(previous
      ? [`**Full changelog · Cambios completos:** https://github.com/${REPO}/compare/v${version(previous)}...${tag}`]
      : []),
  ].join('\n\n')
}

function test() {
  for (const [, v] of changelog.matchAll(/^## \[(\d[^\]]*)\]/gm)) notes(changelog, `v${v}`)

  const sample = '## [1.1.0] — hoy\n\nHola. · *Hello.*\n\n### Español\n\n- Uno\n\n### English\n\n- One\n\n'
  const limit = '### Limitaciones conocidas · *Known limitations*\n\n- Pega. · *Catch.*\n\n'
  const out = notes(`${sample}${limit}## [1.0.0] — ayer\n`.replace(/\n/g, '\r\n'), 'v1.1.0')
  const [en, es] = out.match(/<details[^]*?<\/details>/g)
  assert.match(en, / open>[^]*Hello\.[^]*- One[^]*- Catch\./)
  assert.doesNotMatch(en, /Hola\.|- Uno|Pega\./)
  assert.match(es, /^<details name="lang">[^]*Hola\.[^]*- Uno[^]*- Pega\./)
  assert.doesNotMatch(es, /Hello\.|- One|Catch\./)
  assert.match(out, /compare\/v1\.0\.0\.\.\.v1\.1\.0$/)
  assert.throws(() => notes(sample, 'v9.9.9'), /no tiene/)
  assert.throws(() => notes(sample.replace('### English', '### Otro'), 'v1.1.0'), /falta/)
  assert.throws(() => notes(sample.replace(' · *Hello.*', ''), 'v1.1.0'), /sin/)
  console.log('notas de release: OK')
}

const arg = process.argv[2] ?? process.env.GITHUB_REF_NAME
if (arg === '--test') test()
else {
  const body = notes(changelog, arg)
  console.log(body)
  if (process.env.GITHUB_OUTPUT) {
    const end = randomUUID()
    appendFileSync(process.env.GITHUB_OUTPUT, `body<<${end}\n${body}\n${end}\n`)
  }
}
