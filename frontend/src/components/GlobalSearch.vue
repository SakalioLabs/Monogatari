<template>
  <div class="global-search" :class="{ expanded: isExpanded }">
    <button class="search-trigger" @click="toggleSearch" title="Search (Ctrl+K)">
      <span v-if="!isExpanded">&#128269;</span>
      <span v-else class="close-icon">&times;</span>
    </button>

    <Transition name="slide">
      <div v-if="isExpanded" class="search-panel">
        <div class="search-input-wrap">
          <input
            ref="inputRef"
            v-model="query"
            class="search-input"
            placeholder="Search characters, knowledge, dialogues..."
            @input="doSearch"
            @keydown.escape="closeSearch"
          />
          <span class="search-shortcut">ESC</span>
        </div>

        <div v-if="query.length > 0" class="search-results">
          <section v-if="characterResults.length > 0" class="result-group">
            <span class="result-label">{{ t('nav.characters', 'Characters') }}</span>
            <div v-for="c in characterResults" :key="c.id" class="result-item" @click="navigate('/characters')">
              <span class="result-icon">&#9786;</span>
              <span class="result-name">{{ c.name }}</span>
              <span class="result-desc">{{ c.description }}</span>
            </div>
          </section>

          <section v-if="knowledgeResults.length > 0" class="result-group">
            <span class="result-label">{{ t('nav.knowledge', 'Knowledge') }}</span>
            <div v-for="k in knowledgeResults" :key="k.id" class="result-item" @click="navigate('/knowledge')">
              <span class="result-icon">&#128218;</span>
              <span class="result-name">{{ k.title }}</span>
              <span class="result-desc">{{ k.category }}</span>
            </div>
          </section>

          <section v-if="dialogueResults.length > 0" class="result-group">
            <span class="result-label">{{ t('nav.dialogues', 'Dialogues') }}</span>
            <div v-for="d in dialogueResults" :key="d.id" class="result-item" @click="navigate('/dialogue-editor')">
              <span class="result-icon">&#128172;</span>
              <span class="result-name">{{ d.title }}</span>
            </div>
          </section>

          <div v-if="noResults" class="no-results">
            <span>No results found for "{{ query }}"</span>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from '../lib/i18n'
import { invokeCommand } from '../lib/tauri'

const { t } = useI18n()
const router = useRouter()
const isExpanded = ref(false)
const query = ref('')
const inputRef = ref<HTMLInputElement>()
const characters = ref<any[]>([])
const knowledge = ref<any[]>([])
const dialogues = ref<any[]>([])

const characterResults = computed(() => {
  if (!query.value) return []
  const q = query.value.toLowerCase()
  return characters.value.filter(c => c.name?.toLowerCase().includes(q) || c.description?.toLowerCase().includes(q))
})

const knowledgeResults = computed(() => {
  if (!query.value) return []
  const q = query.value.toLowerCase()
  return knowledge.value.filter(k => k.title?.toLowerCase().includes(q) || k.content?.toLowerCase().includes(q))
})

const dialogueResults = computed(() => {
  if (!query.value) return []
  const q = query.value.toLowerCase()
  return dialogues.value.filter(d => d.title?.toLowerCase().includes(q))
})

const noResults = computed(() =>
  query.value.length > 0 &&
  characterResults.value.length === 0 &&
  knowledgeResults.value.length === 0 &&
  dialogueResults.value.length === 0
)

async function loadData() {
  try { characters.value = await invokeCommand<any[]>('get_characters', undefined, []) } catch {}
  try { knowledge.value = await invokeCommand<any[]>('load_knowledge', undefined, []) } catch {}
  try { dialogues.value = await invokeCommand<any[]>('load_dialogues', undefined, []) } catch {}
}

function toggleSearch() {
  isExpanded.value = !isExpanded.value
  if (isExpanded.value) {
    query.value = ''
    nextTick(() => inputRef.value?.focus())
    loadData()
  }
}

function closeSearch() {
  isExpanded.value = false
  query.value = ''
}

function navigate(path: string) {
  router.push(path)
  closeSearch()
}

function doSearch() {}

// Listen for Ctrl+K
if (typeof window !== 'undefined') {
  window.addEventListener('keydown', (e: KeyboardEvent) => {
    if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
      e.preventDefault()
      toggleSearch()
    }
  })
}
</script>

<style scoped>
.global-search { position: relative; }
.search-trigger {
  width: 34px; height: 34px;
  border: 1px solid var(--border); border-radius: var(--radius-sm);
  background: var(--surface-1); color: var(--text-secondary);
  cursor: pointer; font-size: 16px; line-height: 1;
  transition: all 0.15s;
}
.search-trigger:hover { border-color: var(--brand); color: var(--brand-light); }
.close-icon { font-size: 20px; }
.search-panel {
  position: absolute; top: calc(100% + 8px); right: 0;
  width: 400px; max-height: 480px;
  border: 1px solid var(--border); border-radius: var(--radius);
  background: var(--surface-1); box-shadow: var(--shadow-lg); z-index: 50;
  overflow: hidden;
}
.search-input-wrap {
  display: flex; align-items: center; gap: 8px;
  padding: 12px; border-bottom: 1px solid var(--border);
}
.search-input {
  flex: 1; border: none; background: none; color: var(--text-primary);
  font: inherit; font-size: 14px; outline: none;
}
.search-input::placeholder { color: var(--text-tertiary); }
.search-shortcut {
  padding: 2px 6px; border: 1px solid var(--border); border-radius: 4px;
  font-size: 10px; color: var(--text-tertiary); font-weight: 700;
}
.search-results { overflow-y: auto; max-height: 400px; }
.result-group { padding: 8px; display: grid; gap: 4px; }
.result-label {
  display: block; padding: 4px 8px;
  font-size: 11px; font-weight: 800; color: var(--text-tertiary);
  text-transform: uppercase;
}
.result-item {
  display: flex; gap: 10px; align-items: center;
  padding: 8px; border-radius: var(--radius-sm);
  cursor: pointer; transition: background 0.1s;
}
.result-item:hover { background: var(--surface-2); }
.result-icon { font-size: 14px; width: 20px; text-align: center; }
.result-name { font-size: 13px; font-weight: 700; color: var(--text-primary); white-space: nowrap; }
.result-desc { font-size: 12px; color: var(--text-tertiary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.no-results { padding: 24px; text-align: center; color: var(--text-tertiary); font-size: 13px; }
.slide-enter-active, .slide-leave-active { transition: all 0.15s ease; }
.slide-enter-from, .slide-leave-to { opacity: 0; transform: translateY(-8px); }
</style>
