<template>
  <div class="knowledge-workbench">
    <header class="page-header">
      <div>
        <span class="eyebrow">{{ t('knowledge.eyebrow', 'Project context') }}</span>
        <h1>{{ t('knowledge.title', 'Knowledge Base') }}</h1>
        <p>{{ t('knowledge.subtitle', 'Author the world facts and character context injected into model prompts.') }}</p>
      </div>
      <div class="header-actions">
        <label class="search-field">
          <Search :size="15" />
          <input v-model="searchQuery" class="input" :placeholder="t('knowledge.search', 'Search knowledge...')" />
        </label>
        <button v-if="browserDraft" class="btn btn-secondary btn-sm" @click="restoreProjectKnowledge"><RotateCcw :size="14" />{{ t('knowledge.restore-project', 'Restore project') }}</button>
        <button class="btn btn-primary btn-sm" @click="createEntry"><Plus :size="14" />{{ t('knowledge.create', 'Create Entry') }}</button>
      </div>
    </header>

    <div class="knowledge-layout">
      <aside class="filter-panel" :aria-label="t('knowledge.filters', 'Knowledge filters')">
        <section>
          <span class="filter-label">{{ t('knowledge.categories', 'Categories') }}</span>
          <div class="category-strip">
            <button class="filter-btn" :class="{ active: selectedCategory === null }" @click="selectedCategory = null">
              <span>{{ t('knowledge.all', 'All') }}</span><small>{{ entries.length }}</small>
            </button>
            <button
              v-for="category in categories"
              :key="category"
              class="filter-btn"
              :class="{ active: selectedCategory === category }"
              @click="selectedCategory = category"
            >
              <span>{{ category }}</span><small>{{ countByCategory(category) }}</small>
            </button>
          </div>
        </section>
        <section v-if="allTags.length > 0">
          <span class="filter-label">{{ t('knowledge.tags', 'Tags') }}</span>
          <div class="tag-cloud">
            <button
              v-for="tag in allTags"
              :key="tag"
              :class="{ active: selectedTag === tag }"
              @click="selectedTag = selectedTag === tag ? null : tag"
            >{{ tag }}</button>
          </div>
        </section>
      </aside>

      <main class="knowledge-main">
        <div v-if="loading" class="empty-state"><LoaderCircle class="spinner" :size="26" /><p>{{ t('knowledge.loading', 'Loading knowledge...') }}</p></div>

        <section v-else-if="editing" class="entry-editor">
          <div class="surface-toolbar">
            <div>
              <span class="eyebrow">{{ isNewEntry ? t('knowledge.new-entry', 'New entry') : t('knowledge.editing', 'Editing') }}</span>
              <strong>{{ editForm.title || editForm.id }}</strong>
            </div>
            <div class="toolbar-actions">
              <button class="btn btn-secondary btn-sm" @click="cancelEdit"><X :size="14" />{{ t('common.cancel', 'Cancel') }}</button>
              <button class="btn btn-primary btn-sm" :disabled="saving || !canSave" @click="saveEntry"><Save :size="14" />{{ saving ? t('knowledge.saving', 'Saving') : t('common.save', 'Save') }}</button>
            </div>
          </div>

          <div class="entry-form">
            <div class="form-row">
              <label class="form-field">
                <span>{{ t('knowledge.entry-id', 'Entry ID') }}</span>
                <input v-model.trim="editForm.id" class="input mono" :disabled="!isNewEntry" maxlength="128" placeholder="world_lore_entry" />
              </label>
              <label class="form-field">
                <span>{{ t('common.name', 'Name') }}</span>
                <input v-model.trim="editForm.title" class="input" maxlength="256" :placeholder="t('knowledge.title-placeholder', 'Entry title')" />
              </label>
            </div>
            <div class="form-row category-row">
              <label class="form-field">
                <span>{{ t('knowledge.category', 'Category') }}</span>
                <input v-model.trim="editForm.category" class="input mono" maxlength="64" list="knowledge-categories" placeholder="world_lore" />
                <datalist id="knowledge-categories"><option v-for="category in categories" :key="category" :value="category" /></datalist>
              </label>
              <label class="form-field importance-field">
                <span>{{ t('knowledge.importance', 'Importance') }}</span>
                <div><input v-model.number="editForm.importance" type="range" min="0" max="1" step="0.05" /><b>{{ Math.round(editForm.importance * 100) }}%</b></div>
              </label>
            </div>
            <label class="form-field">
              <span>{{ t('knowledge.content', 'Content') }}</span>
              <textarea v-model="editForm.content" class="input content-input" rows="12" maxlength="16384" :placeholder="t('knowledge.content-placeholder', 'Write the canonical context supplied to model prompts...')"></textarea>
            </label>
            <div class="form-row">
              <label class="form-field">
                <span>{{ t('knowledge.tags-comma', 'Tags (comma-separated)') }}</span>
                <input v-model="editForm.tagsText" class="input" placeholder="world, nature, history" />
              </label>
              <label class="form-field">
                <span>{{ t('knowledge.related-comma', 'Related entries (comma-separated)') }}</span>
                <input v-model="editForm.relatedText" class="input mono" placeholder="location_park, item_flower" />
              </label>
            </div>
            <p v-if="validationMessage" class="validation-message">{{ validationMessage }}</p>
          </div>
        </section>

        <section v-else-if="viewingEntry" class="entry-detail">
          <div class="surface-toolbar">
            <div><span class="eyebrow">{{ t('knowledge.entry-detail', 'Entry detail') }}</span><strong>{{ viewingEntry.id }}</strong></div>
            <div class="toolbar-actions">
              <button class="btn btn-secondary btn-sm" @click="editEntry(viewingEntry)"><Pencil :size="14" />{{ t('common.edit', 'Edit') }}</button>
              <button class="btn btn-danger btn-sm" @click="deleteEntry(viewingEntry)"><Trash2 :size="14" />{{ t('common.delete', 'Delete') }}</button>
              <button class="btn btn-secondary btn-sm" @click="viewingEntry = null"><ArrowLeft :size="14" />{{ t('common.back', 'Back') }}</button>
            </div>
          </div>
          <article class="detail-content">
            <div class="detail-heading">
              <div>
                <h2>{{ viewingEntry.title }}</h2>
                <p>{{ viewingEntry.id }}</p>
              </div>
              <span class="importance-badge">{{ Math.round(viewingEntry.importance * 100) }}%</span>
            </div>
            <div class="detail-meta">
              <span>{{ viewingEntry.category || 'other' }}</span>
              <span v-for="tag in viewingEntry.tags" :key="tag">{{ tag }}</span>
            </div>
            <p class="detail-body">{{ viewingEntry.content }}</p>
            <section v-if="viewingEntry.related_entries.length > 0" class="related-entries">
              <span class="eyebrow">{{ t('knowledge.related', 'Related entries') }}</span>
              <button v-for="id in viewingEntry.related_entries" :key="id" @click="openRelated(id)"><Network :size="12" />{{ id }}</button>
            </section>
          </article>
        </section>

        <section v-else class="entry-list">
          <div class="list-summary">
            <span>{{ t('knowledge.visible-count', '{visible} of {total} entries', { visible: filteredEntries.length, total: entries.length }) }}</span>
            <button v-if="selectedCategory || selectedTag || searchQuery" @click="clearFilters">{{ t('knowledge.clear-filters', 'Clear filters') }}</button>
          </div>
          <div v-if="filteredEntries.length === 0" class="empty-state">
            <BookOpen :size="30" />
            <h2>{{ t('knowledge.no-results', 'No entries found') }}</h2>
            <p>{{ t('knowledge.empty-copy', 'Create a knowledge entry to anchor model responses in your project world.') }}</p>
          </div>
          <button v-for="entry in filteredEntries" :key="entry.id" class="entry-row" @click="viewEntry(entry)">
            <span class="entry-icon"><BookOpen :size="16" /></span>
            <span class="entry-copy">
              <span class="entry-title"><strong>{{ entry.title }}</strong><small>{{ entry.id }}</small></span>
              <span>{{ truncate(entry.content, 150) }}</span>
            </span>
            <span class="entry-facts"><small>{{ entry.category || 'other' }}</small><b>{{ Math.round(entry.importance * 100) }}%</b></span>
          </button>
        </section>
      </main>
    </div>

    <Transition name="fade">
      <div v-if="pendingConfirmation" class="modal-backdrop" @click.self="cancelConfirmation">
        <section class="confirm-dialog" role="dialog" aria-modal="true" :aria-label="t('common.confirm', 'Confirm')">
          <span class="confirm-icon" :class="pendingConfirmation.kind"><Trash2 v-if="pendingConfirmation.kind === 'delete'" :size="18" /><RotateCcw v-else :size="18" /></span>
          <div class="confirm-copy">
            <span class="eyebrow">{{ t('common.confirm', 'Confirm') }}</span>
            <h2>{{ confirmationMessage }}</h2>
          </div>
          <div class="confirm-actions">
            <button class="btn btn-secondary btn-sm" :disabled="confirming" @click="cancelConfirmation"><X :size="14" />{{ t('common.cancel', 'Cancel') }}</button>
            <button class="btn btn-danger btn-sm" :disabled="confirming" @click="confirmPendingAction">
              <LoaderCircle v-if="confirming" class="spinner" :size="14" />
              <Trash2 v-else-if="pendingConfirmation.kind === 'delete'" :size="14" />
              <RotateCcw v-else :size="14" />
              {{ confirmationActionLabel }}
            </button>
          </div>
        </section>
      </div>
    </Transition>

    <Transition name="fade"><div v-if="statusMessage" class="status-toast" :class="statusType" @click="statusMessage = ''">{{ statusMessage }}</div></Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { ArrowLeft, BookOpen, LoaderCircle, Network, Pencil, Plus, RotateCcw, Save, Search, Trash2, X } from '@lucide/vue'
import { useI18n } from '../lib/i18n'
import {
  deleteKnowledgeEntryDefinition,
  KNOWLEDGE_AUTHORING_SCHEMA_V1,
  loadKnowledgeAuthoringCatalog,
  resetBrowserKnowledgeDrafts,
  saveKnowledgeEntryDefinition,
  type KnowledgeCatalogSnapshot,
  type KnowledgeEntryDefinition,
} from '../lib/knowledgeContent'

interface EditForm {
  id: string
  title: string
  category: string
  content: string
  tagsText: string
  relatedText: string
  importance: number
  metadata: Record<string, unknown>
}

type PendingConfirmation =
  | { kind: 'delete'; entry: KnowledgeEntryDefinition }
  | { kind: 'restore' }

const { t } = useI18n()
const entries = ref<KnowledgeEntryDefinition[]>([])
const catalogFingerprint = ref('')
const browserDraft = ref(false)
const searchQuery = ref('')
const selectedCategory = ref<string | null>(null)
const selectedTag = ref<string | null>(null)
const editing = ref(false)
const isNewEntry = ref(false)
const originalEntryId = ref<string | null>(null)
const viewingEntry = ref<KnowledgeEntryDefinition | null>(null)
const loading = ref(true)
const saving = ref(false)
const statusMessage = ref('')
const statusType = ref<'success' | 'error'>('success')
const editForm = ref<EditForm>(emptyForm())
const pendingConfirmation = ref<PendingConfirmation | null>(null)
const confirming = ref(false)

const categories = computed(() => [...new Set(entries.value.map(entry => entry.category).filter(Boolean))].sort())
const allTags = computed(() => [...new Set(entries.value.flatMap(entry => entry.tags))].sort())
const filteredEntries = computed(() => {
  const query = searchQuery.value.trim().toLowerCase()
  return entries.value.filter(entry => (
    (!selectedCategory.value || entry.category === selectedCategory.value)
    && (!selectedTag.value || entry.tags.includes(selectedTag.value))
    && (!query
      || entry.id.toLowerCase().includes(query)
      || entry.title.toLowerCase().includes(query)
      || entry.content.toLowerCase().includes(query)
      || entry.tags.some(tag => tag.toLowerCase().includes(query)))
  ))
})
const validationMessage = computed(() => {
  if (!/^[a-z0-9_-]{1,128}$/.test(editForm.value.id)) return t('knowledge.validation.id', 'ID must use lowercase letters, numbers, underscores, or hyphens.')
  if (!editForm.value.title.trim()) return t('knowledge.validation.title', 'Title is required.')
  if (!editForm.value.content.trim()) return t('knowledge.validation.content', 'Content is required.')
  if (!editForm.value.category.trim()) return t('knowledge.validation.category', 'Category is required.')
  if (!/^[a-z0-9_-]{1,64}$/.test(editForm.value.category.trim().toLowerCase())) return t('knowledge.validation.category-format', 'Category must use lowercase letters, numbers, underscores, or hyphens.')
  if (isNewEntry.value && entries.value.some(entry => entry.id === editForm.value.id)) return t('knowledge.validation.duplicate', 'This entry ID already exists.')
  for (const relatedId of commaList(editForm.value.relatedText)) {
    if (!/^[a-z0-9_-]{1,128}$/.test(relatedId)) return t('knowledge.validation.related-id', 'Related entry IDs must use lowercase letters, numbers, underscores, or hyphens.')
    if (relatedId === editForm.value.id) return t('knowledge.validation.related-self', 'An entry cannot reference itself.')
    if (!entries.value.some(entry => entry.id === relatedId)) return t('knowledge.validation.related-missing', 'Related entry "{id}" does not exist.', { id: relatedId })
  }
  return ''
})
const canSave = computed(() => !validationMessage.value)
const confirmationMessage = computed(() => {
  const pending = pendingConfirmation.value
  if (!pending) return ''
  return pending.kind === 'delete'
    ? t('knowledge.delete-confirm', 'Delete "{title}"?', { title: pending.entry.title })
    : t('knowledge.restore-confirm', 'Discard browser drafts and restore project knowledge?')
})
const confirmationActionLabel = computed(() => pendingConfirmation.value?.kind === 'delete'
  ? t('common.delete', 'Delete')
  : t('knowledge.restore-project', 'Restore project'))

function emptyForm(): EditForm {
  return { id: '', title: '', category: 'world_lore', content: '', tagsText: '', relatedText: '', importance: 0.5, metadata: {} }
}

function applySnapshot(snapshot: KnowledgeCatalogSnapshot) {
  if (snapshot.schema !== KNOWLEDGE_AUTHORING_SCHEMA_V1) throw new Error(`Unsupported knowledge schema: ${snapshot.schema}`)
  entries.value = snapshot.entries
  catalogFingerprint.value = snapshot.catalog_fingerprint
  browserDraft.value = Boolean(snapshot.browser_draft)
  if (viewingEntry.value) viewingEntry.value = entries.value.find(entry => entry.id === viewingEntry.value?.id) || null
}

async function loadEntries() {
  loading.value = true
  try {
    applySnapshot(await loadKnowledgeAuthoringCatalog())
  } catch (error) {
    notify('error', t('knowledge.notice.load-failed', 'Knowledge catalog could not be loaded: {error}', { error: String(error) }))
    entries.value = []
  } finally {
    loading.value = false
  }
}

function createEntry() {
  editForm.value = { ...emptyForm(), id: nextEntryId() }
  originalEntryId.value = null
  isNewEntry.value = true
  editing.value = true
  viewingEntry.value = null
}

function editEntry(entry: KnowledgeEntryDefinition) {
  editForm.value = {
    id: entry.id,
    title: entry.title,
    category: entry.category,
    content: entry.content,
    tagsText: entry.tags.join(', '),
    relatedText: entry.related_entries.join(', '),
    importance: entry.importance,
    metadata: cloneMetadata(entry.metadata),
  }
  originalEntryId.value = entry.id
  isNewEntry.value = false
  editing.value = true
}

function cancelEdit() {
  editing.value = false
  if (originalEntryId.value) viewingEntry.value = entries.value.find(entry => entry.id === originalEntryId.value) || null
}

function viewEntry(entry: KnowledgeEntryDefinition) {
  viewingEntry.value = entry
  editing.value = false
}

async function saveEntry() {
  if (!canSave.value || saving.value) return
  saving.value = true
  try {
    const entry = formEntry()
    applySnapshot(await saveKnowledgeEntryDefinition(entry, originalEntryId.value, catalogFingerprint.value))
    viewingEntry.value = entries.value.find(item => item.id === entry.id) || null
    editing.value = false
    isNewEntry.value = false
    originalEntryId.value = entry.id
    notify('success', t('knowledge.notice.saved', 'Knowledge entry saved.'))
  } catch (error) {
    notify('error', t('knowledge.notice.save-failed', 'Knowledge entry could not be saved: {error}', { error: String(error) }))
  } finally {
    saving.value = false
  }
}

function deleteEntry(entry: KnowledgeEntryDefinition) {
  pendingConfirmation.value = { kind: 'delete', entry }
}

function restoreProjectKnowledge() {
  pendingConfirmation.value = { kind: 'restore' }
}

function cancelConfirmation() {
  if (!confirming.value) pendingConfirmation.value = null
}

async function confirmPendingAction() {
  const pending = pendingConfirmation.value
  if (!pending || confirming.value) return
  confirming.value = true
  try {
    if (pending.kind === 'delete') {
      applySnapshot(await deleteKnowledgeEntryDefinition(pending.entry.id, catalogFingerprint.value))
      viewingEntry.value = null
      notify('success', t('knowledge.notice.deleted', 'Knowledge entry deleted.'))
    } else {
      applySnapshot(await resetBrowserKnowledgeDrafts())
      viewingEntry.value = null
      editing.value = false
      notify('success', t('knowledge.notice.restored', 'Project knowledge restored.'))
    }
    pendingConfirmation.value = null
  } catch (error) {
    const message = pending.kind === 'delete'
      ? t('knowledge.notice.delete-failed', 'Knowledge entry could not be deleted: {error}', { error: String(error) })
      : t('knowledge.notice.restore-failed', 'Project knowledge could not be restored: {error}', { error: String(error) })
    notify('error', message)
  } finally {
    confirming.value = false
  }
}

function formEntry(): KnowledgeEntryDefinition {
  return {
    id: editForm.value.id.trim(),
    title: editForm.value.title.trim(),
    category: editForm.value.category.trim().toLowerCase(),
    content: editForm.value.content.trim(),
    tags: commaList(editForm.value.tagsText),
    related_entries: commaList(editForm.value.relatedText),
    importance: Math.max(0, Math.min(1, Number(editForm.value.importance) || 0)),
    metadata: cloneMetadata(editForm.value.metadata),
  }
}

function commaList(value: string): string[] {
  return [...new Set(value.split(',').map(item => item.trim()).filter(Boolean))]
}

function cloneMetadata(value: Record<string, unknown>): Record<string, unknown> {
  return JSON.parse(JSON.stringify(value)) as Record<string, unknown>
}

function nextEntryId(): string {
  let index = 1
  while (entries.value.some(entry => entry.id === `new_entry_${index}`)) index += 1
  return `new_entry_${index}`
}

function countByCategory(category: string): number {
  return entries.value.filter(entry => entry.category === category).length
}

function truncate(value: string, length: number): string {
  return value.length > length ? `${value.slice(0, length)}...` : value
}

function clearFilters() {
  searchQuery.value = ''
  selectedCategory.value = null
  selectedTag.value = null
}

function openRelated(id: string) {
  const entry = entries.value.find(item => item.id === id)
  if (entry) viewEntry(entry)
}

function notify(type: 'success' | 'error', message: string) {
  statusType.value = type
  statusMessage.value = message
}

onMounted(loadEntries)
</script>

<style scoped>
.knowledge-workbench { max-width: 1280px; margin: 0 auto; padding: 32px 36px 48px; }
.page-header { display: flex; align-items: flex-start; justify-content: space-between; gap: 18px; margin-bottom: 20px; }
.page-header h1 { margin: 3px 0 0; color: var(--text-primary); font-size: 28px; line-height: 1.15; }
.page-header p { max-width: 640px; margin: 7px 0 0; color: var(--text-secondary); font-size: 13px; }
.eyebrow { display: block; color: var(--text-tertiary); font-size: 11px; font-weight: 800; text-transform: uppercase; }
.header-actions { display: flex; align-items: center; gap: 8px; flex-shrink: 0; }
.search-field { display: flex; align-items: center; gap: 7px; min-width: 250px; height: 34px; padding: 0 10px; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--surface-1); color: var(--text-tertiary); }
.search-field .input { min-width: 0; height: 30px; padding: 0; border: 0; background: transparent; }
.btn { display: inline-flex; align-items: center; justify-content: center; gap: 7px; }
.knowledge-layout { display: grid; grid-template-columns: 210px minmax(0, 1fr); gap: 18px; align-items: start; }
.filter-panel { display: grid; gap: 22px; padding-right: 14px; border-right: 1px solid var(--border); }
.filter-panel section { display: grid; gap: 5px; }
.category-strip { display: grid; gap: 5px; }
.filter-label { margin-bottom: 4px; color: var(--text-tertiary); font-size: 10px; font-weight: 800; text-transform: uppercase; }
.filter-btn { display: flex; align-items: center; justify-content: space-between; gap: 8px; min-height: 32px; padding: 5px 8px; border: 0; border-radius: var(--radius-sm); background: transparent; color: var(--text-secondary); cursor: pointer; font: inherit; font-size: 11px; text-align: left; }
.filter-btn small { color: var(--text-tertiary); }
.filter-btn:hover { background: var(--surface-2); color: var(--text-primary); }
.filter-btn.active { background: rgba(45, 212, 191, 0.1); color: var(--brand-light); }
.tag-cloud { display: flex; flex-wrap: wrap; gap: 5px; }
.tag-cloud button { padding: 3px 7px; border: 1px solid var(--border); border-radius: 999px; background: transparent; color: var(--text-tertiary); cursor: pointer; font: inherit; font-size: 9px; }
.tag-cloud button.active { border-color: var(--brand); color: var(--brand-light); }
.knowledge-main { min-width: 0; }
.surface-toolbar { display: flex; align-items: center; justify-content: space-between; gap: 14px; min-height: 52px; padding-bottom: 12px; border-bottom: 1px solid var(--border); }
.surface-toolbar strong { display: block; margin-top: 3px; color: var(--text-primary); font-size: 14px; }
.toolbar-actions { display: flex; gap: 7px; }
.entry-form { display: grid; gap: 15px; padding-top: 17px; }
.form-row { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 14px; }
.category-row { grid-template-columns: minmax(0, 1fr) 220px; }
.form-field { display: grid; gap: 6px; min-width: 0; }
.form-field > span { color: var(--text-secondary); font-size: 11px; font-weight: 750; }
.mono { font-family: var(--font-mono); }
.content-input { min-height: 240px; resize: vertical; line-height: 1.6; }
.importance-field > div { display: grid; grid-template-columns: minmax(0, 1fr) 40px; gap: 10px; align-items: center; min-height: 38px; }
.importance-field input { width: 100%; accent-color: var(--brand); }
.importance-field b { color: var(--brand-light); font-family: var(--font-mono); font-size: 11px; text-align: right; }
.validation-message { margin: 0; color: var(--warning); font-size: 11px; }
.detail-content { padding-top: 18px; }
.detail-heading { display: flex; align-items: flex-start; justify-content: space-between; gap: 14px; }
.detail-heading h2 { margin: 0; color: var(--text-primary); font-size: 24px; }
.detail-heading p { margin: 4px 0 0; color: var(--text-tertiary); font-family: var(--font-mono); font-size: 10px; }
.importance-badge { padding: 5px 9px; border: 1px solid var(--border); border-radius: var(--radius-sm); color: var(--brand-light); font-family: var(--font-mono); font-size: 11px; }
.detail-meta { display: flex; flex-wrap: wrap; gap: 5px; margin-top: 13px; }
.detail-meta span { padding: 3px 8px; border: 1px solid var(--border); border-radius: 999px; color: var(--text-secondary); font-size: 9px; }
.detail-body { margin: 20px 0; color: var(--text-secondary); font-size: 13px; line-height: 1.75; white-space: pre-wrap; }
.related-entries { display: flex; flex-wrap: wrap; gap: 6px; padding-top: 14px; border-top: 1px solid var(--border); }
.related-entries .eyebrow { flex-basis: 100%; margin-bottom: 4px; }
.related-entries button { display: inline-flex; align-items: center; gap: 5px; padding: 4px 8px; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--surface-2); color: var(--text-secondary); cursor: pointer; font: inherit; font-family: var(--font-mono); font-size: 9px; }
.list-summary { display: flex; align-items: center; justify-content: space-between; min-height: 38px; margin-bottom: 8px; color: var(--text-tertiary); font-size: 10px; }
.list-summary button { border: 0; background: transparent; color: var(--brand-light); cursor: pointer; font: inherit; font-size: 10px; }
.entry-list { display: grid; gap: 7px; }
.entry-row { display: grid; grid-template-columns: 36px minmax(0, 1fr) auto; gap: 11px; align-items: center; min-height: 74px; padding: 10px; border: 1px solid var(--border); border-radius: var(--radius); background: var(--surface-1); color: inherit; cursor: pointer; font: inherit; text-align: left; }
.entry-row:hover { border-color: var(--brand); background: var(--surface-2); }
.entry-icon { display: grid; place-items: center; width: 36px; height: 36px; border-radius: var(--radius-sm); background: var(--surface-3); color: var(--brand-light); }
.entry-copy { display: grid; gap: 4px; min-width: 0; }
.entry-title { display: flex; align-items: baseline; gap: 8px; min-width: 0; }
.entry-title strong { color: var(--text-primary); font-size: 12px; }
.entry-title small { overflow: hidden; color: var(--text-tertiary); font-family: var(--font-mono); font-size: 9px; text-overflow: ellipsis; white-space: nowrap; }
.entry-copy > span:last-child { overflow: hidden; color: var(--text-tertiary); font-size: 10px; text-overflow: ellipsis; white-space: nowrap; }
.entry-facts { display: grid; justify-items: end; gap: 4px; }
.entry-facts small { color: var(--text-tertiary); font-size: 9px; }
.entry-facts b { color: var(--brand-light); font-family: var(--font-mono); font-size: 10px; }
.empty-state { display: grid; place-items: center; gap: 8px; min-height: 300px; color: var(--text-tertiary); text-align: center; }
.empty-state h2, .empty-state p { margin: 0; }
.empty-state h2 { color: var(--text-primary); font-size: 18px; }
.empty-state p { max-width: 460px; font-size: 11px; }
.spinner { animation: spin 0.8s linear infinite; }
.modal-backdrop { position: fixed; z-index: 100; inset: 0; display: grid; place-items: center; padding: 20px; background: rgba(4, 8, 15, 0.66); backdrop-filter: blur(4px); }
.confirm-dialog { display: grid; grid-template-columns: 38px minmax(0, 1fr); gap: 12px; width: min(430px, 100%); padding: 18px; border: 1px solid var(--border); border-radius: var(--radius); background: var(--surface-1); box-shadow: var(--shadow-lg); }
.confirm-icon { display: grid; place-items: center; width: 38px; height: 38px; border-radius: var(--radius-sm); background: rgba(239, 68, 68, 0.12); color: var(--danger); }
.confirm-icon.restore { background: rgba(245, 158, 11, 0.12); color: var(--warning); }
.confirm-copy { min-width: 0; }
.confirm-copy h2 { margin: 5px 0 0; color: var(--text-primary); font-size: 14px; line-height: 1.45; }
.confirm-actions { grid-column: 1 / -1; display: flex; justify-content: flex-end; gap: 7px; margin-top: 6px; padding-top: 13px; border-top: 1px solid var(--border); }
.status-toast { position: fixed; z-index: 80; right: 20px; bottom: 20px; max-width: min(460px, calc(100vw - 32px)); padding: 11px 13px; border: 1px solid rgba(45, 212, 191, 0.35); border-radius: var(--radius); background: rgba(15, 118, 110, 0.96); color: white; box-shadow: var(--shadow-lg); font-size: 11px; }
.status-toast.error { border-color: rgba(239, 68, 68, 0.4); background: rgba(127, 29, 29, 0.96); }
.fade-enter-active, .fade-leave-active { transition: opacity 0.2s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
@keyframes spin { to { transform: rotate(360deg); } }

@media (max-width: 900px) {
  .knowledge-layout { grid-template-columns: 1fr; }
  .filter-panel { display: flex; gap: 16px; overflow-x: auto; padding: 0 0 12px; border-right: 0; border-bottom: 1px solid var(--border); }
  .filter-panel section { min-width: 180px; }
}

@media (max-width: 720px) {
  .knowledge-workbench { padding: 22px 16px 96px; }
  .page-header { flex-direction: column; }
  .header-actions { display: grid; grid-template-columns: 1fr auto; width: 100%; }
  .search-field { grid-column: 1 / -1; min-width: 0; }
  .filter-panel { display: grid; gap: 12px; overflow: visible; }
  .filter-panel section { min-width: 0; }
  .category-strip, .tag-cloud { display: flex; flex-wrap: nowrap; gap: 6px; overflow-x: auto; padding-bottom: 3px; scrollbar-width: none; }
  .category-strip::-webkit-scrollbar, .tag-cloud::-webkit-scrollbar { display: none; }
  .filter-btn { flex: 0 0 auto; min-height: 30px; gap: 10px; padding-inline: 9px; border: 1px solid var(--border); }
  .tag-cloud button { flex: 0 0 auto; }
  .surface-toolbar { align-items: flex-start; flex-direction: column; }
  .toolbar-actions { width: 100%; }
  .toolbar-actions .btn { flex: 1; }
  .form-row, .category-row { grid-template-columns: 1fr; }
  .entry-row { grid-template-columns: 36px minmax(0, 1fr); }
  .entry-facts { grid-column: 2; grid-auto-flow: column; justify-content: start; justify-items: start; }
}
</style>
