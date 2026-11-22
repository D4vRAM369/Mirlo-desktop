// ==UserScript==
// @name         Enviar a Mirlo
// @name:en      Send to Mirlo
// @namespace    https://github.com/D4vRAM369/Mirlo-desktop
// @version      0.1.0
// @description  Añade «Enviar a Mirlo» al menú Compartir de X.
// @description:en Adds "Send to Mirlo" to the Share menu on X.
// @author       D4vRAM369
// @license      GPL-3.0-or-later
// @homepageURL  https://github.com/D4vRAM369/Mirlo-desktop
// @match        https://x.com/*
// @match        https://www.x.com/*
// @match        https://mobile.x.com/*
// @match        https://twitter.com/*
// @grant        none
// @noframes
// @run-at       document-idle
// ==/UserScript==

// No lee cookies ni hace peticiones: solo saca el número del post y abre mirlo://save/<número>.
;(() => {
  'use strict'

  // X tiene dos interfaces: la nueva (data-engagement-action) y la clásica (data-testid).
  const OPEN_SHARE = [
    'button[data-engagement-action="share"][aria-expanded="true"]',
    'article[data-testid="tweet"] [role="group"] button[aria-haspopup="menu"][aria-expanded="true"]:not([data-testid])',
  ].join(',')
  const QUOTE = '[data-slot="embedded-tweet-frame"],[role="link"]'

  /**
   * El ID del post de ese botón. No vale "el primer enlace": en un post con cita, el primero es
   * el del citado. Vale el enlace de la fecha que pertenece a este article y no a una cita.
   * Si no está claro devuelve null: mejor sin botón que guardar otro post.
   */
  function postId(trigger, path) {
    const article = trigger.closest('article')
    if (!article) return path.match(/^\/[^/]+\/status\/(\d{1,20})(?:\/|$)/)?.[1] ?? null
    for (const a of article.querySelectorAll('a[href*="/status/"]')) {
      const id = a.getAttribute('href').match(/^\/[^/]+\/status\/(\d{1,20})$/)?.[1]
      if (!id || a.closest('article') !== article) continue
      const quote = a.parentElement.closest(QUOTE)
      if (quote && article.contains(quote)) continue
      if (a.querySelector('time') || a.hasAttribute('data-base-ui-tooltip-trigger')) return id
    }
    return null
  }

  // `npm run check` carga este archivo en Node, donde no hay página: solo prueba postId.
  if (typeof document === 'undefined') {
    module.exports = { postId }
    return
  }

  const ITEM = 'data-mirlo-send'
  const BIRD = 'M2 14c4 0 6-2 7-5a5 5 0 0 1 9-2l4 1-3 2c0 6-4 10-10 10H5l3-3c-3 0-5-1-6-3z'

  function send(item) {
    const id = item.getAttribute(ITEM)
    // X cierra sus menús con Escape
    item.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', code: 'Escape', bubbles: true }))
    location.href = `mirlo://save/${id}`
  }

  /** Copia una opción nativa para heredar su aspecto; clonar no copia lo que hacía al pulsarla. */
  function build(model) {
    const item = model.cloneNode(true)
    for (const name of ['id', 'data-testid', 'data-disabled', 'aria-disabled', 'href']) item.removeAttribute(name)
    item.setAttribute('tabindex', '0')
    item.setAttribute('data-highlighted', '') // el hover de X solo se pinta con este atributo

    const label = document.documentElement.lang.startsWith('es') ? 'Enviar a Mirlo' : 'Send to Mirlo'
    const texts = []
    const walker = document.createTreeWalker(item, NodeFilter.SHOW_TEXT)
    while (walker.nextNode()) if (walker.currentNode.data.trim()) texts.push(walker.currentNode)
    texts.forEach((node, i) => (node.data = i ? '' : label))

    const svg = item.querySelector('svg')
    if (svg) {
      const path = document.createElementNS('http://www.w3.org/2000/svg', 'path')
      path.setAttribute('d', BIRD)
      svg.replaceChildren(path)
      svg.removeAttribute('data-icon')
    }

    item.addEventListener('click', (e) => {
      e.preventDefault()
      e.stopPropagation()
      if (e.isTrusted) send(item)
    })
    return item
  }

  function sync() {
    const trigger = document.querySelector(OPEN_SHARE)
    if (!trigger) return
    const controls = trigger.getAttribute('aria-controls')
    const menu = controls ? document.getElementById(controls) : [...document.querySelectorAll('[role="menu"]')].at(-1)
    if (!menu) return

    let item = menu.querySelector(`[${ITEM}]`)
    const id = postId(trigger, location.pathname)
    if (!id) return void item?.remove()
    if (!item) {
      const model = [...menu.querySelectorAll('[role="menuitem"]')].at(-1)
      if (!model) return
      item = build(model)
      model.after(item)
    }
    // X reutiliza menús al navegar: el número se actualiza siempre
    if (item.getAttribute(ITEM) !== id) item.setAttribute(ITEM, id)
  }

  // Las flechas de X solo conocen sus opciones: aquí se enlaza la nuestra al final de la lista.
  // En window y en captura para llegar antes que los manejadores de X.
  window.addEventListener('keydown', (e) => {
    if (!e.isTrusted) return
    const current = e.target.closest?.('[role="menuitem"]')
    const menu = current?.closest('[role="menu"]')
    const item = menu?.querySelector(`[${ITEM}]`)
    if (!item) return
    const all = [...menu.querySelectorAll('[role="menuitem"]')]
    const i = all.indexOf(current)
    let next
    if (current === item && (e.key === 'Enter' || e.key === ' ')) next = null
    else if (e.key === 'ArrowDown' && (current === item || all[i + 1] === item)) next = all[(i + 1) % all.length]
    else if (e.key === 'ArrowUp' && (current === item || i === 0)) next = all.at(i - 1)
    else return
    e.preventDefault()
    e.stopImmediatePropagation()
    if (next) next.focus()
    else send(item)
  }, true)

  new MutationObserver(sync).observe(document.body, {
    childList: true,
    subtree: true,
    attributes: true,
    attributeFilter: ['aria-expanded', 'aria-controls'],
  })
})()
