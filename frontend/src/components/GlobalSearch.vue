<template>
  <div class="global-search">
    <button
      class="search-trigger"
      data-testid="global-search-trigger"
      :title="t('search.open', 'Search workspace (Ctrl+K)')"
      @click="openSearch"
    >
      <Search :size="16" aria-hidden="true" />
      <span>{{ t('search.label', 'Search') }}</span>
      <kbd>Ctrl K</kbd>
    </button>

    <Teleport to="body">
      <Transition name="search-fade">
        <div v-if="isExpanded" class="search-overlay" @mousedown.self="closeSearch">
          <section class="search-dialog" role="dialog" aria-modal="true" :aria-label="t('search.title', 'Search workspace')">
            <header class="search-header">
              <Search :size="18" aria-hidden="true" />
              <input
                ref="inputRef"
                v-model="query"
                :placeholder="t('search.placeholder', 'Search characters, knowledge, and dialogues')"
                :aria-label="t('search.title', 'Search workspace')"
                @keydown.escape="closeSearch"
              />
              <button class="close-button" :title="t('common.close', 'Close')" @click="closeSearch">
                <X :size="17" aria-hidden="true" />
              </button>
            </header>

            <div class="search-body">
              <div v-if="!query" class="search-hint">
                <Command :size="22" aria-hidden="true" />
                <strong>{{ t('search.start-title', 'Find anything in your project') }}</strong>
                <p>{{ t('search.start-copy', 'Search authored characters, lore entries, and dialogue graphs from one place.') }}</p>
              </div>

              <template v-else>
                <section v-if="characterResults.length" class="result-group">
                  <span class="result-label">{{ t('nav.characters', 'Characters') }}</span>
                  <button v-for="character in characterResults" :key="character.id" class="result-item" @click="navigate('/characters')">
                    <UserRound :size="17" aria-hidden="true" />
                    <span>
                      <strong>{{ character.name }}</strong>
                      <small>{{ character.description || t('search.character-result', 'Character') }}</small>
                    </span>
                  </button>
                </section>

                <section v-if="knowledgeResults.length" class="result-group">
                  <span class="result-label">{{ t('nav.knowledge', 'Knowledge') }}</span>
                  <button v-for="entry in knowledgeResults" :key="entry.id" class="result-item" @click="navigate('/knowledge')">
                    <Library :size="17" aria-hidden="true" />
                    <span>
                      <strong>{{ entry.title }}</strong>
                      <small>{{ entry.category || t('search.knowledge-result', 'Knowledge entry') }}</small>
                    </span>
                  </button>
                </section>

                <section v-if="dialogueResults.length" class="result-group">
                  <span class="result-label">{{ t('nav.dialogues', 'Dialogues') }}</span>
                  <button v-for="dialogue in dialogueResults" :key="dialogue.id" class="result-item" @click="navigate('/dialogue-editor')">
                    <MessageSquareText :size="17" aria-hidden="true" />
                    <span>
                      <strong>{{ dialogue.title }}</strong>
                      <small>{{ t('search.dialogue-result', 'Dialogue graph') }}</small>
                    </span>
                  </button>
                </section>

                <div v-if="noResults" class="search-hint compact">
                  <SearchX :size="22" aria-hidden="true" />
                  <strong>{{ t('search.no-results', 'No results for “{query}”', { query }) }}</strong>
                  <p>{{ t('search.no-results-copy', 'Try a character name, lore title, or dialogue title.') }}</p>
                </div>
              </template>
            </div>

            <footer class="search-footer">
              <span><kbd>Esc</kbd> {{ t('search.close-hint', 'to close') }}</span>
              <span>{{ t('search.scope', 'Project search') }}</span>
            </footer>
          </section>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { Command, Library, MessageSquareText, Search, SearchX, UserRound, X } from '@lucide/vue'
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

const normalizedQuery = computed(() => query.value.trim().toLocaleLowerCase())
const characterResults = computed(() => normalizedQuery.value
  ? characters.value.filter((item) => [item.name, item.description].some((value) => value?.toLocaleLowerCase().includes(normalizedQuery.value))).slice(0, 5)
  : [])
const knowledgeResults = computed(() => normalizedQuery.value
  ? knowledge.value.filter((item) => [item.title, item.content, item.category].some((value) => value?.toLocaleLowerCase().includes(normalizedQuery.value))).slice(0, 5)
  : [])
const dialogueResults = computed(() => normalizedQuery.value
  ? dialogues.value.filter((item) => item.title?.toLocaleLowerCase().includes(normalizedQuery.value)).slice(0, 5)
  : [])
const noResults = computed(() => Boolean(normalizedQuery.value)
  && characterResults.value.length === 0
  && knowledgeResults.value.length === 0
  && dialogueResults.value.length === 0)

async function loadData() {
  const [nextCharacters, nextKnowledge, nextDialogues] = await Promise.all([
    invokeCommand<any[]>('get_characters', undefined, []).catch(() => []),
    invokeCommand<any[]>('load_knowledge', undefined, []).catch(() => []),
    invokeCommand<any[]>('load_dialogues', undefined, []).catch(() => []),
  ])
  characters.value = nextCharacters
  knowledge.value = nextKnowledge
  dialogues.value = nextDialogues
}

function openSearch() {
  isExpanded.value = true
  query.value = ''
  void loadData()
  void nextTick(() => inputRef.value?.focus())
}

function closeSearch() {
  isExpanded.value = false
  query.value = ''
}

function navigate(path: string) {
  void router.push(path)
  closeSearch()
}

function handleShortcut(event: KeyboardEvent) {
  if ((event.ctrlKey || event.metaKey) && event.key.toLocaleLowerCase() === 'k') {
    event.preventDefault()
    isExpanded.value ? closeSearch() : openSearch()
  }
  if (event.key === 'Escape' && isExpanded.value) closeSearch()
}

onMounted(() => window.addEventListener('keydown', handleShortcut))
onUnmounted(() => window.removeEventListener('keydown', handleShortcut))
</script>

<style scoped>
.search-trigger {
  display: flex;
  height: 34px;
  align-items: center;
  gap: 7px;
  padding: 0 8px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-1);
  color: var(--text-secondary);
  cursor: pointer;
  font: inherit;
  font-size: 11px;
}
.search-trigger:hover { border-color: var(--border-strong); color: var(--text-primary); }
kbd { padding: 2px 5px; border: 1px solid var(--border); border-radius: 4px; background: var(--surface-0); color: var(--text-tertiary); font: 9px/1.3 var(--font-mono); }
.search-overlay { position: fixed; inset: 0; z-index: 300; display: grid; place-items: start center; padding: 12vh 16px 16px; background: rgba(0, 0, 0, 0.58); }
.search-dialog { width: min(620px, 100%); overflow: hidden; border: 1px solid var(--border-strong); border-radius: var(--radius); background: var(--surface-1); box-shadow: var(--shadow-lg); }
.search-header { display: grid; min-height: 54px; grid-template-columns: auto minmax(0, 1fr) auto; align-items: center; gap: 10px; padding: 8px 10px 8px 16px; border-bottom: 1px solid var(--border); color: var(--text-tertiary); }
.search-header input { width: 100%; border: 0; outline: 0; background: transparent; color: var(--text-primary); font: inherit; font-size: 14px; }
.search-header input::placeholder { color: var(--text-tertiary); }
.close-button { display: grid; width: 32px; height: 32px; place-items: center; border: 0; border-radius: var(--radius-sm); background: transparent; color: var(--text-tertiary); cursor: pointer; }
.close-button:hover { background: var(--surface-2); color: var(--text-primary); }
.search-body { max-height: min(480px, 62vh); overflow-y: auto; padding: 8px; }
.search-hint { display: grid; min-height: 220px; place-items: center; align-content: center; gap: 8px; padding: 28px; text-align: center; }
.search-hint svg { color: var(--brand); }
.search-hint strong { color: var(--text-primary); font-size: 13px; }
.search-hint p { max-width: 360px; color: var(--text-tertiary); font-size: 11px; }
.search-hint.compact { min-height: 170px; }
.result-group { display: grid; gap: 3px; padding: 4px; }
.result-group + .result-group { margin-top: 5px; padding-top: 9px; border-top: 1px solid var(--border-subtle); }
.result-label { padding: 4px 8px; color: var(--text-tertiary); font-size: 9px; font-weight: 750; text-transform: uppercase; }
.result-item { display: grid; width: 100%; grid-template-columns: 24px minmax(0, 1fr); align-items: center; gap: 8px; padding: 8px; border: 0; border-radius: var(--radius-sm); background: transparent; color: var(--text-secondary); cursor: pointer; text-align: left; }
.result-item:hover { background: var(--surface-2); color: var(--text-primary); }
.result-item > span { display: grid; min-width: 0; }
.result-item strong, .result-item small { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.result-item strong { color: var(--text-primary); font-size: 12px; }
.result-item small { margin-top: 2px; color: var(--text-tertiary); font-size: 10px; }
.search-footer { display: flex; min-height: 34px; align-items: center; justify-content: space-between; padding: 6px 12px; border-top: 1px solid var(--border); color: var(--text-tertiary); font-size: 9px; }
.search-footer span { display: flex; align-items: center; gap: 5px; }
.search-fade-enter-active, .search-fade-leave-active { transition: opacity 120ms ease; }
.search-fade-enter-from, .search-fade-leave-to { opacity: 0; }
@media (max-width: 680px) {
  .search-trigger span, .search-trigger kbd { display: none; }
  .search-trigger { width: 34px; justify-content: center; padding: 0; }
  .search-overlay { padding-top: 64px; }
}
</style>
