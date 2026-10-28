<script lang="ts">
  // UI básica de la fase 3 con el esqueleto "Bandeja": comando arriba, filtros, lista y lectura.
  // El estilo final (tokens, tema oscuro, galería con visor) llega en la fase 4.
  import * as api from './lib/api'
  import type { Detail, Facets, Filter, Kind, PostRow, Status } from './lib/api'

  let input = $state('')
  let filter = $state<Filter>({})
  let posts = $state<PostRow[]>([])
  let counts = $state<Facets>({ authors: [], ok: 0, partial: 0, error: 0 })
  let selectedId = $state<string | null>(null)
  let detail = $state<Detail | null>(null)
  let saving = $state<string[]>([]) // enlaces en cola de guardado
  let message = $state('')

  let field: HTMLInputElement
  const rows: Record<string, HTMLElement> = {}

  const KINDS: [Kind, string][] = [
    ['video', 'Con vídeo'],
    ['photo', 'Con fotos'],
    ['gif', 'Con GIF'],
    ['document', 'Con documentos'],
    ['text', 'Solo texto'],
  ]
  const STATUS: Record<Status, string> = { ok: '✓ Completo', partial: '◐ Parcial', error: '✕ Error' }
  const total = $derived(counts.ok + counts.partial + counts.error)
  const media = $derived(detail?.files.filter((f) => f.status === 'ok' && f.path.startsWith('media/')) ?? [])

  async function refresh() {
    try {
      ;[posts, counts] = await Promise.all([api.listPosts(filter), api.facets()])
    } catch (e) {
      message = String(e)
    }
  }

  // Al cambiar un filtro se recarga la lista; la búsqueda espera a que dejes de teclear.
  $effect(() => {
    const snapshot = { ...filter }
    const t = setTimeout(() => refresh(), snapshot.q ? 150 : 0)
    return () => clearTimeout(t)
  })

  $effect(() => {
    const id = selectedId
    if (!id) return void (detail = null)
    api.getPost(id).then((d) => {
      if (selectedId === id) detail = d
    })
  })

  // Vídeos del post abierto como blob: (ver api.videoUrl). Se liberan al cambiar de post.
  let videos = $state<Record<string, string>>({})
  $effect(() => {
    const d = detail
    const urls: string[] = []
    let alive = true
    videos = {}
    for (const f of media.filter((f) => f.path.endsWith('.mp4'))) {
      if (!d) break
      api
        .videoUrl(d.post.id, f.path)
        .then((u) => {
          urls.push(u)
          if (alive) videos = { ...videos, [f.path]: u }
          else URL.revokeObjectURL(u)
        })
        .catch((e) => (message = String(e)))
    }
    return () => {
      alive = false
      urls.forEach((u) => URL.revokeObjectURL(u))
    }
  })

  async function save(url: string) {
    saving = [...saving, url]
    message = ''
    try {
      const row = await api.savePost(url)
      selectedId = row.id
      message = row.status === 'ok' ? `Guardado @${row.handle}` : `Guardado con avisos: @${row.handle}`
    } catch (e) {
      message = `No se pudo guardar: ${e}`
    } finally {
      saving = saving.filter((u) => u !== url)
      await refresh()
    }
  }

  async function retry(id: string) {
    message = 'Reintentando…'
    try {
      const row = await api.retryPost(id)
      message = row.status === 'ok' ? 'Reintento correcto' : 'Sigue incompleto'
    } catch (e) {
      message = `Sigue fallando: ${e}`
    }
    await refresh()
    if (selectedId === id) detail = await api.getPost(id)
  }

  function onInput(e: Event) {
    // un enlace pegado se guarda al momento; uno tecleado espera a Enter; lo demás busca
    if (!api.looksLikePostUrl(input)) {
      filter.q = input.trim() || undefined
    } else if ((e as InputEvent).inputType === 'insertFromPaste') {
      save(input.trim())
      input = ''
    }
  }

  function onSubmit(e: SubmitEvent) {
    e.preventDefault()
    if (api.looksLikePostUrl(input)) {
      save(input.trim())
      input = ''
    }
  }

  function setFilter(patch: Filter) {
    filter = { q: filter.q, ...patch }
  }

  function move(step: number) {
    if (!posts.length) return
    const i = posts.findIndex((p) => p.id === selectedId)
    const next = posts[Math.max(0, Math.min(posts.length - 1, i + step))]
    selectedId = next.id
    rows[next.id]?.focus()
  }

  const typing = (t: EventTarget | null) => t instanceof HTMLInputElement || t instanceof HTMLTextAreaElement || t instanceof HTMLSelectElement

  function onKey(e: KeyboardEvent) {
    if (typing(e.target)) return
    // Ctrl+V fuera de un campo: WebKitGTK no dispara `paste` sin un campo editable enfocado,
    // así que se enfoca el campo ANTES de que el navegador pegue y el texto cae ahí (ver onInput).
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'v') {
      field.focus()
      field.select()
      return
    }
    if (e.ctrlKey || e.metaKey || e.altKey) return
    const actions: Record<string, () => void> = {
      j: () => move(1),
      ArrowDown: () => move(1),
      k: () => move(-1),
      ArrowUp: () => move(-1),
      '/': () => field.focus(),
      o: () => selectedId && api.openFolder(selectedId).catch((err) => (message = String(err))),
      r: () => selectedId && detail?.post.status !== 'ok' && retry(selectedId),
    }
    const action = actions[e.key]
    if (action) {
      e.preventDefault()
      action()
    }
  }

  const date = (iso: string | null, long = false) =>
    iso
      ? new Intl.DateTimeFormat('es', long ? { dateStyle: 'long', timeStyle: 'short' } : { day: 'numeric', month: 'short', year: 'numeric' }).format(new Date(iso))
      : 'Sin fecha'
  const size = (n: number) => (n > 1048576 ? `${(n / 1048576).toFixed(1)} MB` : `${Math.ceil(n / 1024)} KB`)
  const mediaLabel = (p: PostRow) =>
    [...p.media_kinds.map((k) => ({ photo: 'Fotos', video: 'Vídeo', gif: 'GIF' })[k]), p.has_attachments ? 'Documento' : ''].filter(Boolean).join(', ') ||
    (p.status === 'error' ? '' : 'Texto')
</script>

<svelte:window onkeydown={onKey} />

<div class="grid h-screen grid-rows-[auto_minmax(0,1fr)_auto] bg-slate-200 text-slate-900">
  <form class="flex gap-2 border-b border-slate-300 bg-slate-50 p-3" onsubmit={onSubmit}>
    <input
      bind:this={field}
      bind:value={input}
      oninput={onInput}
      class="flex-1 rounded border border-slate-300 bg-white px-3 py-2"
      placeholder="Pega un enlace de X para guardarlo, o escribe para buscar"
      aria-label="Enlace o búsqueda"
    />
    <button class="rounded bg-blue-800 px-4 py-2 font-bold text-white disabled:opacity-50" disabled={!api.looksLikePostUrl(input)}>
      Guardar post
    </button>
  </form>

  <div class="grid min-h-0 grid-cols-[13rem_minmax(0,22rem)_minmax(0,1fr)]">
    <nav class="overflow-auto border-r border-slate-300 p-3 text-sm" aria-label="Filtros">
      <button class="nav" aria-pressed={!filter.status && !filter.kind && !filter.handle} onclick={() => setFilter({})}>
        Todos <span>{total}</span>
      </button>
      <button class="nav" aria-pressed={filter.status === 'partial'} onclick={() => setFilter({ status: 'partial' })}>
        Parciales <span>{counts.partial}</span>
      </button>
      <button class="nav" aria-pressed={filter.status === 'error'} onclick={() => setFilter({ status: 'error' })}>
        Con error <span>{counts.error}</span>
      </button>
      <h2 class="mt-4 mb-1 font-bold text-slate-600">Por tipo</h2>
      {#each KINDS as [kind, label]}
        <button class="nav" aria-pressed={filter.kind === kind} onclick={() => setFilter({ kind })}>{label}</button>
      {/each}
      <h2 class="mt-4 mb-1 font-bold text-slate-600">Autores</h2>
      {#each counts.authors as a}
        <button class="nav" aria-pressed={filter.handle === a.handle} onclick={() => setFilter({ handle: a.handle })}>
          @{a.handle} <span>{a.count}</span>
        </button>
      {/each}
      <h2 class="mt-4 mb-1 font-bold text-slate-600">Fecha del post</h2>
      <label class="block">Desde <input type="date" class="w-full" bind:value={filter.from} /></label>
      <label class="block">Hasta <input type="date" class="w-full" bind:value={filter.to} /></label>
    </nav>

    <section class="flex min-h-0 flex-col border-r border-slate-300 bg-slate-50" aria-label="Posts">
      {#each saving as url}
        <p class="border-b border-slate-300 bg-blue-50 p-3 text-sm" role="status">Guardando {url.replace(/^https?:\/\//, '')}…</p>
      {/each}
      <div class="min-h-0 flex-1 overflow-auto" role="listbox" aria-label="Posts guardados">
        {#each posts as p (p.id)}
          <div
            bind:this={rows[p.id]}
            class="cursor-pointer border-b border-l-4 border-b-slate-200 p-3"
            class:border-l-blue-800={p.id === selectedId}
            class:bg-white={p.id === selectedId}
            class:border-l-transparent={p.id !== selectedId}
            role="option"
            tabindex={p.id === selectedId || (!selectedId && p === posts[0]) ? 0 : -1}
            aria-selected={p.id === selectedId}
            onclick={() => (selectedId = p.id)}
            onkeydown={(e) => e.key === 'Enter' && (selectedId = p.id)}
          >
            <div class="flex justify-between text-sm"><b>{p.handle ? `@${p.handle}` : p.url}</b><time>{date(p.created_at)}</time></div>
            <p class="line-clamp-2 text-sm">{p.text || p.error}</p>
            <div class="mt-1 flex justify-between text-xs text-slate-600">
              <span>{mediaLabel(p)}</span><span class="st-{p.status}">{STATUS[p.status]}</span>
            </div>
          </div>
        {:else}
          <p class="p-4 text-sm text-slate-600">
            {filter.q || filter.kind || filter.handle || filter.status ? 'Nada coincide con estos filtros.' : 'Copia el enlace de un post de X y pulsa Ctrl+V aquí para guardarlo.'}
          </p>
        {/each}
      </div>
    </section>

    <article class="min-h-0 overflow-auto bg-white p-6" aria-label="Post abierto">
      {#if detail}
        {@const p = detail.post}
        <header class="mb-4 flex flex-wrap items-start gap-3">
          <div class="flex-1">
            <b>{p.author_name || 'Autor desconocido'}</b>
            <div class="text-sm text-slate-600">{p.handle ? `@${p.handle}, ` : ''}{date(p.created_at, true)}</div>
          </div>
          {#if p.dir}<button class="btn" onclick={() => api.openFolder(p.id).catch((e) => (message = String(e)))}>Abrir carpeta</button>{/if}
          <button class="btn" onclick={() => api.openOnX(p.id).catch((e) => (message = String(e)))}>Ver en X</button>
          {#if p.status !== 'ok'}<button class="btn" onclick={() => retry(p.id)}>Reintentar</button>{/if}
        </header>
        {#if p.error}<p class="mb-4 rounded bg-red-50 p-3 text-sm text-red-800">{p.error}</p>{/if}
        <p class="max-w-prose whitespace-pre-wrap">{p.text}</p>
        {#if media.length && p.dir}
          <div class="mt-4 grid max-w-3xl grid-cols-2 gap-2">
            {#each media as f, i}
              {#if f.path.endsWith('.mp4')}
                <!-- X no da subtítulos de sus vídeos: una <track> vacía solo engañaría al lector de pantalla -->
                <!-- svelte-ignore a11y_media_has_caption -->
                {#if videos[f.path]}
                  <video class="w-full rounded" controls preload="metadata" src={videos[f.path]}></video>
                {:else}
                  <p class="rounded bg-slate-100 p-4 text-sm">Cargando vídeo…</p>
                {/if}
              {:else}
                <img class="w-full rounded" loading="lazy" alt="Foto {i + 1} de {media.length}" src={api.fileUrl(p.dir, f.path)} />
              {/if}
            {/each}
          </div>
        {/if}
        {#if p.quoted_handle}
          <blockquote class="mt-4 max-w-prose border-l-4 border-slate-300 pl-3">
            <b>Cita a @{p.quoted_handle}</b>
            <p class="whitespace-pre-wrap">{p.quoted_text}</p>
          </blockquote>
        {/if}
        {#if detail.files.length}
          <table class="mt-6 text-sm">
            <caption class="text-left font-bold">Archivos en disco</caption>
            <thead><tr><th class="pr-6 text-left">Archivo</th><th class="pr-6 text-left">Estado</th><th class="text-right">Tamaño</th></tr></thead>
            <tbody>
              {#each detail.files as f}
                <tr>
                  <td class="pr-6">{f.path}</td>
                  <td class="pr-6">{f.status === 'ok' ? '✓ Descargado' : f.status === 'skipped' ? `Omitido: ${f.error}` : `✕ ${f.error}`}</td>
                  <td class="text-right">{f.status === 'ok' ? size(f.size) : ''}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}
        <p class="mt-2 text-xs text-slate-600">Fuente: {p.source || 'ninguna'}</p>
      {:else}
        <p class="text-slate-600">Elige un post de la lista.</p>
      {/if}
    </article>
  </div>

  <footer class="flex flex-wrap gap-4 border-t border-slate-300 bg-slate-50 px-3 py-2 text-xs text-slate-700" aria-label="Atajos de teclado">
    <span><kbd>J</kbd> <kbd>K</kbd> siguiente y anterior</span>
    <span><kbd>/</kbd> buscar</span>
    <span><kbd>Ctrl</kbd> <kbd>V</kbd> guardar enlace</span>
    <span><kbd>O</kbd> abrir carpeta</span>
    <span><kbd>R</kbd> reintentar</span>
    <span class="ml-auto" role="status" aria-live="polite">{message}</span>
  </footer>
</div>

<style>
  .nav {
    display: flex;
    width: 100%;
    justify-content: space-between;
    border-radius: 0.25rem;
    padding: 0.25rem 0.5rem;
    text-align: left;
  }
  .nav[aria-pressed='true'] {
    background: white;
    font-weight: bold;
  }
  .btn {
    border: 1px solid #c9d1da;
    border-radius: 0.25rem;
    padding: 0.25rem 0.75rem;
    font-size: 0.875rem;
  }
  .st-ok { color: #1d7349; }
  .st-partial { color: #9a5f00; }
  .st-error { color: #b3261e; }
  :focus-visible { outline: 2px solid #2446c7; outline-offset: 2px; }
  kbd { border: 1px solid #c9d1da; border-radius: 0.2rem; padding: 0 0.25rem; background: white; }
</style>
