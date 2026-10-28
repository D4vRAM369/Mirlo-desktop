<script lang="ts">
  // Medios del post abierto: fotos en rejilla con visor a pantalla completa y vídeos inline.
  import * as api from './api'
  import type { FileRow } from './api'
  import { t, tn } from './i18n.svelte'

  let { id, dir, files }: { id: string; dir: string; files: FileRow[] } = $props()

  const photos = $derived(files.filter((f) => !f.path.endsWith('.mp4')))
  const clips = $derived(files.filter((f) => f.path.endsWith('.mp4')))
  // la rejilla muestra 3 fotos como mucho; la tercera lleva "+N" si hay más
  const shown = $derived(photos.slice(0, 3))

  let dialog: HTMLDialogElement
  let open = $state(0)

  function show(i: number) {
    open = i
    dialog.showModal()
  }

  const go = (step: number) => (open = (open + step + photos.length) % photos.length)

  function onKey(e: KeyboardEvent) {
    const step = e.key === 'ArrowRight' ? 1 : e.key === 'ArrowLeft' ? -1 : 0
    if (step) {
      e.preventDefault()
      go(step)
    }
  }

  const alt = (f: FileRow, i: number) => f.alt || t('gallery.photo_alt', { n: i + 1, total: photos.length })

  // Vídeos como blob: (WebKitGTK no reproduce <video> por el protocolo asset:). Se liberan al cambiar de post.
  let videos = $state<Record<string, string>>({})
  $effect(() => {
    const urls: string[] = []
    let alive = true
    videos = {}
    for (const f of clips) {
      api
        .videoUrl(id, f.path)
        .then((u) => {
          urls.push(u)
          if (alive) videos = { ...videos, [f.path]: u }
          else URL.revokeObjectURL(u)
        })
        .catch(() => {})
    }
    return () => {
      alive = false
      urls.forEach((u) => URL.revokeObjectURL(u))
    }
  })
</script>

{#each clips as f (f.path)}
  {#if videos[f.path]}
    <!-- X no da subtítulos de sus vídeos: una <track> vacía solo engañaría al lector de pantalla -->
    <!-- svelte-ignore a11y_media_has_caption -->
    <video class="clip" controls preload="metadata" src={videos[f.path]}></video>
  {:else}
    <div class="clip loading">{t('gallery.loading_video')}</div>
  {/if}
{/each}

{#if photos.length}
  <div class="gallery n{shown.length}">
    {#each shown as f, i (f.path)}
      <button onclick={() => show(i)} aria-label={t('gallery.view_full', { alt: alt(f, i) })}>
        <img src={api.fileUrl(dir, f.path)} alt="" loading="lazy" />
        {#if i === 2 && photos.length > 3}<span class="more">+{photos.length - 3}</span>{/if}
      </button>
    {/each}
  </div>
  <p class="hint">{tn('gallery.hint', photos.length)}</p>
{/if}

<dialog bind:this={dialog} onkeydown={onKey} aria-label={t('gallery.viewer')}>
  {#if photos[open]}
    <figure>
      <img src={api.fileUrl(dir, photos[open].path)} alt={alt(photos[open], open)} />
      <figcaption>
        <span>{t('gallery.counter', { n: open + 1, total: photos.length })}</span>
        {#if photos[open].alt}<span>{photos[open].alt}</span>{/if}
      </figcaption>
    </figure>
  {/if}
  <div class="controls">
    {#if photos.length > 1}
      <button class="btn" onclick={() => go(-1)}>{t('gallery.prev')}</button>
      <button class="btn" onclick={() => go(1)}>{t('gallery.next')}</button>
    {/if}
    <form method="dialog"><button class="btn">{t('gallery.close')}</button></form>
  </div>
</dialog>

<style>
  .clip {
    display: block;
    width: 100%;
    max-width: 720px;
    max-height: 70vh;
    border-radius: 6px;
    background: #000;
    margin-bottom: 12px;
  }
  .loading { display: grid; place-items: center; aspect-ratio: 16 / 9; color: #c9d1da; }
  .gallery {
    display: grid;
    gap: 6px;
    max-width: 720px;
    margin-bottom: 8px;
  }
  .gallery.n1 { grid-template-columns: 1fr; }
  .gallery.n2 { grid-template-columns: 1fr 1fr; aspect-ratio: 16 / 9; }
  .gallery.n3 { grid-template-columns: 2fr 1fr; grid-template-rows: 1fr 1fr; aspect-ratio: 16 / 9; }
  .gallery.n3 > :first-child { grid-row: span 2; }
  .gallery button {
    position: relative;
    border: 0;
    padding: 0;
    border-radius: 6px;
    overflow: hidden;
    cursor: zoom-in;
    background: var(--line);
    min-height: 0;
  }
  .gallery img { display: block; width: 100%; height: 100%; object-fit: cover; }
  .gallery.n1 img { height: auto; max-height: 70vh; object-fit: contain; }
  .more {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    background: rgba(0, 0, 0, 0.45);
    color: #fff;
    font-size: 22px;
    font-weight: 700;
  }
  .hint { color: var(--muted); font-size: 13px; margin: 0 0 22px; }
  dialog {
    border: 0;
    padding: 0;
    background: #000;
    color: #e3e9ef;
    max-width: 94vw;
    max-height: 94vh;
    border-radius: 8px;
  }
  dialog::backdrop { background: rgba(10, 14, 18, 0.85); }
  figure { margin: 0; }
  figure img { display: block; max-width: 94vw; max-height: calc(94vh - 52px); margin: 0 auto; }
  figcaption { display: flex; gap: 16px; padding: 8px 12px 0; font-size: 14px; }
  figcaption > :first-child { white-space: nowrap; }
  .controls { display: flex; justify-content: flex-end; gap: 8px; padding: 8px 12px 12px; }
  .controls form { margin: 0; }
  .controls .btn { background: #232c36; color: #e3e9ef; border-color: #34404c; }
</style>
