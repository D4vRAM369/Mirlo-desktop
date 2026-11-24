<script lang="ts">
  // Integrar con X (SPEC, parte E): la oferta que sale una vez y el asistente de cuatro pasos.
  // Mirlo no instala nada en el navegador: abre cada página y la persona confirma allí.
  import { onMount } from 'svelte'
  import * as api from './api'
  import { errorText, t } from './i18n.svelte'

  let { connected = $bindable(false), onmessage }: { connected: boolean; onmessage: (msg: string) => void } = $props()

  function recall(key: string) {
    try {
      return localStorage.getItem(key) === '1'
    } catch {
      return false
    }
  }
  function remember(key: string) {
    try {
      localStorage.setItem(key, '1')
    } catch {}
  }

  connected = recall('x-connected')
  let offered = $state(recall('x-offered'))
  let dialog: HTMLDialogElement
  let browser = $state<api.Browser>('firefox')
  let status = $state<api.XStatus | null>(null)
  let script = $state('') // se pide al abrir: el portapapeles solo acepta escribir durante el propio clic
  let installUrl = $state('')
  let checkUrl = $state('')
  let waiting = $state(false)
  let copied = $state(false)
  let registering = $state(false)
  let error = $state('')

  async function attempt(work: () => Promise<unknown>) {
    error = ''
    try {
      await work()
    } catch (e) {
      error = errorText(e)
    }
  }

  export function show() {
    dismiss()
    installUrl = checkUrl = ''
    copied = waiting = false
    status = null
    dialog.showModal()
    attempt(async () => {
      ;[status, script] = await Promise.all([api.xStatus(), api.xScript()])
    })
  }

  function dismiss() {
    offered = true
    remember('x-offered')
  }

  const register = () =>
    attempt(async () => {
      registering = true
      try {
        status = await api.xRegister()
      } finally {
        registering = false
      }
    })

  const install = () => attempt(async () => (installUrl = await api.xInstall()))

  const copy = () =>
    attempt(async () => {
      await navigator.clipboard.writeText(script).catch(() => Promise.reject('copy_failed'))
      copied = true
    })

  const check = () =>
    attempt(async () => {
      checkUrl = await api.xCheck()
      waiting = true
    })

  async function collectGreeting() {
    if (!(await api.xTakeConnected())) return
    connected = true
    waiting = false
    checkUrl = ''
    remember('x-connected')
    dismiss()
    onmessage(t('x.connected'))
    if (dialog.open) status = await api.xStatus()
  }

  onMount(() => {
    const failed = (e: unknown) => onmessage(errorText(e))
    // primero escuchar y después recoger: un saludo entre medias no se pierde
    const listening = api.onXConnected(() => collectGreeting().catch(failed))
    listening.then(collectGreeting).catch(failed)
    return () => void listening.then((stop) => stop())
  })
</script>

{#if !offered && !connected}
  <section class="offer" aria-labelledby="x-offer-title">
    <p><b id="x-offer-title">{t('x.offer')}</b> {t('x.offer_why')}</p>
    <button type="button" class="btn primary" onclick={show}>{t('x.integrate')}</button>
    <button type="button" class="btn" onclick={dismiss}>{t('x.not_now')}</button>
  </section>
{/if}

<dialog bind:this={dialog} aria-labelledby="x-title">
  <header>
    <h2 id="x-title">{t('x.title')}</h2>
    <button type="button" class="btn" onclick={() => dialog.close()}>{t('x.close')}</button>
  </header>

  <div class="links" role="status">
    {#if !status}
      <p>{t('x.links_checking')}</p>
    {:else}
      <p class="st" class:ok={status.links === 'here'} class:partial={status.links === 'other' || status.links === 'nobody'}>
        {t(`x.links_${status.links}`)}
      </p>
      {#if status.links !== 'here'}
        {#if status.can_register}
          <p class="note">{t(status.links === 'other' ? 'x.register_note_other' : 'x.register_note')}</p>
          <button type="button" class="btn primary" disabled={registering} onclick={register}>{t('x.register')}</button>
        {:else if status.links !== 'unknown'}
          <p class="note">{t('x.use_installer')}</p>
        {/if}
      {/if}
    {/if}
  </div>

  <fieldset>
    <legend>{t('x.browser')}</legend>
    <label><input type="radio" bind:group={browser} value="firefox" /> Firefox</label>
    <label><input type="radio" bind:group={browser} value="chromium" /> Chrome, Edge, Brave, Chromium</label>
  </fieldset>

  <ol>
    <li>
      <p>{t('x.manager')}</p>
      <p class="note">{t('x.manager_note')}</p>
      <button type="button" class="btn" onclick={() => attempt(() => api.xOpenManager('violentmonkey'))}>Violentmonkey</button>
      <button type="button" class="btn" onclick={() => attempt(() => api.xOpenManager('tampermonkey'))}>Tampermonkey</button>
    </li>
    {#if browser === 'chromium'}
      <li>
        <p>{t('x.allow_scripts')}</p>
        <p class="note">{t('x.allow_scripts_note')}</p>
      </li>
    {/if}
    <li>
      <p>{t('x.install_step')}</p>
      <button type="button" class="btn" onclick={install}>{t('x.install')}</button>
      <button type="button" class="btn" disabled={!script} onclick={copy}>{t('x.copy')}</button>
      {#if copied}<p class="note" role="status">{t('x.copied')}</p>{/if}
      {#if installUrl}
        <p class="note">{t('x.install_url')}</p>
        <code>{installUrl}</code>
      {/if}
    </li>
    <li>
      <p>{t('x.check_step')}</p>
      <button type="button" class="btn" onclick={check}>{t('x.check')}</button>
      {#if checkUrl}
        <p class="note">{t('x.check_url')}</p>
        <code>{checkUrl}</code>
      {/if}
      <p class="st" class:ok={connected && !waiting} role="status">
        {t(waiting ? 'x.waiting' : connected ? 'x.connected' : 'x.unchecked')}
      </p>
    </li>
  </ol>

  {#if error}<p class="error" role="alert">✕ {error}</p>{/if}
</dialog>

<style>
  .offer { display: flex; flex-wrap: wrap; align-items: center; gap: 8px 10px; padding: 8px 14px; border-bottom: 1px solid var(--line); background: var(--accent-soft); }
  .offer p { flex: 1; min-width: 260px; margin: 0; }

  dialog { width: min(680px, 92vw); max-height: 90vh; overflow-y: auto; padding: 18px 22px; border: 1px solid var(--line); border-radius: 10px; background: var(--paper); color: var(--ink); }
  dialog::backdrop { background: rgb(0 0 0 / 0.45); }
  header { display: flex; justify-content: space-between; align-items: center; gap: 10px; margin-bottom: 12px; }
  h2 { margin: 0; font-size: 20px; }
  p { margin: 0 0 6px; }
  .links { padding: 10px 12px; border-radius: 6px; background: var(--pane); }
  .links p:last-child { margin: 0; }
  fieldset { display: flex; flex-wrap: wrap; gap: 6px 18px; margin: 14px 0 0; padding: 0; border: 0; }
  legend { font-weight: 700; padding: 0 0 6px; }
  fieldset label { display: flex; align-items: center; gap: 6px; cursor: pointer; }
  ol { margin: 12px 0 0; padding-left: 22px; }
  li { padding: 8px 0; }
  li + li { border-top: 1px solid var(--line); }
  li .btn { margin: 0 6px 6px 0; }
  .note { color: var(--muted); font-size: 14px; }
  code { display: block; margin-bottom: 6px; padding: 5px 8px; border-radius: 4px; background: var(--pane); overflow-wrap: anywhere; user-select: all; }
  .error { padding: 8px 10px; border-radius: 6px; background: var(--err-soft); color: var(--err); }
</style>
