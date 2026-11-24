<script lang="ts">
  // Conectar el móvil: deja Syncthing listo para la app de Android sin tocar Syncthing a mano.
  // Mirlo prepara el ordenador (carpeta, dispositivo, compartir); en el móvil solo se escanea y se acepta.
  import * as api from './api'
  import { errorText, t } from './i18n.svelte'

  let dialog: HTMLDialogElement
  let status = $state<api.PhoneStatus | null>(null)
  let qr = $state('')
  let accepting = $state('')
  let error = $state('')
  let timer: ReturnType<typeof setInterval> | undefined

  async function refresh() {
    try {
      const s = await api.phoneStatus()
      if (!qr) qr = URL.createObjectURL(new Blob([new Uint8Array(s.qr)], { type: 'image/png' }))
      status = s
      error = ''
    } catch (e) {
      error = errorText(e)
    }
  }

  export function show() {
    dialog.showModal()
    refresh()
    // ponytail: sondeo cada 2 s mientras está abierto; el móvil aparece en pendientes al escanear
    timer = setInterval(refresh, 2000)
  }

  function closed() {
    clearInterval(timer)
  }

  async function accept(d: api.PhoneDevice) {
    accepting = d.id
    try {
      await api.phoneAccept(d.id, d.name)
      await refresh()
    } catch (e) {
      error = errorText(e)
    } finally {
      accepting = ''
    }
  }
</script>

<dialog bind:this={dialog} onclose={closed} aria-labelledby="phone-title">
  <header>
    <h2 id="phone-title">{t('phone.title')}</h2>
    <button type="button" class="btn" onclick={() => dialog.close()}>{t('x.close')}</button>
  </header>
  <p class="note">{t('phone.intro')}</p>

  {#if error}
    <p class="error" role="alert">✕ {error}</p>
  {/if}

  {#if status}
    <ol>
      <li>
        <p>{t('phone.step_scan')}</p>
        <img src={qr} alt={t('phone.qr_alt')} width="200" height="200" />
        <code>{status.my_id}</code>
      </li>
      <li>
        <p>{t('phone.step_accept')}</p>
        {#each status.pending as d (d.id)}
          <p class="pending">
            <b>{d.name || d.id.slice(0, 7)}</b>
            <button type="button" class="btn primary" disabled={accepting === d.id} onclick={() => accept(d)}>{t('phone.accept')}</button>
          </p>
        {:else}
          <p class="st" role="status">{t('phone.waiting')}</p>
        {/each}
      </li>
      <li>
        <p>{t('phone.step_folder')}</p>
        {#each status.phones as d (d.id)}
          <p class="st ok">✓ {t('phone.shared', { name: d.name || d.id.slice(0, 7) })}</p>
        {/each}
      </li>
    </ol>
  {/if}
</dialog>

<style>
  dialog { width: min(620px, 92vw); max-height: 90vh; overflow-y: auto; padding: 18px 22px; border: 1px solid var(--line); border-radius: 10px; background: var(--paper); color: var(--ink); }
  dialog::backdrop { background: rgb(0 0 0 / 0.45); }
  header { display: flex; justify-content: space-between; align-items: center; gap: 10px; margin-bottom: 12px; }
  h2 { margin: 0; font-size: 20px; }
  p { margin: 0 0 6px; }
  ol { margin: 12px 0 0; padding-left: 22px; }
  li { padding: 8px 0; }
  li + li { border-top: 1px solid var(--line); }
  img { display: block; margin: 6px 0; border-radius: 6px; background: #fff; image-rendering: pixelated; }
  .pending { display: flex; align-items: center; gap: 10px; }
  .note { color: var(--muted); font-size: 14px; }
  code { display: block; margin-bottom: 6px; padding: 5px 8px; border-radius: 4px; background: var(--pane); overflow-wrap: anywhere; user-select: all; }
  .error { padding: 8px 10px; border-radius: 6px; background: var(--err-soft); color: var(--err); }
</style>
