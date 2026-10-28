// Comandos de Rust con sus tipos. El frontend no toca red ni disco: todo pasa por aquí.
import { invoke, convertFileSrc } from '@tauri-apps/api/core'

export type Status = 'ok' | 'partial' | 'error'
export type Kind = 'photo' | 'video' | 'gif' | 'document' | 'text'

export interface PostRow {
  id: string
  url: string
  handle: string
  author_name: string
  created_at: string | null
  saved_at: string
  text: string
  source: string
  status: Status
  error: string | null
  dir: string | null
  media_kinds: Exclude<Kind, 'document' | 'text'>[]
  has_attachments: boolean
  quoted_handle: string | null
  quoted_text: string | null
  collection_id: number | null
  favorite: boolean
  note: string
}

export interface FileRow {
  path: string
  url: string
  status: 'ok' | 'error' | 'skipped'
  size: number
  sha256: string
  error: string | null
  alt: string | null
}

/** Misma paleta que organize::COLORS; cada nombre es un token --tag-<color> de app.css. */
export const TAG_COLORS = ['red', 'orange', 'yellow', 'green', 'teal', 'blue', 'purple', 'pink'] as const

export interface Tag {
  id: number
  name: string
  /** nombre de la paleta o `#rrggbb` propio (Rust valida que sea uno de los dos) */
  color: string | null
}

/** Valor CSS de un color de etiqueta: los de la paleta cambian con el tema; un HEX propio, no. */
export const tagColor = (c: string | null) => (!c ? 'transparent' : c.startsWith('#') ? c : `var(--tag-${c})`)

export interface Collection {
  id: number
  name: string
}

export interface Detail {
  post: PostRow
  files: FileRow[]
  tags: Tag[]
}

export interface Filter {
  q?: string
  handle?: string
  kind?: Kind
  status?: Status
  from?: string
  to?: string
  collection_id?: number
  tag_id?: number
  favorite?: boolean
}

export interface Facets {
  authors: { handle: string; name: string; count: number }[]
  ok: number
  partial: number
  error: number
  favorites: number
  collections: (Collection & { count: number })[]
  tags: (Tag & { count: number })[]
}

/** `lang`: idioma de la app, para los títulos de post.md */
export const savePost = (url: string, lang: string) => invoke<PostRow>('save_post', { url, lang })
export const retryPost = (id: string, lang: string) => invoke<PostRow>('retry_post', { id, lang })
export const listPosts = (filter: Filter) => invoke<PostRow[]>('list_posts', { filter })
export const getPost = (id: string) => invoke<Detail | null>('get_post', { id })
export const facets = () => invoke<Facets>('facets')
export const openFolder = (id: string) => invoke<void>('open_folder', { id })

/** URL que el WebView puede cargar para un archivo de la bóveda (protocolo asset:). */
export const fileUrl = (dir: string, path: string) => convertFileSrc(`${dir}/${path}`)

/** Misma regla que x_url.rs, solo para decidir si el campo guarda o busca. Rust valida de verdad. */
export const looksLikePostUrl = (s: string) =>
  /^https?:\/\/(www\.|mobile\.)?(x|twitter)\.com\/.+\/status(es)?\/\d+/i.test(s.trim())
export const openOnX = (id: string) => invoke<void>('open_on_x', { id })

/** Vídeo como URL blob: (WebKitGTK no reproduce vídeos por el protocolo asset:). Libérala con URL.revokeObjectURL. */
export async function videoUrl(id: string, path: string) {
  const bytes = await invoke<ArrayBuffer>('read_media', { id, path })
  return URL.createObjectURL(new Blob([bytes], { type: 'video/mp4' }))
}

// ---- organizar (los nombres de los argumentos van en camelCase: así los espera Tauri) ----
export const setFavorite = (id: string, value: boolean) => invoke<void>('set_favorite', { id, value })
export const setNote = (id: string, note: string) => invoke<void>('set_note', { id, note })
export const setCollection = (id: string, collectionId: number | null) => invoke<void>('set_collection', { id, collectionId })
/** Crea la etiqueta si no existe (sin distinguir mayúsculas). */
export const addTag = (id: string, name: string) => invoke<Tag>('add_tag', { id, name })
export const removeTag = (id: string, tagId: number) => invoke<void>('remove_tag', { id, tagId })
export const createCollection = (name: string) => invoke<Collection>('create_collection', { name })
export const renameCollection = (id: number, name: string) => invoke<void>('rename_collection', { id, name })
export const deleteCollection = (id: number) => invoke<void>('delete_collection', { id })
export const moveCollection = (id: number, position: number) => invoke<void>('move_collection', { id, position })
export const renameTag = (id: number, name: string) => invoke<void>('rename_tag', { id, name })
export const setTagColor = (id: number, color: string | null) => invoke<void>('set_tag_color', { id, color })
export const deleteTag = (id: number) => invoke<void>('delete_tag', { id })
