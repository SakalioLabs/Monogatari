<template>
  <div class="knowledge-workbench">
    <header class="workbench-header">
      <div class="title-lockup">
        <span class="title-icon"><BookOpen :size="17" /></span>
        <div>
          <span class="eyebrow">{{ t('knowledge.eyebrow', 'Project context') }}</span>
          <h1>{{ t('knowledge.title', 'Knowledge Base') }}</h1>
        </div>
      </div>

      <div class="header-summary" :aria-label="t('knowledge.catalog-summary', 'Knowledge catalog summary')">
        <span><strong>{{ entries.length }}</strong>{{ t('knowledge.entries', 'Entries') }}</span>
        <span><strong>{{ categories.length }}</strong>{{ t('knowledge.categories', 'Categories') }}</span>
      </div>

      <div class="header-actions">
        <button
          v-if="browserDraft"
          class="icon-command restore-command"
          :title="t('knowledge.restore-project', 'Restore project')"
          :aria-label="t('knowledge.restore-project', 'Restore project')"
          @click="restoreProjectKnowledge"
        ><RotateCcw :size="15" /></button>
        <button
          class="primary-command"
          :disabled="saving"
          :title="t('knowledge.create', 'Create Entry')"
          :aria-label="t('knowledge.create', 'Create Entry')"
          @click="requestCreateEntry"
        >
          <Plus :size="15" />
          <span>{{ t('knowledge.create', 'Create Entry') }}</span>
        </button>
      </div>
    </header>

    <aside class="taxonomy-panel" :aria-label="t('knowledge.filters', 'Knowledge filters')">
      <section class="taxonomy-group categories-group">
        <div class="panel-heading">
          <span><FolderTree :size="13" />{{ t('knowledge.categories', 'Categories') }}</span>
          <strong>{{ categories.length }}</strong>
        </div>
        <nav class="category-list">
          <button :class="{ active: selectedCategory === null }" @click="selectedCategory = null">
            <span>{{ t('knowledge.all', 'All') }}</span>
            <small>{{ entries.length }}</small>
          </button>
          <button
            v-for="category in categories"
            :key="category"
            :class="{ active: selectedCategory === category }"
            @click="selectedCategory = category"
          >
            <span>{{ categoryLabel(category) }}</span>
            <small>{{ countByCategory(category) }}</small>
          </button>
        </nav>
      </section>

      <section class="taxonomy-group tags-group">
        <div class="panel-heading">
          <span><Tag :size="13" />{{ t('knowledge.tags', 'Tags') }}</span>
          <strong>{{ allTags.length }}</strong>
        </div>
        <div class="tag-list">
          <button
            v-for="tag in allTags"
            :key="tag"
            :class="{ active: selectedTag === tag }"
            @click="selectedTag = selectedTag === tag ? null : tag"
          ><Hash :size="10" /><span>{{ tag }}</span></button>
          <span v-if="allTags.length === 0" class="taxonomy-empty">{{ t('knowledge.no-tags', 'No tags') }}</span>
        </div>
      </section>
    </aside>

    <section class="entry-panel" :aria-label="t('knowledge.entries', 'Entries')">
      <div class="list-tools">
        <label class="search-field">
          <Search :size="14" />
          <input v-model="searchQuery" :placeholder="t('knowledge.search', 'Search knowledge...')" />
          <button
            v-if="searchQuery"
            type="button"
            :title="t('knowledge.clear-search', 'Clear search')"
            :aria-label="t('knowledge.clear-search', 'Clear search')"
            @click="searchQuery = ''"
          ><X :size="13" /></button>
        </label>
      </div>

      <div class="compact-filters">
        <label>
          <FolderTree :size="12" />
          <select v-model="compactCategoryFilter" :aria-label="t('knowledge.categories', 'Categories')">
            <option value="">{{ t('knowledge.all-categories', 'All categories') }}</option>
            <option v-for="category in categories" :key="category" :value="category">{{ categoryLabel(category) }}</option>
          </select>
        </label>
        <label>
          <Tag :size="12" />
          <select v-model="compactTagFilter" :aria-label="t('knowledge.tags', 'Tags')">
            <option value="">{{ t('knowledge.all-tags', 'All tags') }}</option>
            <option v-for="tag in allTags" :key="tag" :value="tag">{{ tag }}</option>
          </select>
        </label>
      </div>

      <div class="list-summary">
        <span>{{ t('knowledge.visible-count', '{visible} of {total} entries', { visible: filteredEntries.length, total: entries.length }) }}</span>
        <button
          v-if="selectedCategory || selectedTag || searchQuery"
          :title="t('knowledge.clear-filters', 'Clear filters')"
          :aria-label="t('knowledge.clear-filters', 'Clear filters')"
          @click="clearFilters"
        ><RotateCcw :size="12" /></button>
      </div>

      <div class="entry-scroll">
        <div v-if="loading" class="panel-empty">
          <LoaderCircle class="spinner" :size="20" />
          <span>{{ t('knowledge.loading', 'Loading knowledge...') }}</span>
        </div>

        <div v-else-if="filteredEntries.length === 0" class="panel-empty">
          <BookOpen :size="22" />
          <strong>{{ t('knowledge.no-results', 'No entries found') }}</strong>
        </div>

        <button
          v-for="entry in filteredEntries"
          v-else
          :key="entry.id"
          class="entry-row"
          :class="{ selected: viewingEntry?.id === entry.id }"
          :aria-current="viewingEntry?.id === entry.id ? 'true' : undefined"
          @click="requestViewEntry(entry)"
        >
          <span class="entry-glyph"><BookOpen :size="14" /></span>
          <span class="entry-copy">
            <span class="entry-title">{{ entry.title }}</span>
            <span class="entry-id">{{ entry.id }}</span>
            <span class="entry-excerpt">{{ truncate(entry.content, 92) }}</span>
          </span>
          <span class="entry-meta">
            <small>{{ categoryLabel(entry.category) }}</small>
            <strong>{{ Math.round(entry.importance * 100) }}%</strong>
            <span class="importance-track"><i :style="{ width: `${Math.round(entry.importance * 100)}%` }"></i></span>
          </span>
        </button>
      </div>
    </section>

    <aside class="inspector-panel" :class="{ 'compact-open': compactInspectorOpen }" :aria-label="t('knowledge.inspector', 'Entry inspector')">
      <header v-if="editing" class="inspector-header">
        <div>
          <span class="eyebrow">{{ isNewEntry ? t('knowledge.new-entry', 'New entry') : t('knowledge.editing', 'Editing') }}</span>
          <h2>{{ editForm.title || editForm.id }}</h2>
        </div>
        <div class="inspector-actions">
          <button class="secondary-command" :disabled="saving" @click="cancelEdit"><X :size="14" />{{ t('common.cancel', 'Cancel') }}</button>
          <button class="primary-command" :disabled="saving || !canSave" @click="saveEntry">
            <LoaderCircle v-if="saving" class="spinner" :size="14" />
            <Save v-else :size="14" />
            {{ saving ? t('knowledge.saving', 'Saving') : t('common.save', 'Save') }}
          </button>
        </div>
      </header>

      <header v-else-if="viewingEntry" class="inspector-header">
        <div>
          <span class="eyebrow">{{ t('knowledge.entry-detail', 'Entry detail') }}</span>
          <h2>{{ viewingEntry.title }}</h2>
        </div>
        <div class="inspector-actions">
          <button class="icon-command" :title="t('common.edit', 'Edit')" :aria-label="t('common.edit', 'Edit')" @click="editEntry(viewingEntry)"><Pencil :size="14" /></button>
          <button class="icon-command danger-command" :title="t('common.delete', 'Delete')" :aria-label="t('common.delete', 'Delete')" @click="deleteEntry(viewingEntry)"><Trash2 :size="14" /></button>
          <button class="icon-command inspector-close" :title="t('knowledge.close-inspector', 'Close inspector')" :aria-label="t('knowledge.close-inspector', 'Close inspector')" @click="closeCompactInspector"><X :size="14" /></button>
        </div>
      </header>

      <header v-else class="inspector-header empty-header">
        <div>
          <span class="eyebrow">{{ t('knowledge.inspector', 'Entry inspector') }}</span>
          <h2>{{ t('knowledge.select-entry', 'Select an entry') }}</h2>
        </div>
      </header>

      <form v-if="editing" class="inspector-scroll editor-form" @submit.prevent="saveEntry">
        <div class="form-grid">
          <label class="form-field">
            <span>{{ t('knowledge.entry-id', 'Entry ID') }}</span>
            <input v-model.trim="editForm.id" class="input mono" :disabled="!isNewEntry" maxlength="128" :placeholder="t('knowledge.id-placeholder', 'world_lore_entry')" />
          </label>
          <label class="form-field">
            <span>{{ t('common.name', 'Name') }}</span>
            <input v-model.trim="editForm.title" class="input" maxlength="256" :placeholder="t('knowledge.title-placeholder', 'Entry title')" />
          </label>
        </div>

        <div class="form-grid category-grid">
          <label class="form-field">
            <span>{{ t('knowledge.category', 'Category') }}</span>
            <input v-model.trim="editForm.category" class="input mono" maxlength="64" list="knowledge-categories" :placeholder="t('knowledge.category-placeholder', 'world_lore')" />
            <datalist id="knowledge-categories"><option v-for="category in categories" :key="category" :value="category" /></datalist>
          </label>
          <label class="form-field importance-field">
            <span>{{ t('knowledge.importance', 'Importance') }}</span>
            <div>
              <input v-model.number="editForm.importance" type="range" min="0" max="1" step="0.05" />
              <output>{{ Math.round(editForm.importance * 100) }}%</output>
            </div>
          </label>
        </div>

        <label class="form-field content-field">
          <span>{{ t('knowledge.content', 'Content') }}</span>
          <textarea v-model="editForm.content" class="input" rows="12" maxlength="16384" :placeholder="t('knowledge.content-placeholder', 'Write the canonical context supplied to model prompts...')"></textarea>
        </label>

        <label class="form-field">
          <span>{{ t('knowledge.tags-comma', 'Tags (comma-separated)') }}</span>
          <input v-model="editForm.tagsText" class="input" :placeholder="t('knowledge.tags-placeholder', 'world, nature, history')" />
        </label>

        <label class="form-field">
          <span>{{ t('knowledge.related-comma', 'Related entries (comma-separated)') }}</span>
          <input v-model="editForm.relatedText" class="input mono" :placeholder="t('knowledge.related-placeholder', 'location_park, item_flower')" />
        </label>

        <p v-if="validationMessage" class="validation-message"><AlertTriangle :size="13" />{{ validationMessage }}</p>
      </form>

      <div v-else-if="viewingEntry" class="inspector-scroll detail-view">
        <div class="detail-identity">
          <span class="detail-icon"><BookOpen :size="18" /></span>
          <div>
            <h3>{{ viewingEntry.title }}</h3>
            <code>{{ viewingEntry.id }}</code>
          </div>
          <strong>{{ Math.round(viewingEntry.importance * 100) }}%</strong>
        </div>

        <div class="detail-facts">
          <span><FolderTree :size="12" />{{ categoryLabel(viewingEntry.category) }}</span>
          <span><Network :size="12" />{{ t('knowledge.reference-count', '{count} references', { count: viewingEntry.related_entries.length }) }}</span>
        </div>

        <section class="detail-section content-section">
          <span class="section-label">{{ t('knowledge.content', 'Content') }}</span>
          <p>{{ viewingEntry.content }}</p>
        </section>

        <section v-if="viewingEntry.tags.length > 0" class="detail-section">
          <span class="section-label">{{ t('knowledge.tags', 'Tags') }}</span>
          <div class="detail-tags"><span v-for="tag in viewingEntry.tags" :key="tag"><Hash :size="10" />{{ tag }}</span></div>
        </section>

        <section v-if="viewingEntry.related_entries.length > 0" class="detail-section">
          <span class="section-label">{{ t('knowledge.related', 'Related entries') }}</span>
          <div class="related-list">
            <button v-for="id in viewingEntry.related_entries" :key="id" @click="openRelated(id)"><Network :size="12" /><span>{{ relatedTitle(id) }}</span><code>{{ id }}</code></button>
          </div>
        </section>
      </div>

      <div v-else class="inspector-empty"><BookOpen :size="24" /><span>{{ t('knowledge.select-entry', 'Select an entry') }}</span></div>
    </aside>

    <Transition name="fade">
      <div v-if="pendingConfirmation" class="modal-backdrop" @click.self="cancelConfirmation">
        <section class="confirm-dialog" role="dialog" aria-modal="true" :aria-label="t('common.confirm', 'Confirm')">
          <Trash2 v-if="pendingConfirmation.kind === 'delete'" :size="19" />
          <AlertTriangle v-else :size="19" />
          <div>
            <span class="eyebrow">{{ t('common.confirm', 'Confirm') }}</span>
            <h2>{{ confirmationMessage }}</h2>
          </div>
          <footer>
            <button class="secondary-command" :disabled="confirming" @click="cancelConfirmation"><X :size="14" />{{ t('common.cancel', 'Cancel') }}</button>
            <button :class="pendingConfirmation.kind === 'delete' ? 'danger-action' : 'primary-command'" :disabled="confirming" @click="confirmPendingAction">
              <LoaderCircle v-if="confirming" class="spinner" :size="14" />
              <Trash2 v-else-if="pendingConfirmation.kind === 'delete'" :size="14" />
              <RotateCcw v-else-if="pendingConfirmation.kind === 'restore'" :size="14" />
              <AlertTriangle v-else :size="14" />
              {{ confirmationActionLabel }}
            </button>
          </footer>
        </section>
      </div>
    </Transition>

    <Transition name="fade"><button v-if="statusMessage" class="status-toast" :class="statusType" @click="statusMessage = ''">{{ statusMessage }}</button></Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { AlertTriangle, BookOpen, FolderTree, Hash, LoaderCircle, Network, Pencil, Plus, RotateCcw, Save, Search, Tag, Trash2, X } from '@lucide/vue'
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
  | { kind: 'discard'; next: 'create'; target: null }
  | { kind: 'discard'; next: 'view'; target: KnowledgeEntryDefinition }

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
const compactInspectorOpen = ref(false)
const loading = ref(true)
const saving = ref(false)
const statusMessage = ref('')
const statusType = ref<'success' | 'error'>('success')
const editForm = ref<EditForm>(emptyForm())
const editBaseline = ref('')
const pendingConfirmation = ref<PendingConfirmation | null>(null)
const confirming = ref(false)
let statusTimer: number | null = null

const categories = computed(() => [...new Set(entries.value.map(entry => entry.category).filter(Boolean))].sort())
const allTags = computed(() => [...new Set(entries.value.flatMap(entry => entry.tags))].sort())
const compactCategoryFilter = computed({
  get: () => selectedCategory.value || '',
  set: (value: string) => { selectedCategory.value = value || null },
})
const compactTagFilter = computed({
  get: () => selectedTag.value || '',
  set: (value: string) => { selectedTag.value = value || null },
})
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
const isFormDirty = computed(() => editing.value && serializeForm(editForm.value) !== editBaseline.value)
const confirmationMessage = computed(() => {
  const pending = pendingConfirmation.value
  if (!pending) return ''
  if (pending.kind === 'delete') return t('knowledge.delete-confirm', 'Delete "{title}"?', { title: pending.entry.title })
  if (pending.kind === 'restore') return t('knowledge.restore-confirm', 'Discard browser drafts and restore project knowledge?')
  return t('knowledge.discard-confirm', 'Discard unsaved changes?')
})
const confirmationActionLabel = computed(() => {
  if (pendingConfirmation.value?.kind === 'delete') return t('common.delete', 'Delete')
  if (pendingConfirmation.value?.kind === 'restore') return t('knowledge.restore-project', 'Restore project')
  return t('knowledge.discard', 'Discard')
})

function emptyForm(): EditForm {
  return { id: '', title: '', category: 'world_lore', content: '', tagsText: '', relatedText: '', importance: 0.5, metadata: {} }
}

function applySnapshot(snapshot: KnowledgeCatalogSnapshot) {
  if (snapshot.schema !== KNOWLEDGE_AUTHORING_SCHEMA_V1) throw new Error(`Unsupported knowledge schema: ${snapshot.schema}`)
  const selectedId = viewingEntry.value?.id
  entries.value = snapshot.entries
  catalogFingerprint.value = snapshot.catalog_fingerprint
  browserDraft.value = Boolean(snapshot.browser_draft)
  const selectedEntry = selectedId ? entries.value.find(entry => entry.id === selectedId) : null
  if (selectedEntry) viewingEntry.value = selectedEntry
  else if (!editing.value) viewingEntry.value = entries.value[0] || null
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
  editBaseline.value = serializeForm(editForm.value)
  originalEntryId.value = null
  isNewEntry.value = true
  editing.value = true
  compactInspectorOpen.value = true
}

function requestCreateEntry() {
  if (editing.value && isFormDirty.value) {
    pendingConfirmation.value = { kind: 'discard', next: 'create', target: null }
    return
  }
  createEntry()
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
  editBaseline.value = serializeForm(editForm.value)
  originalEntryId.value = entry.id
  isNewEntry.value = false
  viewingEntry.value = entry
  editing.value = true
  compactInspectorOpen.value = true
}

function cancelEdit() {
  editing.value = false
  isNewEntry.value = false
  if (originalEntryId.value) viewingEntry.value = entries.value.find(entry => entry.id === originalEntryId.value) || viewingEntry.value
  originalEntryId.value = null
  editBaseline.value = ''
}

function requestViewEntry(entry: KnowledgeEntryDefinition) {
  if (editing.value && isFormDirty.value) {
    pendingConfirmation.value = { kind: 'discard', next: 'view', target: entry }
    return
  }
  viewEntry(entry)
}

function viewEntry(entry: KnowledgeEntryDefinition) {
  viewingEntry.value = entry
  editing.value = false
  isNewEntry.value = false
  originalEntryId.value = null
  editBaseline.value = ''
  compactInspectorOpen.value = true
}

function closeCompactInspector() {
  compactInspectorOpen.value = false
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
    originalEntryId.value = null
    editBaseline.value = ''
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
      notify('success', t('knowledge.notice.deleted', 'Knowledge entry deleted.'))
    } else if (pending.kind === 'restore') {
      editing.value = false
      applySnapshot(await resetBrowserKnowledgeDrafts())
      notify('success', t('knowledge.notice.restored', 'Project knowledge restored.'))
    } else {
      if (pending.next === 'create') createEntry()
      else viewEntry(pending.target)
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

function serializeForm(value: EditForm): string {
  return JSON.stringify(value)
}

function nextEntryId(): string {
  let index = 1
  while (entries.value.some(entry => entry.id === `new_entry_${index}`)) index += 1
  return `new_entry_${index}`
}

function countByCategory(category: string): number {
  return entries.value.filter(entry => entry.category === category).length
}

function categoryLabel(category: string): string {
  const normalized = category.trim().toLowerCase()
  const keyByCategory: Record<string, string> = {
    character: 'knowledge.category.character',
    items: 'knowledge.category.items',
    location: 'knowledge.category.location',
    locations: 'knowledge.category.locations',
    lore: 'knowledge.category.lore',
    world: 'knowledge.category.world',
    world_lore: 'knowledge.category.world-lore',
  }
  return t(keyByCategory[normalized] || '', category || t('knowledge.category.other', 'Other'))
}

function relatedTitle(id: string): string {
  return entries.value.find(entry => entry.id === id)?.title || id
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
  if (entry) requestViewEntry(entry)
}

function notify(type: 'success' | 'error', message: string) {
  statusType.value = type
  statusMessage.value = message
  if (statusTimer !== null) window.clearTimeout(statusTimer)
  statusTimer = window.setTimeout(() => { statusMessage.value = '' }, 3600)
}

onMounted(loadEntries)
onUnmounted(() => {
  if (statusTimer !== null) window.clearTimeout(statusTimer)
})
</script>

<style scoped>
.knowledge-workbench {
  position: relative;
  display: grid;
  height: calc(100svh - 56px);
  min-height: 0;
  grid-template-columns: 178px 310px minmax(0, 1fr);
  grid-template-rows: 54px minmax(0, 1fr);
  overflow: hidden;
  background: var(--surface-0);
}

.workbench-header {
  display: grid;
  min-width: 0;
  grid-column: 1 / -1;
  grid-template-columns: minmax(210px, 1fr) auto auto;
  align-items: center;
  gap: 14px;
  padding: 8px 12px;
  border-bottom: 1px solid var(--border);
  background: var(--surface-1);
}

.title-lockup,
.header-summary,
.header-actions,
.panel-heading,
.panel-heading > span,
.search-field,
.compact-filters label,
.entry-meta,
.inspector-actions,
.detail-facts,
.detail-facts span,
.detail-tags,
.detail-tags span,
.related-list button,
.validation-message,
.confirm-dialog footer {
  display: flex;
  align-items: center;
}

.title-lockup { min-width: 0; gap: 9px; }
.title-icon,
.entry-glyph,
.detail-icon {
  display: inline-grid;
  flex: 0 0 auto;
  place-items: center;
  border-radius: 6px;
  background: color-mix(in srgb, var(--brand) 14%, var(--surface-2));
  color: var(--brand-light);
}
.title-icon { width: 32px; height: 32px; }
.title-lockup > div { display: grid; min-width: 0; gap: 2px; }
.eyebrow { color: var(--text-tertiary); font-size: 8px; font-weight: 800; letter-spacing: 0; text-transform: uppercase; }
.title-lockup h1,
.inspector-header h2 { margin: 0; overflow: hidden; color: var(--text-primary); text-overflow: ellipsis; white-space: nowrap; }
.title-lockup h1 { font-size: 14px; line-height: 1.2; }

.header-summary { gap: 5px; }
.header-summary > span { display: grid; min-width: 66px; gap: 1px; padding: 4px 8px; border-left: 1px solid var(--border); color: var(--text-tertiary); font-size: 7px; text-transform: uppercase; }
.header-summary strong { color: var(--text-primary); font-family: var(--font-mono); font-size: 10px; }
.header-actions { justify-content: flex-end; gap: 6px; }

.icon-command,
.primary-command,
.secondary-command,
.danger-action {
  display: inline-flex;
  min-width: 0;
  align-items: center;
  justify-content: center;
  gap: 6px;
  border-radius: 6px;
  font: inherit;
  font-size: 9px;
  font-weight: 800;
  cursor: pointer;
  white-space: nowrap;
}
.icon-command { width: 32px; height: 32px; flex: 0 0 32px; padding: 0; border: 1px solid var(--border); background: var(--surface-2); color: var(--text-secondary); }
.primary-command, .secondary-command, .danger-action { min-height: 32px; padding: 0 10px; }
.primary-command { border: 1px solid var(--brand); background: var(--brand); color: var(--surface-0); }
.secondary-command { border: 1px solid var(--border); background: var(--surface-2); color: var(--text-secondary); }
.danger-action { border: 1px solid var(--danger); background: var(--danger); color: white; }
.icon-command:hover:not(:disabled), .secondary-command:hover:not(:disabled) { border-color: var(--border-strong); color: var(--text-primary); }
.primary-command:hover:not(:disabled) { border-color: var(--brand-light); background: var(--brand-light); }
.danger-command:hover:not(:disabled) { border-color: color-mix(in srgb, var(--danger) 55%, var(--border)); color: var(--danger); }
.restore-command { color: var(--warning); }
.icon-command:disabled, .primary-command:disabled, .secondary-command:disabled, .danger-action:disabled { cursor: not-allowed; opacity: 0.42; }

.taxonomy-panel,
.entry-panel,
.inspector-panel { min-width: 0; min-height: 0; overflow: hidden; }
.taxonomy-panel { display: grid; grid-template-rows: minmax(180px, 0.72fr) minmax(160px, 1fr); border-right: 1px solid var(--border); background: var(--surface-1); }
.taxonomy-group { display: grid; min-height: 0; grid-template-rows: 38px minmax(0, 1fr); }
.categories-group { border-bottom: 1px solid var(--border); }
.panel-heading { min-width: 0; justify-content: space-between; gap: 8px; padding: 8px 10px; color: var(--text-tertiary); font-size: 8px; font-weight: 800; text-transform: uppercase; }
.panel-heading > span { min-width: 0; gap: 6px; }
.panel-heading strong { color: var(--text-secondary); font-family: var(--font-mono); font-size: 8px; }
.category-list, .tag-list, .entry-scroll, .inspector-scroll { min-height: 0; overflow-y: auto; scrollbar-width: none; }
.category-list::-webkit-scrollbar, .tag-list::-webkit-scrollbar, .entry-scroll::-webkit-scrollbar, .inspector-scroll::-webkit-scrollbar { display: none; }
.category-list { display: grid; align-content: start; gap: 2px; padding: 2px 6px 8px; }
.category-list button { display: grid; min-width: 0; min-height: 31px; grid-template-columns: minmax(0, 1fr) auto; align-items: center; gap: 8px; padding: 5px 7px; border: 1px solid transparent; border-radius: 5px; background: transparent; color: var(--text-secondary); font: inherit; font-size: 9px; text-align: left; cursor: pointer; }
.category-list button span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.category-list button small { color: var(--text-tertiary); font-family: var(--font-mono); font-size: 8px; }
.category-list button:hover { background: var(--surface-2); }
.category-list button.active { border-color: color-mix(in srgb, var(--brand) 34%, var(--border)); background: color-mix(in srgb, var(--brand) 9%, var(--surface-1)); color: var(--brand-light); }
.tag-list { display: flex; align-content: flex-start; flex-wrap: wrap; gap: 4px; padding: 2px 8px 10px; }
.tag-list button { display: inline-flex; max-width: 100%; height: 24px; align-items: center; gap: 3px; padding: 0 6px; border: 1px solid var(--border); border-radius: 5px; background: transparent; color: var(--text-tertiary); font: inherit; font-size: 8px; cursor: pointer; }
.tag-list button span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.tag-list button:hover { color: var(--text-secondary); }
.tag-list button.active { border-color: color-mix(in srgb, var(--brand) 42%, var(--border)); background: color-mix(in srgb, var(--brand) 8%, transparent); color: var(--brand-light); }
.taxonomy-empty { padding: 6px 2px; color: var(--text-tertiary); font-size: 8px; }

.entry-panel { display: grid; grid-template-rows: 42px 30px minmax(0, 1fr); border-right: 1px solid var(--border); background: var(--surface-0); }
.list-tools { padding: 6px 8px; border-bottom: 1px solid var(--border); }
.search-field { height: 30px; gap: 6px; padding: 0 7px; border: 1px solid var(--border); border-radius: 6px; background: var(--surface-1); color: var(--text-tertiary); }
.search-field:focus-within { border-color: var(--border-strong); color: var(--text-secondary); }
.search-field input { width: 100%; min-width: 0; border: 0; outline: 0; background: transparent; color: var(--text-primary); font: inherit; font-size: 9px; }
.search-field input::placeholder { color: var(--text-tertiary); }
.search-field button, .list-summary button { display: inline-grid; width: 24px; height: 24px; flex: 0 0 24px; place-items: center; padding: 0; border: 0; background: transparent; color: var(--text-tertiary); cursor: pointer; }
.compact-filters { display: none; }
.list-summary { display: flex; min-width: 0; align-items: center; justify-content: space-between; gap: 8px; padding: 4px 9px; color: var(--text-tertiary); font-size: 8px; }
.entry-scroll { padding: 4px 6px 8px; }
.entry-row { display: grid; width: 100%; min-width: 0; min-height: 70px; grid-template-columns: 30px minmax(0, 1fr) 50px; align-items: center; gap: 8px; margin-bottom: 3px; padding: 7px; border: 1px solid transparent; border-radius: 6px; background: transparent; color: inherit; font: inherit; text-align: left; cursor: pointer; }
.entry-row:hover { background: var(--surface-1); }
.entry-row.selected { border-color: color-mix(in srgb, var(--brand) 34%, var(--border)); background: color-mix(in srgb, var(--brand) 8%, var(--surface-1)); }
.entry-glyph { width: 30px; height: 30px; }
.entry-copy { display: grid; min-width: 0; gap: 2px; }
.entry-title, .entry-id, .entry-excerpt { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.entry-title { color: var(--text-primary); font-size: 9px; font-weight: 800; }
.entry-id { color: var(--text-tertiary); font-family: var(--font-mono); font-size: 7px; }
.entry-excerpt { color: var(--text-secondary); font-size: 8px; }
.entry-meta { min-width: 0; flex-direction: column; align-items: flex-end; gap: 3px; }
.entry-meta small { max-width: 50px; overflow: hidden; color: var(--text-tertiary); font-size: 7px; text-overflow: ellipsis; white-space: nowrap; }
.entry-meta strong { color: var(--brand-light); font-family: var(--font-mono); font-size: 8px; }
.importance-track { width: 42px; height: 3px; overflow: hidden; border-radius: 999px; background: var(--surface-3); }
.importance-track i { display: block; height: 100%; border-radius: inherit; background: var(--brand); }
.panel-empty, .inspector-empty { display: grid; min-height: 100%; place-items: center; align-content: center; gap: 7px; padding: 20px; color: var(--text-tertiary); font-size: 9px; text-align: center; }
.panel-empty strong { color: var(--text-secondary); font-size: 10px; }

.inspector-panel { display: grid; grid-template-rows: 54px minmax(0, 1fr); background: var(--surface-1); }
.inspector-header { display: flex; min-width: 0; align-items: center; justify-content: space-between; gap: 10px; padding: 7px 10px; border-bottom: 1px solid var(--border); }
.inspector-header > div:first-child { display: grid; min-width: 0; gap: 3px; }
.inspector-header h2 { max-width: 440px; font-size: 11px; }
.inspector-actions { flex: 0 0 auto; justify-content: flex-end; gap: 5px; }
.inspector-close { display: none; }
.empty-header { justify-content: flex-start; }
.inspector-scroll { padding: 14px; }

.editor-form { display: grid; align-content: start; gap: 13px; }
.form-grid { display: grid; grid-template-columns: minmax(0, 0.85fr) minmax(0, 1.15fr); gap: 10px; }
.category-grid { grid-template-columns: minmax(0, 1fr) minmax(160px, 0.65fr); }
.form-field { display: grid; min-width: 0; gap: 5px; }
.form-field > span, .section-label { color: var(--text-tertiary); font-size: 8px; font-weight: 800; text-transform: uppercase; }
.input { width: 100%; min-width: 0; min-height: 34px; padding: 7px 8px; border: 1px solid var(--border); border-radius: 6px; outline: 0; background: var(--surface-0); color: var(--text-primary); font: inherit; font-size: 9px; }
.input:focus { border-color: var(--border-strong); box-shadow: var(--shadow-brand); }
.input:disabled { color: var(--text-tertiary); cursor: not-allowed; }
.mono { font-family: var(--font-mono); }
.content-field textarea { min-height: 230px; resize: vertical; line-height: 1.6; }
.importance-field > div { display: grid; min-height: 34px; grid-template-columns: minmax(0, 1fr) 38px; align-items: center; gap: 8px; }
.importance-field input { width: 100%; accent-color: var(--brand); }
.importance-field output { color: var(--brand-light); font-family: var(--font-mono); font-size: 9px; text-align: right; }
.validation-message { gap: 6px; margin: 0; padding-top: 2px; color: var(--warning); font-size: 8px; line-height: 1.45; }

.detail-view { display: grid; align-content: start; gap: 16px; }
.detail-identity { display: grid; min-width: 0; grid-template-columns: 38px minmax(0, 1fr) auto; align-items: center; gap: 10px; padding-bottom: 14px; border-bottom: 1px solid var(--border); }
.detail-icon { width: 38px; height: 38px; }
.detail-identity > div { display: grid; min-width: 0; gap: 4px; }
.detail-identity h3 { margin: 0; overflow: hidden; color: var(--text-primary); font-size: 15px; text-overflow: ellipsis; white-space: nowrap; }
.detail-identity code { overflow: hidden; color: var(--text-tertiary); font-family: var(--font-mono); font-size: 8px; text-overflow: ellipsis; white-space: nowrap; }
.detail-identity > strong { color: var(--brand-light); font-family: var(--font-mono); font-size: 15px; }
.detail-facts { min-width: 0; flex-wrap: wrap; gap: 6px; }
.detail-facts span { min-height: 26px; gap: 5px; padding: 4px 7px; border: 1px solid var(--border); border-radius: 5px; color: var(--text-secondary); font-size: 8px; }
.detail-section { display: grid; gap: 8px; }
.content-section p { margin: 0; color: var(--text-secondary); font-size: 10px; line-height: 1.7; white-space: pre-wrap; overflow-wrap: anywhere; }
.detail-tags { flex-wrap: wrap; gap: 5px; }
.detail-tags span { gap: 3px; padding: 4px 6px; border: 1px solid var(--border); border-radius: 5px; color: var(--text-secondary); font-size: 8px; }
.related-list { display: grid; gap: 4px; }
.related-list button { min-width: 0; min-height: 36px; display: grid; grid-template-columns: 16px minmax(0, 1fr) auto; gap: 7px; padding: 5px 7px; border: 1px solid var(--border); border-radius: 6px; background: var(--surface-0); color: var(--text-secondary); font: inherit; font-size: 9px; text-align: left; cursor: pointer; }
.related-list button:hover { border-color: var(--border-strong); color: var(--text-primary); }
.related-list button span, .related-list button code { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.related-list button code { max-width: 150px; color: var(--text-tertiary); font-family: var(--font-mono); font-size: 7px; }

.modal-backdrop { position: absolute; z-index: 80; inset: 0; display: grid; place-items: center; padding: 16px; background: color-mix(in srgb, var(--surface-0) 82%, transparent); }
.confirm-dialog { display: grid; width: min(390px, 100%); grid-template-columns: 30px minmax(0, 1fr); align-items: start; gap: 9px; padding: 16px; border: 1px solid var(--border-strong); border-radius: 6px; background: var(--surface-1); box-shadow: var(--shadow-lg); }
.confirm-dialog > svg { color: var(--warning); }
.confirm-dialog:has(> svg:first-child:last-of-type) > svg { color: var(--danger); }
.confirm-dialog > div { display: grid; min-width: 0; gap: 4px; }
.confirm-dialog h2 { margin: 0; color: var(--text-primary); font-size: 13px; line-height: 1.45; }
.confirm-dialog footer { grid-column: 1 / -1; justify-content: flex-end; gap: 6px; margin-top: 4px; padding-top: 11px; border-top: 1px solid var(--border); }
.status-toast { position: fixed; z-index: 100; right: 18px; bottom: 18px; max-width: min(440px, calc(100vw - 28px)); padding: 9px 11px; border: 1px solid color-mix(in srgb, var(--success) 38%, var(--border)); border-radius: 6px; background: color-mix(in srgb, var(--success) 18%, var(--surface-1)); color: var(--text-primary); box-shadow: var(--shadow-lg); font: inherit; font-size: 9px; cursor: pointer; }
.status-toast.error { border-color: color-mix(in srgb, var(--danger) 44%, var(--border)); background: color-mix(in srgb, var(--danger) 18%, var(--surface-1)); }
.fade-enter-active, .fade-leave-active { transition: opacity 0.18s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
.spinner { animation: spin 0.8s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }

@media (max-width: 1060px) {
  .knowledge-workbench { grid-template-columns: 300px minmax(0, 1fr); }
  .taxonomy-panel { display: none; }
  .entry-panel { grid-template-rows: 42px 38px 30px minmax(0, 1fr); }
  .compact-filters { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 5px; padding: 4px 7px; border-bottom: 1px solid var(--border); }
  .compact-filters label { min-width: 0; height: 29px; gap: 4px; padding: 0 6px; border: 1px solid var(--border); border-radius: 5px; background: var(--surface-1); color: var(--text-tertiary); }
  .compact-filters select { width: 100%; min-width: 0; border: 0; outline: 0; background: transparent; color: var(--text-secondary); font: inherit; font-size: 8px; }
}

@media (max-width: 760px) {
  .knowledge-workbench { height: calc(100svh - 56px - 60px - env(safe-area-inset-bottom, 0px)); grid-template-columns: 1fr; grid-template-rows: 56px minmax(0, 1fr); }
  .workbench-header { grid-template-columns: minmax(0, 1fr) auto; gap: 8px; padding: 7px 9px; }
  .title-icon { width: 30px; height: 30px; }
  .header-summary { display: none; }
  .entry-panel { grid-row: 2; border-right: 0; }
  .inspector-panel { position: absolute; z-index: 40; inset: 56px 0 0; display: none; border-top: 1px solid var(--border-strong); box-shadow: var(--shadow-lg); }
  .inspector-panel.compact-open { display: grid; }
  .inspector-close { display: inline-flex; }
  .inspector-header h2 { max-width: min(45vw, 220px); }
  .form-grid, .category-grid { grid-template-columns: 1fr; }
  .content-field textarea { min-height: 210px; }
  .status-toast { right: 14px; bottom: calc(70px + env(safe-area-inset-bottom, 0px)); }
}

@media (max-width: 430px) {
  .primary-command > span { display: none; }
  .header-actions .primary-command { width: 34px; padding: 0; }
  .inspector-header { align-items: flex-start; }
  .inspector-actions { gap: 3px; }
  .inspector-actions .secondary-command, .inspector-actions .primary-command { min-width: 34px; padding: 0 7px; font-size: 8px; }
  .inspector-scroll { padding: 11px; }
  .detail-identity { grid-template-columns: 34px minmax(0, 1fr) auto; }
  .detail-icon { width: 34px; height: 34px; }
}
</style>
