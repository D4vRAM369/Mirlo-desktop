<script lang="ts">
  // Pantalla provisional de las fases 1 y 2: prueba el extractor y el guardado. La UI "Bandeja" llega en la fase 3.
  import { invoke } from '@tauri-apps/api/core'

  let url = $state('')
  let result = $state('')
  let error = $state('')
  let busy = $state(false)

  async function run(command: 'extract_post' | 'save_post') {
    busy = true
    error = ''
    result = ''
    try {
      result = JSON.stringify(await invoke(command, { url }), null, 2)
    } catch (e) {
      error = String(e)
    } finally {
      busy = false
    }
  }
</script>

<main class="mx-auto max-w-3xl p-6 font-sans">
  <h1 class="mb-4 text-2xl font-bold">XVault: prueba del extractor</h1>
  <form class="flex gap-2" onsubmit={(e) => { e.preventDefault(); run('save_post') }}>
    <input class="flex-1 rounded border px-3 py-2" bind:value={url} placeholder="https://x.com/usuario/status/…" aria-label="Enlace del post" />
    <button class="rounded bg-blue-700 px-4 py-2 font-bold text-white disabled:opacity-50" disabled={busy || !url}>
      {busy ? 'Trabajando…' : 'Guardar post'}
    </button>
    <button type="button" class="rounded border px-4 py-2 disabled:opacity-50" disabled={busy || !url} onclick={() => run('extract_post')}>
      Solo leer
    </button>
  </form>
  {#if error}<p class="mt-4 text-red-700" role="alert">{error}</p>{/if}
  {#if result}<pre class="mt-4 overflow-auto rounded bg-slate-100 p-4 text-sm">{result}</pre>{/if}
</main>
