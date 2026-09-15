import { invoke } from '@tauri-apps/api/core'

export interface InspirationCategory {
  id: string
  name: string
  color?: string | null
  sortOrder: number
  inspirationCount: number
}

export interface InspirationTag {
  id: string
  name: string
}

export interface InspirationSummary {
  id: string
  title: string
  bodyText: string
  categoryId?: string | null
  categoryName?: string | null
  tagIds: string[]
  tagNames: string[]
  createdAt: string
  updatedAt: string
}

export interface InspirationDetail extends InspirationSummary {
  bodyHtml: string
}

export interface ListInspirationsQuery {
  categoryId?: string | null
  tagIds?: string[]
  keyword?: string
}

export interface UpsertInspirationPayload {
  title?: string
  bodyHtml: string
  bodyText: string
  categoryId?: string | null
  tagIds?: string[]
  tagNames?: string[]
}

export const UNCLASSIFIED_CATEGORY_ID = '1930000000000000001'

export function listCategories() {
  return invoke<InspirationCategory[]>('list_inspiration_categories')
}

export function createCategory(name: string, color?: string) {
  return invoke<InspirationCategory>('create_inspiration_category', {
    req: { name, color },
  })
}

export function updateCategory(id: string, name: string, color?: string) {
  return invoke<void>('update_inspiration_category', {
    id,
    req: { name, color },
  })
}

export function deleteCategory(id: string) {
  return invoke<void>('delete_inspiration_category', { id })
}

export function listTags() {
  return invoke<InspirationTag[]>('list_inspiration_tags')
}

export function createTag(name: string) {
  return invoke<InspirationTag>('create_inspiration_tag', { req: { name } })
}

export function updateTag(id: string, name: string) {
  return invoke<void>('update_inspiration_tag', { id, req: { name } })
}

export function deleteTag(id: string) {
  return invoke<void>('delete_inspiration_tag', { id })
}

export function listInspirations(query: ListInspirationsQuery = {}) {
  return invoke<InspirationSummary[]>('list_inspirations', { query })
}

export function getInspiration(id: string) {
  return invoke<InspirationDetail>('get_inspiration', { id })
}

export function createInspiration(req: UpsertInspirationPayload) {
  return invoke<InspirationDetail>('create_inspiration', { req })
}

export function updateInspiration(id: string, req: UpsertInspirationPayload) {
  return invoke<InspirationDetail>('update_inspiration', { id, req })
}

export function deleteInspiration(id: string) {
  return invoke<void>('delete_inspiration', { id })
}
