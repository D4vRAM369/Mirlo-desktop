<script lang="ts">
  // Menú "Traducir todos": traduce los posts guardados que aún no tienen traducción al idioma de la app.
  // Recorre la lista uno a uno con el mismo comando que el botón de cada post. Nada sale hacia Google
  // hasta el segundo clic; al primer error se para y, al volver a pulsar, sigue con los que faltan.
  import * as api from './api'
  import { errorParts, i18n, t } from './i18n.svelte'

  let { onmessage, ondone }: { onmessage: (msg: string) => void; ondone: () => void } = $props()

  // ponytail: pausa fija entre posts para que Google no responda 429; subirla si vuelve a pasar
  const PAUSE_MS = 600

  let pending = $state<string[] | null>(null)
  let done = $state(0)
  let total = $state(0)
  let running = $state(false)
  let stop = false

  async function onToggle(e: Event) {
    if ((e.currentTarget as HTMLDetailsElement).open && !running) {
      pending = null
      pending = await api.untranslatedIds(i18n.lang).catch(() => [])
    }
  }

  async function run(menu: HTMLDetailsElement) {
    const ids = pending ?? []
    const lang = i18n.lang // fijo: cambiar de idioma a mitad no mezcla destinos
    menu.open = false
    running = true
    stop = false
    done = 0
    total = ids.length
    try {
      for (const id of ids) {
        if (stop) break
        await api.translatePost(id, lang)
        done++
        if (done < total) await new Promise((r) => setTimeout(r, PAUSE_MS))
      }
      onmessage(stop ? t('tall.stopped', { done, total }) : t('tall.finished', { done }))
    } catch (e) {
      onmessage(t('tall.failed', { done, total, error: errorParts(e).text }))
    } finally {
      running = false
      ondone()
    }
  }
</script>

{#if running}
  <button type="button" class="btn" onclick={() => (stop = true)}>{t('tall.stop', { done, total })}</button>
{:else}
  <details class="menu" ontoggle={onToggle}>
    <summary class="btn">{t('tall.menu')}</summary>
    <div class="panel">
      {#if pending === null}
        <p>…</p>
      {:else if pending.length === 0}
        <p>{t('tall.none')}</p>
      {:else}
        <p>{t('tall.warning', { n: pending.length })}</p>
        <button type="button" class="btn primary" onclick={(e) => run((e.currentTarget as HTMLElement).closest('details')!)}>
          {t('tall.start', { n: pending.length })}
        </button>
      {/if}
    </div>
  </details>
{/if}

<style>
  .menu { position: relative; }
  summary { list-style: none; cursor: pointer; }
  summary::-webkit-details-marker { display: none; }
  .panel { position: absolute; right: 0; top: calc(100% + 6px); z-index: 10; display: grid; gap: 8px; width: 280px; padding: 10px; border: 1px solid var(--line); border-radius: 8px; background: var(--paper); box-shadow: 0 6px 20px rgb(0 0 0 / 0.18); }
  p { margin: 0; font-size: 14px; }
</style>
