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

export interface Detail {
  post: PostRow
  files: FileRow[]
}

export interface Filter {
  q?: string
  handle?: string
  kind?: Kind
  status?: Status
  from?: string
  to?: string
}

export interface Facets {
  authors: { handle: string; name: string; count: number }[]
  ok: number
  partial: number
  error: number
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
