<template>
  <div class="kb-workbench">
    <header class="kb-header">
      <div>
        <span class="eyebrow">Knowledge</span>
        <h1>Knowledge Base</h1>
        <p>Manage world lore, character backgrounds, and context entries that feed into AI prompts.</p>
      </div>
      <div class="header-actions">
        <div class="search-bar">
          <input class="input" v-model="searchQuery" placeholder="Search entries..." />
        </div>
        <button class="btn btn-primary btn-sm" @click="createEntry">+ New Entry</button>
      </div>
    </header>

    <div class="kb-layout">
      <aside class="kb-sidebar">
        <div class="filter-group">
          <span class="filter-label">Categories</span>
          <button
            class="filter-btn"
            :class="{ active: selectedCategory === null }"
            @click="selectedCategory = null"
          >All ({{ entries.length }})</button>
          <button
            v-for="cat in categories"
            :key="cat"
            class="filter-btn"
            :class="{ active: selectedCategory === cat }"
            @click="selectedCategory = cat"
          >{{ cat }} ({{ countByCategory(cat) }})</button>
        </div>
        <div class="filter-group">
          <span class="filter-label">Tags</span>
          <div class="tag-cloud">
            <button
              v-for="tag in allTags"
              :key="tag"
              class="tag-chip"
              :class="{ active: selectedTag === tag }"
              @click="selectedTag = selectedTag === tag ? null : tag"
            >{{ tag }}</button>
          </div>
        </div>
      </aside>

      <main class="kb-main">
        <div v-if="editing" class="entry-editor">
          <div class="editor-toolbar">
            <span class="eyebrow">{{ isNewEntry ? 'New Entry' : 'Editing' }}</span>
            <div>
              <button class="btn btn-secondary btn-sm" @click="cancelEdit">Cancel</button>
              <button class="btn btn-primary btn-sm" @click="saveEntry">Save</button>
            </div>
          </div>
          <div class="entry-form">
            <label class="form-field">
              <span>Title</span>
              <input class="input" v-model="editForm.title" placeholder="Entry title" />
            </label>
            <div class="form-row">
              <label class="form-field">
                <span>Category</span>
                <input class="input" v-model="editForm.category" placeholder="e.g. world_lore, character_bg" />
              </label>
              <label class="form-field">
                <span>Keywords (comma-separated)</span>
                <input class="input" v-model="editForm.keywordsStr" placeholder="cherry, blossom, park" />
              </label>
            </div>
            <label class="form-field">
              <span>Content</span>
              <textarea class="input" v-model="editForm.content" rows="10" placeholder="The knowledge content that will be injected into AI prompts..."></textarea>
            </label>
            <label class="form-field">
              <span>Tags (comma-separated)</span>
              <input class="input" v-model="editForm.tagsStr" placeholder="world, nature, history" />
            </label>
          </div>
        </div>

        <div v-else-if="viewingEntry" class="entry-detail">
          <div class="editor-toolbar">
            <span class="eyebrow">Entry Detail</span>
            <div>
              <button class="btn btn-secondary btn-sm" @click="editEntry(viewingEntry)">Edit</button>
              <button class="btn btn-danger btn-sm" @click="deleteEntry(viewingEntry)">Delete</button>
              <button class="btn btn-secondary btn-sm" @click="viewingEntry = null">Back</button>
            </div>
          </div>
          <div class="detail-content">
            <h2>{{ viewingEntry.title }}</h2>
            <div class="detail-meta">
              <span class="meta-chip">{{ viewingEntry.category || 'uncategorized' }}</span>
              <span v-for="tag in (viewingEntry.tags || [])" :key="tag" class="meta-chip tag">{{ tag }}</span>
            </div>
            <div class="detail-keywords" v-if="viewingEntry.keywords?.length">
              <span class="eyebrow">Keywords</span>
              <span v-for="kw in viewingEntry.keywords" :key="kw" class="kw-chip">{{ kw }}</span>
            </div>
            <div class="detail-body">
              {{ viewingEntry.content }}
            </div>
          </div>
        </div>

        <div v-else class="entry-grid">
          <div v-if="filteredEntries.length === 0" class="empty-state">
            <span class="empty-mark">KB</span>
            <h2>No entries found</h2>
            <p>Create knowledge entries to give AI characters context about the world, lore, and backstory.</p>
          </div>
          <article
            v-for="entry in filteredEntries"
            :key="entry.id"
            class="entry-card"
            @click="viewEntry(entry)"
          >
            <div class="card-head">
              <strong>{{ entry.title }}</strong>
              <span class="card-category">{{ entry.category || 'general' }}</span>
            </div>
            <p class="card-body">{{ truncate(entry.content, 120) }}</p>
            <div class="card-footer">
              <span v-for="kw in (entry.keywords || []).slice(0, 3)" :key="kw" class="kw-chip">{{ kw }}</span>
            </div>
          </article>
        </div>
      </main>
    </div>

    <Transition name="fade">
      <div v-if="statusMsg" class="status-toast" :class="{ error: !statusOk }" @click="statusMsg = null">
        {{ statusMsg }}
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import { invokeCommand } from '../lib/tauri'

interface KnowledgeEntry {
  id: string
  title: string
  content: string
  category: string
  keywords: string[]
  tags: string[]
}

interface EditForm {
  title: string
  content: string
  category: string
  keywordsStr: string
  tagsStr: string
}

const entries = ref<KnowledgeEntry[]>([])
const searchQuery = ref('')
const selectedCategory = ref<string | null>(null)
const selectedTag = ref<string | null>(null)
const editing = ref(false)
const isNewEntry = ref(false)
const viewingEntry = ref<KnowledgeEntry | null>(null)
const statusMsg = ref<string | null>(null)
const statusOk = ref(true)

const editForm = ref<EditForm>({
  title: '', content: '', category: '', keywordsStr: '', tagsStr: ''
})

const categories = computed(() => {
  const cats = new Set(entries.value.map(e => e.category).filter(Boolean))
  return Array.from(cats).sort()
})

const allTags = computed(() => {
  const tags = new Set<string>()
  for (const entry of entries.value) {
    for (const tag of entry.tags || []) tags.add(tag)
  }
  return Array.from(tags).sort()
})

const filteredEntries = computed(() => {
  let result = entries.value
  if (selectedCategory.value) {
    result = result.filter(e => e.category === selectedCategory.value)
  }
  if (selectedTag.value) {
    result = result.filter(e => (e.tags || []).includes(selectedTag.value!))
  }
  if (searchQuery.value.trim()) {
    const q = searchQuery.value.toLowerCase()
    result = result.filter(e =>
      e.title.toLowerCase().includes(q) ||
      e.content.toLowerCase().includes(q) ||
      (e.keywords || []).some(k => k.toLowerCase().includes(q))
    )
  }
  return result
})

function countByCategory(cat: string): number {
  return entries.value.filter(e => e.category === cat).length
}

function truncate(text: string, len: number): string {
  if (!text) return ''
  return text.length > len ? text.slice(0, len) + '...' : text
}

async function loadEntries() {
  try {
    const result = await invokeCommand<KnowledgeEntry[]>('list_knowledge_entries', undefined, [])
    entries.value = result
  } catch (e) {
    console.error('Failed to load knowledge:', e)
    entries.value = []
  }
}

function createEntry() {
  editForm.value = { title: '', content: '', category: 'world_lore', keywordsStr: '', tagsStr: '' }
  isNewEntry.value = true
  editing.value = true
  viewingEntry.value = null
}

function editEntry(entry: KnowledgeEntry) {
  editForm.value = {
    title: entry.title,
    content: entry.content,
    category: entry.category,
    keywordsStr: (entry.keywords || []).join(', '),
    tagsStr: (entry.tags || []).join(', '),
  }
  isNewEntry.value = false
  editing.value = true
}

function cancelEdit() {
  editing.value = false
}

function viewEntry(entry: KnowledgeEntry) {
  viewingEntry.value = entry
  editing.value = false
}

async function saveEntry() {
  statusMsg.value = 'Knowledge entry saved (requires backend knowledge store)'
  statusOk.value = true
  editing.value = false
}

async function deleteEntry(entry: KnowledgeEntry) {
  if (!confirm('Delete "' + entry.title + '"?')) return
  statusMsg.value = 'Entry deleted (requires backend support)'
  statusOk.value = true
  viewingEntry.value = null
  await loadEntries()
}

onMounted(loadEntries)
</script>

<style scoped>
.kb-workbench {
  max-width: 1280px;
  margin: 0 auto;
  padding: 34px 40px;
}

.kb-header {
  display: flex;
  justify-content: space-between;
  gap: 18px;
  align-items: flex-start;
  margin-bottom: 24px;
}

.kb-header h1 {
  color: var(--text-primary);
  font-size: 28px;
  line-height: 1.15;
  margin-top: 3px;
}

.kb-header p {
  color: var(--text-tertiary);
  font-size: 13px;
  margin-top: 4px;
}

.eyebrow {
  display: block;
  color: var(--text-tertiary);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0;
  text-transform: uppercase;
}

.header-actions {
  display: flex;
  gap: 10px;
  align-items: center;
  flex-shrink: 0;
}

.search-bar .input {
  min-width: 240px;
}

.kb-layout {
  display: grid;
  grid-template-columns: 220px minmax(0, 1fr);
  gap: 18px;
  align-items: start;
}

.kb-sidebar {
  position: sticky;
  top: 18px;
  display: grid;
  gap: 20px;
  padding: 16px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-1);
}

.filter-label {
  display: block;
  color: var(--text-tertiary);
  font-size: 10px;
  font-weight: 800;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 8px;
}

.filter-btn {
  display: block;
  width: 100%;
  padding: 7px 10px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 12px;
  font-weight: 600;
  text-align: left;
  transition: background var(--transition-fast);
}

.filter-btn:hover {
  background: var(--surface-2);
}

.filter-btn.active {
  background: var(--surface-3);
  color: var(--brand-light);
  font-weight: 700;
}

.tag-cloud {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.tag-chip {
  padding: 3px 10px;
  border: 1px solid var(--border);
  border-radius: 999px;
  background: var(--surface-2);
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 11px;
  font-weight: 600;
}

.tag-chip:hover, .tag-chip.active {
  border-color: var(--brand);
  color: var(--brand-light);
}

.kb-main {
  min-width: 0;
}

.editor-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 14px 18px;
  margin-bottom: 16px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-1);
}

.editor-toolbar .eyebrow {
  margin-bottom: 2px;
}

.entry-form {
  display: grid;
  gap: 16px;
  padding: 20px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-1);
}

.form-field {
  display: grid;
  gap: 6px;
}

.form-field span {
  color: var(--text-secondary);
  font-size: 12px;
  font-weight: 700;
}

.form-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.entry-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 14px;
}

.entry-card {
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 16px;
  background: var(--surface-1);
  cursor: pointer;
  transition: border-color var(--transition-fast), background var(--transition-fast);
}

.entry-card:hover {
  border-color: var(--brand);
  background: var(--surface-2);
}

.card-head {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  gap: 10px;
  margin-bottom: 8px;
}

.card-head strong {
  color: var(--text-primary);
  font-size: 14px;
}

.card-category {
  color: var(--text-tertiary);
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  padding: 2px 8px;
  border: 1px solid var(--border);
  border-radius: 999px;
  flex-shrink: 0;
}

.card-body {
  color: var(--text-secondary);
  font-size: 12px;
  line-height: 1.5;
  margin-bottom: 10px;
}

.card-footer {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.kw-chip {
  padding: 2px 8px;
  border-radius: var(--radius-sm);
  background: var(--surface-3);
  color: var(--brand-light);
  font-size: 10px;
  font-weight: 700;
}

.detail-content {
  padding: 20px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-1);
}

.detail-content h2 {
  color: var(--text-primary);
  font-size: 20px;
  margin-bottom: 12px;
}

.detail-meta {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
  margin-bottom: 14px;
}

.meta-chip {
  padding: 3px 10px;
  border: 1px solid var(--border);
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  color: var(--text-secondary);
}

.meta-chip.tag {
  border-color: var(--brand);
  color: var(--brand-light);
}

.detail-keywords {
  margin-bottom: 16px;
}

.detail-keywords .eyebrow {
  margin-bottom: 8px;
}

.detail-keywords .kw-chip {
  margin-right: 6px;
  margin-bottom: 4px;
  display: inline-block;
}

.detail-body {
  color: var(--text-primary);
  font-size: 14px;
  line-height: 1.8;
  white-space: pre-wrap;
  border-top: 1px solid var(--border);
  padding-top: 16px;
}

.empty-state {
  grid-column: 1 / -1;
  text-align: center;
  padding: 60px 20px;
}

.empty-state h2 {
  color: var(--text-primary);
  font-size: 22px;
  margin-top: 16px;
}

.empty-state p {
  color: var(--text-tertiary);
  font-size: 13px;
  margin-top: 8px;
  max-width: 480px;
  margin-left: auto;
  margin-right: auto;
}

.empty-mark {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 44px;
  height: 44px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-2);
  color: var(--brand-light);
  font-family: var(--font-mono);
  font-weight: 900;
}

.status-toast {
  position: fixed;
  bottom: 20px;
  left: 50%;
  transform: translateX(-50%);
  min-width: 300px;
  padding: 12px 18px;
  border: 1px solid rgba(45,212,191,0.36);
  border-radius: var(--radius);
  background: rgba(15,118,110,0.96);
  color: white;
  font-size: 13px;
  font-weight: 600;
  text-align: center;
  z-index: 100;
  box-shadow: var(--shadow-lg);
  cursor: pointer;
}

.status-toast.error {
  border-color: rgba(239,68,68,0.42);
  background: rgba(127,29,29,0.96);
}

.fade-enter-active, .fade-leave-active { transition: opacity 0.2s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }

@media (max-width: 860px) {
  .kb-layout {
    grid-template-columns: 1fr;
  }
  .kb-sidebar {
    position: static;
  }
  .form-row {
    grid-template-columns: 1fr;
  }
}
</style>
