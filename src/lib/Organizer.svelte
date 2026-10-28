<script lang="ts">
  // Lo que la persona añade a un post: favorito, colección, etiquetas y nota.
  // Cada cambio se guarda al momento (la nota, al salir del campo) y luego avisa con `onchange`
  // para que App recargue lista y recuentos. Los atajos F, T, M y N llaman a las funciones exportadas.
  import * as api from './api'
  import { tagColor, type Collection, type Detail, type Tag } from './api'
  import { errorText, t } from './i18n.svelte'

  let { detail, collections, allTags, onchange, onerror }: {
    detail: Detail
    collections: Collection[]
    allTags: Tag[]
    onchange: () => void
    onerror: (msg: string) => void
  } = $props()

  const post = $derived(detail.post)
  let note = $state('')
  let tagInput = $state('')
  let tagField: HTMLInputElement
  let collectionField: HTMLSelectElement
  let noteField: HTMLTextAreaElement

  // al abrir otro post, la nota del campo pasa a ser la de ese post
  $effect(() => {
    note = post.note
  })

  // sugerencias: etiquetas que existen y este post aún no tiene
  const suggestions = $derived(allTags.filter((tag) => !detail.tags.some((mine) => mine.id === tag.id)))

  async function run(action: Promise<unknown>) {
    try {
      await action
      onchange()
    } catch (e) {
      onerror(errorText(e))
    }
  }

  export const toggleFavorite = () => run(api.setFavorite(post.id, !post.favorite))
  export const focusTags = () => tagField.focus()
  export const focusCollection = () => collectionField.focus()
  export const focusNote = () => noteField.focus()

  function onTagKey(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault()
      if (!tagInput.trim()) return
      run(api.addTag(post.id, tagInput))
      tagInput = ''
    } else if (e.key === 'Backspace' && !tagInput && detail.tags.length) {
      run(api.removeTag(post.id, detail.tags[detail.tags.length - 1].id))
    } else if (e.key === 'Escape') {
      tagField.blur()
    }
  }

  // WebKitGTK (Linux) no cambia un <select> cerrado con las flechas ni abre su lista con el teclado:
  // ↑/↓ se manejan aquí para que M + flechas funcione en todos los sistemas
  function onCollectionKey(e: KeyboardEvent) {
    if (e.key === 'Escape') return collectionField.blur()
    if (e.key !== 'ArrowDown' && e.key !== 'ArrowUp') return
    e.preventDefault()
    const ids = [null, ...collections.map((c) => c.id)]
    const i = ids.indexOf(post.collection_id) + (e.key === 'ArrowDown' ? 1 : -1)
    if (i >= 0 && i < ids.length) run(api.setCollection(post.id, ids[i]))
  }

  function saveNote() {
    if (note !== post.note) run(api.setNote(post.id, note))
  }
</script>

<section class="organize" aria-label={t('org.tags')}>
  <button
    class="star"
    aria-pressed={post.favorite}
    aria-label={post.favorite ? t('org.remove_favorite') : t('org.add_favorite')}
    title={post.favorite ? t('org.remove_favorite') : t('org.add_favorite')}
    onclick={toggleFavorite}
  >{post.favorite ? '★' : '☆'} {t('org.favorite')}</button>

  <label class="collection">
    <span>{t('org.collection')}</span>
    <select
      bind:this={collectionField}
      value={post.collection_id ?? ''}
      onkeydown={onCollectionKey}
      onchange={(e) => {
        const v = e.currentTarget.value
        run(api.setCollection(post.id, v ? Number(v) : null))
      }}
    >
      <option value="">{t('org.no_collection')}</option>
      {#each collections as c (c.id)}
        <option value={c.id}>{c.name}</option>
      {/each}
    </select>
  </label>

  <div class="tags">
    <span class="label" id="tags-label">{t('org.tags')}</span>
    <div class="chips">
      {#each detail.tags as tag (tag.id)}
        <span class="chip">
          <i class="tag-dot" style:background={tagColor(tag.color)} aria-hidden="true"></i>
          {tag.name}
          <button aria-label={t('org.remove_tag', { name: tag.name })} onclick={() => run(api.removeTag(post.id, tag.id))}>×</button>
        </span>
      {/each}
      <input
        bind:this={tagField}
        bind:value={tagInput}
        onkeydown={onTagKey}
        list="tag-suggestions"
        placeholder={t('org.tag_placeholder')}
        aria-label={t('org.tag_input')}
        aria-describedby="tags-label"
      />
      <datalist id="tag-suggestions">
        {#each suggestions as tag (tag.id)}<option value={tag.name}></option>{/each}
      </datalist>
    </div>
  </div>

  <label class="note">
    <span>{t('org.note')}</span>
    <textarea
      bind:this={noteField}
      bind:value={note}
      onblur={saveNote}
      onkeydown={(e) => e.key === 'Escape' && noteField.blur()}
      rows="2"
      placeholder={t('org.note_placeholder')}
    ></textarea>
  </label>
</section>

<style>
  .organize { display: grid; grid-template-columns: auto minmax(0, 1fr); gap: 10px 16px; align-items: center; max-width: 720px; padding: 12px 14px; margin-bottom: 18px; border: 1px solid var(--line); border-radius: 8px; background: var(--paper); }
  .label, .collection span, .note span { color: var(--muted); font-size: 14px; }
  .star { justify-self: start; border: 1px solid var(--line); background: var(--pane); border-radius: 6px; padding: 4px 10px; cursor: pointer; }
  .star[aria-pressed='true'] { color: var(--warn); border-color: var(--warn); font-weight: 700; }
  .collection { display: flex; gap: 8px; align-items: center; }
  select, textarea, .chips input { background: var(--pane); border: 1px solid var(--line); border-radius: 6px; padding: 4px 8px; font: inherit; color: inherit; }
  .tags, .note { grid-column: 1 / -1; display: grid; grid-template-columns: 70px minmax(0, 1fr); gap: 8px; align-items: start; }
  .tags .label, .note span { padding-top: 5px; }
  .chips { display: flex; flex-wrap: wrap; gap: 6px; align-items: center; }
  .chip { display: inline-flex; align-items: center; gap: 6px; border: 1px solid var(--line); border-radius: 999px; padding: 2px 4px 2px 10px; font-size: 14px; }
  .chip button { border: 0; background: none; cursor: pointer; color: var(--muted); padding: 0 6px; border-radius: 999px; font-size: 16px; line-height: 1; }
  .chip button:hover { color: var(--err); }
  .chips input { flex: 1; min-width: 14ch; }
  textarea { width: 100%; resize: vertical; }
</style>
