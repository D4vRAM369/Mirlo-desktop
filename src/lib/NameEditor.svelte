<script lang="ts">
  // Editor en línea de una colección o una etiqueta del sidebar: nombre, color (solo etiquetas) y borrar.
  // Enter guarda el nombre, Escape cierra. Borrar pide una segunda pulsación: no hay papelera.
  import { untrack } from 'svelte'
  import { TAG_COLORS, tagColor } from './api'
  import ColorWheel from './ColorWheel.svelte'
  import { t } from './i18n.svelte'

  let { name, color, deleteHint, onrename, oncolor, ondelete, onclose }: {
    name: string
    /** undefined = colección (sin color) */
    color?: string | null
    deleteHint: string
    onrename: (name: string) => void
    oncolor?: (color: string | null) => void
    ondelete: () => void
    onclose: () => void
  } = $props()

  // el campo arranca con el nombre actual y después es de quien escribe
  let value = $state(untrack(() => name))
  let confirming = $state(false)
  let wheelOpen = $state(false)
  const custom = $derived(!!color?.startsWith('#'))

  // `autofocus` solo vale al cargar la página: un campo que aparece después se enfoca a mano
  const focusOnMount = (node: HTMLInputElement) => {
    node.focus()
    node.select()
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault()
      value.trim() === name ? onclose() : onrename(value)
    } else if (e.key === 'Escape') {
      e.preventDefault()
      onclose()
    }
  }
</script>

<div class="editor">
  <input use:focusOnMount bind:value onkeydown={onKey} aria-label={t('edit.name')} />
  {#if color !== undefined && oncolor}
    <div class="swatches" role="group" aria-label={t('edit.color')}>
      <button class="swatch none" aria-pressed={color === null} aria-label={t('edit.no_color')} title={t('edit.no_color')} onclick={() => oncolor(null)}></button>
      {#each TAG_COLORS as c}
        <button class="swatch" style:background="var(--tag-{c})" aria-pressed={color === c} aria-label={t(`color.${c}`)} title={t(`color.${c}`)} onclick={() => oncolor(c)}></button>
      {/each}
      <!-- 9.º círculo: rueda para un color propio; si la etiqueta ya tiene uno, se ve ese -->
      <button
        class="swatch more"
        style:background={custom ? tagColor(color ?? null) : null}
        aria-pressed={custom}
        aria-expanded={wheelOpen}
        aria-label={t('edit.custom_color')}
        title={t('edit.custom_color')}
        onclick={() => (wheelOpen = !wheelOpen)}
      ></button>
    </div>
    {#if wheelOpen}
      <ColorWheel value={custom ? (color ?? null) : null} onpick={(hex) => oncolor(hex)} />
    {/if}
  {/if}
  <div class="actions">
    {#if confirming}
      <button class="btn danger" onclick={ondelete}>{t('edit.confirm_delete')}</button>
      <button class="btn" onclick={() => (confirming = false)}>{t('edit.cancel')}</button>
      <small>{deleteHint}</small>
    {:else}
      <button class="btn" onclick={() => (value.trim() === name ? onclose() : onrename(value))}>{t('edit.done')}</button>
      <button class="btn" onclick={() => (confirming = true)}>{t('edit.delete')}</button>
    {/if}
  </div>
</div>

<style>
  .editor { min-width: 0; margin: 2px 0 6px; padding: 8px; border: 1px solid var(--accent); border-radius: 6px; background: var(--paper); display: grid; gap: 8px; }
  input { width: 100%; background: var(--pane); border: 1px solid var(--line); border-radius: 6px; padding: 4px 6px; }
  .swatches { display: flex; flex-wrap: wrap; gap: 5px; }
  .swatch { width: 20px; height: 20px; border-radius: 50%; border: 1px solid var(--line); cursor: pointer; padding: 0; }
  .swatch[aria-pressed='true'] { outline: 2px solid var(--ink); outline-offset: 1px; }
  .swatch.more { background: conic-gradient(red, yellow, lime, aqua, blue, magenta, red); }
  .swatch.none { background: linear-gradient(135deg, transparent 45%, var(--muted) 45% 55%, transparent 55%); }
  .actions { display: flex; flex-wrap: wrap; gap: 6px; align-items: center; }
  .actions .btn { padding: 3px 9px; font-size: 13px; }
  .danger { color: var(--err); border-color: var(--err); }
  small { color: var(--muted); font-size: 12px; }
</style>
