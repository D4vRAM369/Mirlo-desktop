<script lang="ts">
  // Menú "Copia de seguridad": exportar la bóveda a un .zip o importar uno (fusiona, nunca pisa).
  // <details> nativo: se abre y cierra con teclado y ratón sin código, también en WebKitGTK.
  import { open, save } from '@tauri-apps/plugin-dialog'
  import * as api from './api'
  import { errorText, t } from './i18n.svelte'

  let { onmessage, ondone }: { onmessage: (msg: string) => void; ondone: () => void } = $props()

  let withMedia = $state(true)
  let busy = $state(false)
  let menu: HTMLDetailsElement
  const zip = [{ name: 'Zip', extensions: ['zip'] }]

  async function run(work: () => Promise<string | null>) {
    menu.open = false
    busy = true
    try {
      const msg = await work()
      if (msg) onmessage(msg)
    } catch (e) {
      onmessage(errorText(e))
    } finally {
      busy = false
    }
  }

  const exportVault = () =>
    run(async () => {
      const dest = await save({ defaultPath: `mirlo-export-${new Date().toISOString().slice(0, 10)}.zip`, filters: zip })
      if (!dest) return null
      onmessage(t('backup.exporting'))
      const r = await api.exportVault(dest, withMedia)
      return t('backup.exported', { posts: r.posts, files: r.files })
    })

  const importVault = () =>
    run(async () => {
      const src = await open({ multiple: false, directory: false, filters: zip })
      if (!src) return null
      onmessage(t('backup.importing'))
      const r = await api.importVault(src)
      ondone()
      const parts = [t('backup.imported', { added: r.added, merged: r.merged })]
      if (r.bad_hash.length) parts.push(t('backup.bad_hash', { n: r.bad_hash.length }))
      if (r.errors.length) parts.push(t('backup.errors', { n: r.errors.length, first: r.errors[0] }))
      return parts.join(' ')
    })
</script>

<details class="backup" bind:this={menu}>
  <summary class="btn" aria-busy={busy}>{busy ? (t('backup.menu') + '…') : t('backup.menu')}</summary>
  <div class="panel">
    <label><input type="checkbox" bind:checked={withMedia} /> {t('backup.with_media')}</label>
    <button type="button" class="btn" disabled={busy} onclick={exportVault}>{t('backup.export')}</button>
    <button type="button" class="btn" disabled={busy} onclick={importVault}>{t('backup.import')}</button>
  </div>
</details>

<style>
  .backup { position: relative; }
  summary { list-style: none; cursor: pointer; }
  summary::-webkit-details-marker { display: none; }
  .panel { position: absolute; right: 0; top: calc(100% + 6px); z-index: 10; display: grid; gap: 8px; width: max-content; padding: 10px; border: 1px solid var(--line); border-radius: 8px; background: var(--paper); box-shadow: 0 6px 20px rgb(0 0 0 / 0.18); }
  label { display: flex; gap: 6px; align-items: center; font-size: 14px; }
  .panel .btn { justify-self: stretch; }
</style>
