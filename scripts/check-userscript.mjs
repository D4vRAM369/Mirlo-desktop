// De qué post es cada botón Compartir (si falla, se guarda otro post sin avisar) y cuándo saluda a Mirlo.
// Estructuras copiadas del DOM de x.com (2026-11-22) y del DOM clásico (data-testid).
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { runInNewContext } from 'node:vm'

// node:vm y no import: con "type": "module" Node trataría el .user.js como ESM, y no lo es
const sandbox = { module: { exports: {} } }
runInNewContext(readFileSync(new URL('../userscript/mirlo-x.user.js', import.meta.url), 'utf8'), sandbox)
const { postId, helloCode, helloLink } = sandbox.module.exports

/** Enlace mínimo: solo lo que el script consulta de un <a>. */
const link = (href, { date = true, time = false, nested = false, frame = false } = {}) => (article) => ({
  getAttribute: () => href,
  hasAttribute: () => date,
  querySelector: () => (time ? {} : null),
  closest: () => (nested ? {} : article),
  parentElement: { closest: () => (frame ? { inArticle: true } : null) },
})
function button(...links) {
  const article = { contains: (el) => el.inArticle === true }
  article.querySelectorAll = () => links.map((make) => make(article))
  return { closest: () => article }
}

const cases = [
  ['post normal', button(link('/jack/status/20')), '/jack', '20'],
  ['cita: su enlace va ANTES que el del post', button(
    link('/cited/status/111', { date: false, frame: true }),
    link('/cited/status/111', { nested: true, frame: true }),
    link('/author/status/222'),
  ), '/author/status/222', '222'],
  ['respuesta dentro de la página de otro post', button(link('/reply/status/333')), '/author/status/222', '333'],
  ['enlace a otro post dentro del texto', button(link('/x/status/444', { date: false }), link('/a/status/555')), '/a', '555'],
  ['DOM clásico: la fecha es un <time>', button(link('/a/status/666', { date: false, time: true })), '/home', '666'],
  ['una foto no es el enlace del post', button(link('/a/status/777/photo/1'), link('/a/status/888')), '/a', '888'],
  ['sin fecha reconocible: no se adivina', button(link('/a/status/999', { date: false })), '/a/status/1', null],
  ['visor de foto, fuera de article', { closest: () => null }, '/a/status/123/photo/1', '123'],
  ['fuera de article y sin post en la URL', { closest: () => null }, '/home', null],
]
for (const [name, trigger, path, expected] of cases) assert.equal(postId(trigger, path), expected, name)
console.log(`userscript: ${cases.length} casos de identificación del post, correctos`)

const greetings = [
  ['#mirlo-hello-18446744073709551615', '18446744073709551615'],
  ['', null],
  ['#mirlo-hello-', null],
  ['#mirlo-hello-12ab', null],
  ['#mirlo-hello-123456789012345678901', null],
  ['#mirlo-hello-12/../../save/20', null],
  ['#x-mirlo-hello-12', null],
]
for (const [hash, expected] of greetings) assert.equal(helloCode(hash), expected, `saludo: ${hash}`)
assert.equal(helloLink({ isTrusted: true }, '42'), 'mirlo://hello/42', 'un clic real saluda')
assert.equal(helloLink({ isTrusted: false }, '42'), null, 'un clic fabricado por la web no saluda')
console.log(`userscript: ${greetings.length + 2} casos del saludo, correctos`)
