import { cloneJsonRecord } from './jsonValue'
import type { KnowledgeEntryDefinition } from './knowledgeContent'

export interface KnowledgeEntryEditForm {
  id: string
  title: string
  category: string
  content: string
  tagsText: string
  relatedText: string
  importance: number
  metadata: Record<string, unknown>
}

export interface KnowledgeEntryFilter {
  query: string
  category: string | null
  tag: string | null
}

export interface KnowledgeTaxonomy {
  categories: string[]
  tags: string[]
}

export interface KnowledgeEditingContext {
  original_entry_id: string | null
  existing_entry_ids: readonly string[]
}

export type KnowledgeEditingIssueCode =
  | 'invalid_id'
  | 'title_required'
  | 'content_required'
  | 'category_required'
  | 'category_format'
  | 'duplicate_id'
  | 'related_id'
  | 'related_self'
  | 'related_missing'

export interface KnowledgeEditingIssue {
  code: KnowledgeEditingIssueCode
  target_id?: string
}

const KNOWLEDGE_ID = /^[a-z0-9_-]{1,128}$/
const KNOWLEDGE_CATEGORY = /^[a-z0-9_-]{1,64}$/

export function emptyKnowledgeEntryEditForm(): KnowledgeEntryEditForm {
  return {
    id: '',
    title: '',
    category: 'world_lore',
    content: '',
    tagsText: '',
    relatedText: '',
    importance: 0.5,
    metadata: {},
  }
}

export function createKnowledgeEntryEditForm(
  existingIds: readonly string[],
): KnowledgeEntryEditForm {
  return {
    ...emptyKnowledgeEntryEditForm(),
    id: nextKnowledgeEntryId(existingIds),
  }
}

export function knowledgeEntryEditFormFromDefinition(
  entry: KnowledgeEntryDefinition,
): KnowledgeEntryEditForm {
  return {
    id: entry.id,
    title: entry.title,
    category: entry.category,
    content: entry.content,
    tagsText: entry.tags.join(', '),
    relatedText: entry.related_entries.join(', '),
    importance: entry.importance,
    metadata: cloneJsonRecord(entry.metadata),
  }
}

export function knowledgeEntryDefinitionFromEditForm(
  form: KnowledgeEntryEditForm,
): KnowledgeEntryDefinition {
  const importance = Number(form.importance)
  return {
    id: form.id.trim(),
    title: form.title.trim(),
    category: form.category.trim().toLowerCase(),
    content: form.content.trim(),
    tags: parseKnowledgeCommaList(form.tagsText),
    related_entries: parseKnowledgeCommaList(form.relatedText),
    importance: Number.isFinite(importance) ? Math.max(0, Math.min(1, importance)) : 0,
    metadata: cloneJsonRecord(form.metadata),
  }
}

export function knowledgeEntryEditFormSnapshot(
  form: KnowledgeEntryEditForm,
): string {
  return JSON.stringify(form)
}

export function parseKnowledgeCommaList(value: string): string[] {
  return [...new Set(value.split(',').map((item) => item.trim()).filter(Boolean))]
}

export function nextKnowledgeEntryId(
  existingIds: readonly string[],
  base = 'new_entry',
): string {
  const normalizedBase = base.trim() || 'new_entry'
  const existing = new Set(existingIds)
  let index = 1
  while (existing.has(`${normalizedBase}_${index}`)) index += 1
  return `${normalizedBase}_${index}`
}

export function knowledgeTaxonomy(
  entries: readonly KnowledgeEntryDefinition[],
): KnowledgeTaxonomy {
  return {
    categories: [...new Set(entries.map((entry) => entry.category).filter(Boolean))].sort(),
    tags: [...new Set(entries.flatMap((entry) => entry.tags).filter(Boolean))].sort(),
  }
}

export function countKnowledgeEntriesByCategory(
  entries: readonly KnowledgeEntryDefinition[],
  category: string,
): number {
  return entries.filter((entry) => entry.category === category).length
}

export function filterKnowledgeEntries(
  entries: readonly KnowledgeEntryDefinition[],
  filter: KnowledgeEntryFilter,
): KnowledgeEntryDefinition[] {
  const query = filter.query.trim().toLocaleLowerCase()
  return entries.filter((entry) => (
    (!filter.category || entry.category === filter.category)
    && (!filter.tag || entry.tags.includes(filter.tag))
    && (!query
      || entry.id.toLocaleLowerCase().includes(query)
      || entry.title.toLocaleLowerCase().includes(query)
      || entry.content.toLocaleLowerCase().includes(query)
      || entry.tags.some((tag) => tag.toLocaleLowerCase().includes(query)))
  ))
}

export function validateKnowledgeEntryEditForm(
  form: KnowledgeEntryEditForm,
  context: KnowledgeEditingContext,
): KnowledgeEditingIssue[] {
  const issues: KnowledgeEditingIssue[] = []
  const category = form.category.trim().toLowerCase()
  if (!KNOWLEDGE_ID.test(form.id)) issues.push({ code: 'invalid_id' })
  if (!form.title.trim()) issues.push({ code: 'title_required' })
  if (!form.content.trim()) issues.push({ code: 'content_required' })
  if (!form.category.trim()) issues.push({ code: 'category_required' })
  else if (!KNOWLEDGE_CATEGORY.test(category)) issues.push({ code: 'category_format' })
  if (!context.original_entry_id && context.existing_entry_ids.includes(form.id)) {
    issues.push({ code: 'duplicate_id', target_id: form.id })
  }

  for (const relatedId of parseKnowledgeCommaList(form.relatedText)) {
    if (!KNOWLEDGE_ID.test(relatedId)) {
      issues.push({ code: 'related_id', target_id: relatedId })
    } else if (relatedId === form.id) {
      issues.push({ code: 'related_self', target_id: relatedId })
    } else if (!context.existing_entry_ids.includes(relatedId)) {
      issues.push({ code: 'related_missing', target_id: relatedId })
    }
  }
  return issues
}
