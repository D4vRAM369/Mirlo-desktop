<script lang="ts">
  // Bandeja: comando arriba, filtros a la izquierda, lista en el centro, lectura a la derecha.
  // Todo se maneja con teclado; los atajos están siempre a la vista en el pie.
  import * as api from './lib/api'
  import type { Detail, Facets, Filter, Kind, PostRow, Status } from './lib/api'
  import Backup from './lib/Backup.svelte'
  import Gallery from './lib/Gallery.svelte'
  import NameEditor from './lib/NameEditor.svelte'
  import Organizer from './lib/Organizer.svelte'
  import { errorParts, errorText, formatDate, formatSize, i18n, setLang, t, withKeys } from './lib/i18n.svelte'

  let input = $state('')
  let filter = $state<Filter>({})
  let posts = $state<PostRow[]>([])
  let counts = $state<Facets>({ authors: [], ok: 0, partial: 0, error: 0, favorites: 0, collections: [], tags: [] })
  let selectedId = $state<string | null>(null)
  let detail = $state<Detail | null>(null)
  let saving = $state<string[]>([]) // enlaces en cola de guardado
  let message = $state('')
  let theme = $state(readTheme())

  let field: HTMLInputElement
  let organizer = $state<ReturnType<typeof Organizer>>()
  // sidebar: qué colección o etiqueta se está editando, y el campo de "nueva colección"
  let editing = $state<{ kind: 'collection' | 'tag'; id: number } | null>(null)
  let newCollection = $state<string | null>(null)
  const rows: Record<string, HTMLElement> = {}

  const KINDS: Kind[] = ['video', 'photo', 'gif', 'document', 'text']
  const status = (s: Status) => t(`status.${s}`)
  const total = $derived(counts.ok + counts.partial + counts.error)
  const media = $derived(detail?.files.filter((f) => f.status === 'ok' && f.path.startsWith('media/')) ?? [])
  const title = $derived(
    filter.favorite ? t('side.favorites')
    : filter.collection_id != null ? (counts.collections.find((c) => c.id === filter.collection_id)?.name ?? '')
    : filter.tag_id != null ? `#${counts.tags.find((tag) => tag.id === filter.tag_id)?.name ?? ''}`
    : filter.status === 'partial' ? t('side.partial')
    : filter.status === 'error' ? t('side.error')
    : filter.kind ? t(`kind.${filter.kind}`)
    : filter.handle ? `@${filter.handle}`
    : t('side.all'),
  )

  // ---- tema: sigue al sistema hasta que eliges uno; se recuerda en este equipo ----
  function readTheme(): 'light' | 'dark' | null {
    try {
      const t = localStorage.getItem('theme')
      return t === 'light' || t === 'dark' ? t : null
    } catch {
      return null
    }
  }
  const isDark = $derived(theme ? theme === 'dark' : matchMedia('(prefers-color-scheme: dark)').matches)
  $effect(() => {
    if (theme) document.documentElement.dataset.theme = theme
  })
  function toggleTheme() {
    theme = isDark ? 'light' : 'dark'
    try {
      localStorage.setItem('theme', theme)
    } catch {}
  }

  // ---- datos ----
  async function refresh() {
    try {
      ;[posts, counts] = await Promise.all([api.listPosts(filter), api.facets()])
    } catch (e) {
      message = errorText(e)
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

  // ---- organizar ----
  async function reload() {
    await refresh()
    if (selectedId) detail = await api.getPost(selectedId)
  }

  /** Cambio de organización desde el sidebar: si va bien, cierra el editor y recarga. */
  async function organize(action: Promise<unknown>) {
    try {
      await action
      editing = null
      await reload()
    } catch (e) {
      message = errorText(e)
    }
  }

  // `autofocus` solo vale al cargar la página: un campo que aparece después se enfoca a mano
  const focusOnMount = (node: HTMLInputElement) => {
    node.focus()
    node.select()
  }

  async function addCollection(e: KeyboardEvent) {
    if (e.key === 'Escape') return void (newCollection = null)
    if (e.key !== 'Enter') return
    e.preventDefault()
    try {
      const c = await api.createCollection(newCollection ?? '')
      newCollection = null
      await refresh()
      setFilter({ collection_id: c.id })
    } catch (err) {
      message = errorText(err)
    }
  }

  async function removeCollection(id: number) {
    if (filter.collection_id === id) setFilter({})
    await organize(api.deleteCollection(id))
  }

  async function removeTag(id: number) {
    if (filter.tag_id === id) setFilter({})
    await organize(api.deleteTag(id))
  }

  // Alt+↑/↓ mueve la colección enfocada; F2 la edita (igual en etiquetas)
  function onSideKey(e: KeyboardEvent, kind: 'collection' | 'tag', id: number) {
    if (e.key === 'F2') {
      e.preventDefault()
      editing = { kind, id }
    } else if (kind === 'collection' && e.altKey && (e.key === 'ArrowUp' || e.key === 'ArrowDown')) {
      e.preventDefault()
      const i = counts.collections.findIndex((c) => c.id === id)
      const to = i + (e.key === 'ArrowUp' ? -1 : 1)
      if (to < 0 || to >= counts.collections.length) return
      organize(api.moveCollection(id, to)).then(() => document.getElementById(`collection-${id}`)?.focus())
    }
  }

  async function save(url: string) {
    saving = [...saving, url]
    message = ''
    try {
      const row = await api.savePost(url, i18n.lang)
      selectedId = row.id
      message = t(row.status === 'ok' ? 'msg.saved' : 'msg.saved_partial', { user: row.handle })
    } catch (e) {
      message = t('msg.save_failed', { error: errorParts(e).text })
    } finally {
      saving = saving.filter((u) => u !== url)
      await refresh()
    }
  }

  async function retry(id: string) {
    message = t('msg.retrying')
    try {
      const row = await api.retryPost(id, i18n.lang)
      message = t(row.status === 'ok' ? 'msg.retry_ok' : 'msg.retry_partial')
    } catch (e) {
      message = t('msg.retry_failed', { error: errorParts(e).text })
    }
    await refresh()
    if (selectedId === id) detail = await api.getPost(id)
  }

  // ---- entrada: un enlace pegado se guarda al momento; uno tecleado espera a Enter; lo demás busca ----
  function onInput(e: Event) {
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
    filter = { q: filter.q, from: filter.from, to: filter.to, ...patch }
  }

  function move(step: number) {
    if (!posts.length) return
    const i = posts.findIndex((p) => p.id === selectedId)
    const next = posts[Math.max(0, Math.min(posts.length - 1, i + step))]
    selectedId = next.id
    rows[next.id]?.focus()
    rows[next.id]?.scrollIntoView({ block: 'nearest' })
  }

  const typing = (t: EventTarget | null) => t instanceof HTMLInputElement || t instanceof HTMLTextAreaElement
  const inDialog = () => document.querySelector('dialog[open]') !== null

  function onKey(e: KeyboardEvent) {
    if (typing(e.target) || inDialog()) {
      if (e.key === 'Escape' && e.target === field) field.blur()
      return
    }
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
      o: () => selectedId && api.openFolder(selectedId).catch((err) => (message = errorText(err))),
      r: () => selectedId && detail?.post.status !== 'ok' && retry(selectedId),
      f: () => organizer?.toggleFavorite(),
      t: () => organizer?.focusTags(),
      m: () => organizer?.focusCollection(),
      n: () => organizer?.focusNote(),
    }
    const action = actions[e.key]
    if (action) {
      e.preventDefault()
      action()
    }
  }

  // ---- formato ----
  const mediaLabel = (p: PostRow) =>
    [...p.media_kinds.map((k) => t(`media.${k}`)), p.has_attachments ? t('media.document') : ''].filter(Boolean).join(', ') ||
    (p.status === 'error' ? '' : t('media.text'))
  // color estable por autor: el mismo @usuario siempre tiene el mismo círculo
  const hue = (handle: string) => [...handle].reduce((a, c) => (a * 31 + c.charCodeAt(0)) % 360, 7)
  const queueLabel = (url: string) => {
    const user = url.match(/\.com\/([^/]+)\/status/i)?.[1]
    return user && user !== 'i' ? t('list.saving_user', { user }) : t('list.saving')
  }
  const fileState = (f: Detail['files'][number]) =>
    f.status === 'ok' ? t('read.downloaded') : f.status === 'skipped' ? t('read.skipped', { why: errorText(f.error) }) : `✕ ${errorText(f.error)}`
</script>

<svelte:window onkeydown={onKey} />

{#snippet keyed(text: string)}
  {#each withKeys(text) as part}{#if part.key}<kbd>{part.text}</kbd>{:else}{part.text}{/if}{/each}
{/snippet}

<div class="app">
  <form class="cmd" onsubmit={onSubmit}>
    <label class="field">
      <input
        bind:this={field}
        bind:value={input}
        oninput={onInput}
        placeholder={t('cmd.placeholder')}
        aria-label={t('cmd.label')}
      />
      <kbd>/</kbd>
    </label>
    <button class="btn primary" disabled={!api.looksLikePostUrl(input)}>{t('cmd.save')}</button>
    <Backup onmessage={(m) => (message = m)} ondone={reload} />
    <button type="button" class="btn" onclick={toggleTheme}>{isDark ? t('cmd.theme_light') : t('cmd.theme_dark')}</button>
    <button type="button" class="btn" lang={i18n.lang === 'es' ? 'en' : 'es'} onclick={() => setLang(i18n.lang === 'es' ? 'en' : 'es')}>{t('cmd.other_lang')}</button>
  </form>

  <div class="panes">
    <nav class="side" aria-label={t('side.label')}>
      <button aria-pressed={!filter.status && !filter.kind && !filter.handle && !filter.favorite && filter.collection_id == null && filter.tag_id == null} onclick={() => setFilter({})}>
        {t('side.all')} <span>{total}</span>
      </button>
      <button aria-pressed={!!filter.favorite} onclick={() => setFilter({ favorite: true })}>
        ★ {t('side.favorites')} <span>{counts.favorites}</span>
      </button>
      <button aria-pressed={filter.status === 'partial'} onclick={() => setFilter({ status: 'partial' })}>
        {t('side.partial')} <span>{counts.partial}</span>
      </button>
      <button aria-pressed={filter.status === 'error'} onclick={() => setFilter({ status: 'error' })}>
        {t('side.error')} <span>{counts.error}</span>
      </button>
      <div class="side-head">
        <h2>{t('side.collections')}</h2>
        <button class="add" aria-label={t('side.new_collection')} title={t('side.new_collection')} onclick={() => (newCollection = '')}>+</button>
      </div>
      {#if newCollection !== null}
        <input class="new" use:focusOnMount bind:value={newCollection} onkeydown={addCollection} onblur={() => (newCollection = null)} placeholder={t('side.new_collection')} aria-label={t('side.new_collection')} />
      {/if}
      {#each counts.collections as c (c.id)}
        {#if editing?.kind === 'collection' && editing.id === c.id}
          <NameEditor
            name={c.name}
            deleteHint={t('edit.delete_collection_hint')}
            onrename={(name) => organize(api.renameCollection(c.id, name))}
            ondelete={() => removeCollection(c.id)}
            onclose={() => (editing = null)}
          />
        {:else}
          <button
            id="collection-{c.id}"
            aria-pressed={filter.collection_id === c.id}
            title={t('side.edit_hint').replace(/[[\]]/g, '')}
            onclick={() => setFilter({ collection_id: c.id })}
            ondblclick={() => (editing = { kind: 'collection', id: c.id })}
            onkeydown={(e) => onSideKey(e, 'collection', c.id)}
          >
            <span class="name">{c.name}</span> <span>{c.count}</span>
          </button>
        {/if}
      {/each}
      <h2>{t('side.tags')}</h2>
      {#if counts.tags.length}
        <div class="tag-list">
          {#each counts.tags as tag (tag.id)}
            {#if editing?.kind === 'tag' && editing.id === tag.id}
              <NameEditor
                name={tag.name}
                color={tag.color}
                deleteHint={t('edit.delete_tag_hint')}
                onrename={(name) => organize(api.renameTag(tag.id, name))}
                oncolor={(color) => organize(api.setTagColor(tag.id, color)).then(() => (editing = { kind: 'tag', id: tag.id }))}
                ondelete={() => removeTag(tag.id)}
                onclose={() => (editing = null)}
              />
            {:else}
              <button
                class="tag"
                aria-pressed={filter.tag_id === tag.id}
                title={t('side.edit_hint_tag').replace(/[[\]]/g, '')}
                onclick={() => setFilter({ tag_id: tag.id })}
                ondblclick={() => (editing = { kind: 'tag', id: tag.id })}
                onkeydown={(e) => onSideKey(e, 'tag', tag.id)}
              >
                <i class="tag-dot" style:background={api.tagColor(tag.color)} aria-hidden="true"></i>{tag.name} <span>{tag.count}</span>
              </button>
            {/if}
          {/each}
        </div>
      {:else}
        <p class="hint">{@render keyed(t('side.no_tags'))}</p>
      {/if}
      <h2>{t('side.by_type')}</h2>
      {#each KINDS as kind}
        <button aria-pressed={filter.kind === kind} onclick={() => setFilter({ kind })}>{t(`kind.${kind}`)}</button>
      {/each}
      {#if counts.authors.length}
        <h2>{t('side.authors')}</h2>
        {#each counts.authors as a}
          <button aria-pressed={filter.handle === a.handle} onclick={() => setFilter({ handle: a.handle })} title={a.name}>
            <span class="handle"><i class="dot" style="background: hsl({hue(a.handle)} 32% 46%)" aria-hidden="true"></i><span class="name">{a.handle}</span></span>
            <span>{a.count}</span>
          </button>
        {/each}
      {/if}
      <details open={!!(filter.from || filter.to)}>
        <summary>{t('side.date', { range: filter.from || filter.to ? t('side.date_range', { from: filter.from ?? '…', to: filter.to ?? '…' }) : t('side.date_any') })}</summary>
        <label>{t('side.from')} <input type="date" bind:value={filter.from} /></label>
        <label>{t('side.to')} <input type="date" bind:value={filter.to} /></label>
        {#if filter.from || filter.to}
          <button class="btn" onclick={() => (filter = { ...filter, from: undefined, to: undefined })}>{t('side.clear_dates')}</button>
        {/if}
      </details>
    </nav>

    <section class="list" aria-label={t('list.label')}>
      <div class="list-head">
        <h1>{title}</h1>
        <small>{filter.q ? t('list.searching', { q: filter.q }) : t('list.newest')}</small>
      </div>
      {#each saving as url}
        <div class="queue" role="status">{queueLabel(url)}<div class="bar"><i></i></div></div>
      {/each}
      <div role="listbox" aria-label={t('list.items_label')}>
        {#each posts as p (p.id)}
          <div
            bind:this={rows[p.id]}
            class="row"
            role="option"
            tabindex={p.id === selectedId || (!selectedId && p === posts[0]) ? 0 : -1}
            aria-selected={p.id === selectedId}
            onclick={() => (selectedId = p.id)}
            onkeydown={(e) => e.key === 'Enter' && (selectedId = p.id)}
          >
            <span class="who">{p.handle ? `@${p.handle}` : t('list.post_id', { id: p.id })}</span><time>{formatDate(p.created_at)}</time>
            <p>{p.text || errorParts(p.error).text}</p>
            <div class="meta">{#if p.favorite}<span class="fav" aria-label={t('org.favorite')}>★</span>{/if}<span>{mediaLabel(p)}</span><span class="st {p.status}">{status(p.status)}</span></div>
          </div>
        {:else}
          <p class="empty">
            {#if filter.collection_id != null && !filter.q}
              {@render keyed(t('list.empty_collection'))}
            {:else if filter.q || filter.kind || filter.handle || filter.status || filter.from || filter.to || filter.favorite || filter.tag_id != null}
              {t('list.no_match')}
            {:else}
              {@render keyed(t('list.empty'))}
            {/if}
          </p>
        {/each}
      </div>
    </section>

    <article class="read" aria-label={t('read.label')}>
      {#if detail}
        {@const p = detail.post}
        <header>
          <i class="dot big" style="background: hsl({hue(p.handle || p.id)} 32% 46%)" aria-hidden="true">{(p.author_name || '?')[0]}</i>
          <div>
            <b>{p.author_name || t('list.post_id', { id: p.id })}</b>
            <span>{p.handle ? `@${p.handle}, ${formatDate(p.created_at, true)}` : p.url}</span>
          </div>
          <div class="btns">
            {#if p.status !== 'ok'}<button class="btn primary" onclick={() => retry(p.id)}>{t('read.retry')}</button>{/if}
            {#if p.dir}<button class="btn" onclick={() => api.openFolder(p.id).catch((e) => (message = errorText(e)))}>{t('read.open_folder')}</button>{/if}
            <button class="btn" onclick={() => api.openOnX(p.id).catch((e) => (message = errorText(e)))}>{t('read.view_on_x')}</button>
          </div>
        </header>
        {#if p.status !== 'error' || p.dir}
          <Organizer bind:this={organizer} {detail} collections={counts.collections} allTags={counts.tags} onchange={reload} onerror={(m) => (message = m)} />
        {/if}
        {#if p.error}
          {@const err = errorParts(p.error)}
          <div class="alert" role="alert">
            <p><b class="st {p.status}">{status(p.status)}.</b> {err.text}</p>
            {#if err.detail}<p class="detail">{t('read.details', { detail: err.detail })}</p>{/if}
          </div>
        {/if}
        {#if p.text}<p class="body">{p.text}</p>{/if}
        {#if media.length && p.dir}
          <Gallery id={p.id} dir={p.dir} files={media} />
        {/if}
        {#if p.quoted_handle}
          <div class="quoted">
            <b>{t('read.quoting', { user: p.quoted_handle })}</b>
            <p>{p.quoted_text}</p>
          </div>
        {/if}
        {#if detail.files.length}
          <table>
            <caption>{t('read.files')}</caption>
            <thead><tr><th>{t('read.file')}</th><th>{t('read.state')}</th><th class="num">{t('read.size')}</th></tr></thead>
            <tbody>
              {#each detail.files as f}
                <tr>
                  <td>{f.path}</td>
                  <td class:st={f.status !== 'ok'} class:error={f.status === 'error'}>{fileState(f)}</td>
                  <td class="num">{f.status === 'ok' ? formatSize(f.size) : ''}</td>
                </tr>
              {/each}
              <tr><td>post.json</td><td>{t('read.source', { source: p.source || t('read.source_none') })}</td><td class="num"></td></tr>
            </tbody>
          </table>
        {/if}
      {:else}
        <div class="empty">
          <p>{@render keyed(t('read.pick'))}</p>
          <p>{@render keyed(t('read.how_to_save'))}</p>
        </div>
      {/if}
    </article>
  </div>

  <footer class="keys" aria-label={t('keys.label')}>
    <span><kbd>J</kbd> <kbd>K</kbd> {t('keys.nav')}</span>
    <span><kbd>/</kbd> {t('keys.search')}</span>
    <span><kbd>Ctrl</kbd> <kbd>V</kbd> {t('keys.save')}</span>
    <span><kbd>O</kbd> {t('keys.open')}</span>
    <span><kbd>R</kbd> {t('keys.retry')}</span>
    <span><kbd>F</kbd> {t('keys.favorite')}</span>
    <span><kbd>T</kbd> {t('keys.tags')}</span>
    <span><kbd>M</kbd> {t('keys.collection')}</span>
    <span><kbd>N</kbd> {t('keys.note')}</span>
    <span class="msg" role="status" aria-live="polite">{message}</span>
  </footer>
</div>

<style>
  .app { display: grid; grid-template-rows: auto minmax(0, 1fr) auto; height: 100vh; }

  .cmd { display: flex; gap: 10px; align-items: center; padding: 10px 14px; border-bottom: 1px solid var(--line); background: var(--pane); }
  .field { flex: 1; display: flex; align-items: center; gap: 8px; background: var(--paper); border: 1px solid var(--line); border-radius: 6px; padding: 7px 10px; }
  .field:focus-within { border-color: var(--accent); }
  .field input { flex: 1; border: 0; outline: none; background: transparent; min-width: 0; }

  .panes { display: grid; grid-template-columns: 220px 400px minmax(0, 1fr); min-height: 0; }
  .side, .list { border-right: 1px solid var(--line); overflow-y: auto; }

  .side { background: var(--pane); padding: 12px 8px; }
  .side h2 { font-size: 13px; color: var(--muted); font-weight: 500; margin: 14px 8px 4px; }
  .side > button { display: flex; width: 100%; justify-content: space-between; gap: 8px; padding: 5px 8px; border: 0; border-radius: 6px; background: none; text-align: left; cursor: pointer; }
  .side > button:hover { background: var(--paper); }
  .side > button[aria-pressed='true'] { background: var(--accent-soft); color: var(--accent); font-weight: 700; }
  .side > button > span:last-child { color: var(--muted); font-variant-numeric: tabular-nums; }
  .side-head { display: flex; justify-content: space-between; align-items: center; margin-right: 4px; }
  .side-head .add { border: 1px solid var(--line); background: var(--paper); border-radius: 6px; width: 24px; height: 24px; line-height: 1; cursor: pointer; margin-top: 10px; }
  .side .new { display: block; width: 100%; margin: 2px 0 6px; background: var(--paper); border: 1px solid var(--accent); border-radius: 6px; padding: 4px 8px; }
  .side .name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .tag-list { display: flex; flex-wrap: wrap; gap: 5px; padding: 0 4px; }
  .tag-list :global(.editor) { flex-basis: 100%; }
  .side { min-width: 0; }
  .tag { display: inline-flex; align-items: center; gap: 5px; border: 1px solid var(--line); background: var(--paper); border-radius: 999px; padding: 2px 9px; font-size: 14px; cursor: pointer; }
  .tag span { color: var(--muted); font-variant-numeric: tabular-nums; }
  .tag[aria-pressed='true'] { border-color: var(--accent); background: var(--accent-soft); color: var(--accent); font-weight: 700; }
  .hint { color: var(--muted); font-size: 13px; margin: 0 8px; }
  .fav { color: var(--warn); }
  .handle { display: flex; gap: 8px; align-items: center; min-width: 0; }
  .handle .name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .side details { margin: 14px 8px 0; font-size: 14px; }
  .side summary { color: var(--muted); cursor: pointer; }
  .side details label { display: block; margin-top: 6px; }
  .side details input { display: block; width: 100%; margin-top: 2px; background: var(--paper); border: 1px solid var(--line); border-radius: 6px; padding: 4px 6px; }
  .side details .btn { margin-top: 8px; padding: 4px 10px; }

  .list { background: var(--paper); }
  .list-head { display: flex; justify-content: space-between; align-items: baseline; gap: 8px; padding: 10px 14px; border-bottom: 1px solid var(--line); position: sticky; top: 0; background: var(--paper); z-index: 1; }
  .list-head h1 { margin: 0; font-size: 17px; }
  .list-head small { color: var(--muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .queue { padding: 10px 14px; border-bottom: 1px solid var(--line); background: var(--accent-soft); font-size: 14px; }
  .queue .bar { height: 4px; background: var(--line); border-radius: 2px; margin-top: 6px; overflow: hidden; }
  /* no hay progreso real que mostrar: una barra que va y viene dice "trabajando" sin inventarse un porcentaje */
  .queue .bar i { display: block; height: 100%; width: 30%; background: var(--accent); animation: slide 1.2s ease-in-out infinite alternate; }
  @keyframes slide { from { transform: translateX(0); } to { transform: translateX(233%); } }

  .row { display: grid; grid-template-columns: minmax(0, 1fr) auto; gap: 2px 10px; padding: 11px 14px 11px 12px; border-bottom: 1px solid var(--line); border-left: 4px solid transparent; cursor: pointer; }
  .row:hover { background: var(--pane); }
  .row[aria-selected='true'] { background: var(--accent-soft); border-left-color: var(--accent); }
  .row .who { font-weight: 700; overflow: hidden; text-overflow: ellipsis; }
  .row time { color: var(--muted); font-size: 13px; font-variant-numeric: tabular-nums; }
  .row p { grid-column: 1 / -1; margin: 0; color: var(--muted); display: -webkit-box; -webkit-line-clamp: 2; line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; overflow-wrap: anywhere; }
  .row .meta { grid-column: 1 / -1; display: flex; gap: 12px; font-size: 13px; color: var(--muted); margin-top: 3px; }
  .empty { padding: 18px 14px; color: var(--muted); max-width: 46ch; }

  .read { overflow-y: auto; padding: 22px 32px 40px; background: var(--pane); }
  .read header { display: flex; gap: 12px; align-items: center; margin-bottom: 14px; flex-wrap: wrap; }
  .dot.big { width: 44px; height: 44px; font-size: 18px; }
  .read header b { font-size: 18px; display: block; }
  .read header span { color: var(--muted); }
  .read header .btns { margin-left: auto; display: flex; gap: 8px; }
  .alert { max-width: 62ch; background: var(--err-soft); border-radius: 6px; padding: 10px 12px; margin-bottom: 16px; }
  .alert p { margin: 0; }
  .alert .detail { margin-top: 6px; font-size: 13px; color: var(--muted); overflow-wrap: anywhere; }
  .body { font-size: 18px; line-height: 1.6; max-width: 62ch; margin: 0 0 18px; white-space: pre-wrap; overflow-wrap: anywhere; }
  .quoted { border: 1px solid var(--line); border-radius: 8px; padding: 12px 14px; max-width: 620px; background: var(--paper); margin-bottom: 22px; }
  .quoted b { font-size: 14px; }
  .quoted p { margin: 4px 0 0; color: var(--muted); white-space: pre-wrap; overflow-wrap: anywhere; }
  table { border-collapse: collapse; width: 100%; max-width: 720px; font-size: 14px; }
  caption { text-align: left; font-weight: 700; padding-bottom: 6px; }
  td, th { text-align: left; padding: 7px 8px; border-top: 1px solid var(--line); font-weight: 400; overflow-wrap: anywhere; }
  th { color: var(--muted); font-weight: 500; }
  .num { text-align: right; font-variant-numeric: tabular-nums; color: var(--muted); white-space: nowrap; }

  .keys { display: flex; gap: 18px; padding: 6px 14px; border-top: 1px solid var(--line); background: var(--pane); font-size: 13px; color: var(--muted); flex-wrap: wrap; }
  .msg { margin-left: auto; color: var(--ink); }

  @media (max-width: 1100px) { .panes { grid-template-columns: 180px 320px minmax(0, 1fr); } .read { padding: 18px 20px 32px; } }
</style>
