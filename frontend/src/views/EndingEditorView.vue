<template>
  <div class="ending-editor">
    <header class="editor-header">
      <div class="header-copy">
        <span class="eyebrow">{{ t('ending.editor-eyebrow', 'Story Design') }}</span>
        <h1>{{ t('ending.routes-title', 'Ending Routes') }}</h1>
        <p>{{ t('ending.catalog-summary', '{total} endings · {gated} event-gated · {open} open', {
          total: snapshot?.ending_count || 0,
          gated: gatedCount,
          open: openCount,
        }) }}</p>
      </div>
      <div class="header-actions">
        <button class="btn btn-secondary btn-sm" :disabled="busy" @click="createEnding"><Plus :size="14" />{{ t('authoring.new', 'New') }}</button>
        <button class="btn btn-secondary btn-sm" :disabled="!canDuplicate || busy" @click="duplicateEnding"><Copy :size="14" />{{ t('authoring.duplicate', 'Duplicate') }}</button>
        <button class="btn btn-secondary btn-sm" :disabled="busy" @click="reloadCatalog"><RotateCcw :size="14" />{{ t('authoring.reload', 'Reload') }}</button>
        <button class="btn btn-secondary btn-sm" :disabled="!canPreview || busy" @click="previewEnding"><Play :size="14" />{{ t('common.preview', 'Preview') }}</button>
        <button class="btn btn-primary btn-sm" :disabled="!canSave || busy" @click="saveEnding">
          <Save :size="14" />{{ busy ? t('authoring.working', 'Working') : t('common.save', 'Save') }}
        </button>
      </div>
    </header>

    <div class="catalog-strip">
      <span><strong>{{ scenes.length }}</strong> {{ t('ending.scenes', 'scenes') }}</span>
      <span><strong>{{ dialogues.length }}</strong> {{ t('ending.dialogues', 'dialogues') }}</span>
      <span><strong>{{ snapshot?.catalog_fingerprint.slice(0, 12) || t('authoring.unavailable', 'unavailable') }}</strong> {{ t('authoring.catalog', 'catalog') }}</span>
      <span v-if="dirty" class="dirty-indicator">{{ t('authoring.unsaved-changes', 'Unsaved changes') }}</span>
    </div>

    <main class="editor-workspace">
      <aside class="ending-list" :aria-label="t('ending.catalog-aria', 'Ending catalog')">
        <div class="list-toolbar">
          <label class="search-field">
            <span class="sr-only">{{ t('ending.search', 'Search endings') }}</span>
            <input v-model.trim="search" class="input" type="search" :placeholder="t('ending.search', 'Search endings')" />
          </label>
          <span>{{ filteredEndings.length }}</span>
        </div>

        <div class="ending-items">
          <button
            v-for="ending in filteredEndings"
            :key="ending.id"
            class="ending-item"
            :class="{ active: originalEndingId === ending.id }"
            @click="selectEnding(ending)"
          >
            <span class="ending-item-copy">
              <strong>{{ ending.title }}</strong>
              <small>{{ ending.id }}</small>
            </span>
            <span class="access-dot" :class="ending.access.gated ? 'gated' : 'open'">
              {{ ending.access.gated ? t('authoring.gated-label', 'Gated') : t('authoring.open', 'Open') }}
            </span>
          </button>
          <div v-if="filteredEndings.length === 0" class="empty-list">{{ t('ending.no-endings', 'No endings') }}</div>
        </div>
      </aside>

      <section v-if="draft" class="ending-form">
        <div class="route-map" :aria-label="t('ending.route-association', 'Ending route association')">
          <div class="route-step scene-step">
            <span class="step-number">1</span>
            <span class="step-copy">
              <small>{{ t('ending.scene', 'Scene') }}</small>
              <strong>{{ selectedScene?.name || draft.scene_id || t('authoring.not-selected', 'Not selected') }}</strong>
              <span>{{ selectedScene?.id || t('ending.missing-association', 'Missing association') }}</span>
            </span>
          </div>
          <span class="route-connector" aria-hidden="true"></span>
          <div class="route-step dialogue-step">
            <span class="step-number">2</span>
            <span class="step-copy">
              <small>{{ t('ending.dialogue', 'Dialogue') }}</small>
              <strong>{{ selectedDialogue?.title || draft.dialogue_id || t('authoring.not-selected', 'Not selected') }}</strong>
              <span>{{ t('ending.nodes-count', '{count} nodes', { count: selectedDialogue?.node_count || 0 }) }}</span>
            </span>
          </div>
          <span class="route-connector" aria-hidden="true"></span>
          <div class="route-step ending-step">
            <span class="step-number">3</span>
            <span class="step-copy">
              <small>{{ t('ending.ending', 'Ending') }}</small>
              <strong>{{ draft.title || t('ending.untitled', 'Untitled ending') }}</strong>
              <span>{{ draft.id || t('ending.missing-id', 'Missing ID') }}</span>
            </span>
          </div>
        </div>

        <div v-if="validationIssues.length" class="validation-banner error" role="alert">
          <strong>{{ t('authoring.blocking-issues', '{count} blocking issues', { count: validationIssues.length }) }}</strong>
          <span>{{ validationIssues[0] }}</span>
        </div>
        <div v-else-if="warnings.length" class="validation-banner warning">
          <strong>{{ t('ending.route-warnings', '{count} route warnings', { count: warnings.length }) }}</strong>
          <span>{{ warnings[0] }}</span>
        </div>
        <div v-else class="validation-banner valid">
          <strong>{{ t('ending.route-valid', 'Route valid') }}</strong>
          <span>{{ t('ending.route-valid-copy', 'Scene, dialogue, and ending references are consistent.') }}</span>
        </div>

        <div class="form-scroll">
          <section class="form-section identity-section">
            <div class="section-heading">
              <div>
                <span class="eyebrow">{{ t('authoring.identity', 'Identity') }}</span>
                <h2>{{ t('ending.definition', 'Ending definition') }}</h2>
              </div>
              <span class="source-path">{{ sourcePath }}</span>
            </div>
            <div class="field-grid">
              <label class="form-field id-field">
                <span>{{ t('ending.id', 'Ending ID') }}</span>
                <input v-model.trim="draft.id" class="input mono" :disabled="originalEndingId !== null" />
              </label>
              <label class="form-field title-field">
                <span>{{ t('ending.title-label', 'Title') }}</span>
                <input v-model="draft.title" class="input" maxlength="256" />
              </label>
              <label class="form-field description-field">
                <span>{{ t('common.description', 'Description') }}</span>
                <textarea v-model="draft.description" class="input" rows="4" maxlength="2048"></textarea>
                <small>{{ draft.description.trim().length }} / 2048</small>
              </label>
            </div>
          </section>

          <section class="form-section association-section">
            <div class="section-heading">
              <div>
                <span class="eyebrow">{{ t('ending.stage', 'Stage') }}</span>
                <h2>{{ t('ending.scene-association', 'Scene association') }}</h2>
              </div>
              <span class="state-label" :class="selectedScene?.access.unlocked ? 'ready' : 'locked'">
                {{ selectedScene ? accessLabel(selectedScene.access) : t('authoring.missing', 'Missing') }}
              </span>
            </div>
            <label class="form-field">
              <span>{{ t('ending.scene', 'Scene') }}</span>
              <select v-model="draft.scene_id" class="input">
                <option value="">{{ t('ending.select-scene', 'Select scene') }}</option>
                <option v-for="scene in scenes" :key="scene.id" :value="scene.id">
                  {{ scene.name }} · {{ scene.id }}
                </option>
              </select>
            </label>
            <div v-if="selectedScene" class="reference-details">
              <span><strong>{{ t('scene.background', 'Background') }}</strong>{{ selectedScene.background_path || t('authoring.not-set', 'Not set') }}</span>
              <span><strong>{{ t('ending.time', 'Time') }}</strong>{{ selectedScene.time_of_day || t('ending.any', 'Any') }}</span>
              <span><strong>{{ t('scene.tags', 'Tags') }}</strong>{{ selectedScene.tags.join(', ') || t('authoring.none', 'None') }}</span>
            </div>
          </section>

          <section class="form-section association-section">
            <div class="section-heading">
              <div>
                <span class="eyebrow">{{ t('ending.sequence', 'Sequence') }}</span>
                <h2>{{ t('ending.dialogue-association', 'Dialogue association') }}</h2>
              </div>
              <span class="state-label" :class="selectedDialogue?.access.unlocked ? 'ready' : 'locked'">
                {{ selectedDialogue ? accessLabel(selectedDialogue.access) : t('authoring.missing', 'Missing') }}
              </span>
            </div>
            <label class="form-field">
              <span>{{ t('ending.dialogue', 'Dialogue') }}</span>
              <select v-model="draft.dialogue_id" class="input">
                <option value="">{{ t('ending.select-dialogue', 'Select dialogue') }}</option>
                <option v-for="dialogue in dialogues" :key="dialogue.id" :value="dialogue.id">
                  {{ dialogue.title }} · {{ dialogue.id }}
                </option>
              </select>
            </label>
            <div v-if="selectedDialogue" class="reference-details">
              <span><strong>{{ t('ending.start-node', 'Start node') }}</strong>{{ selectedDialogue.start_node_id }}</span>
              <span><strong>{{ t('ending.node-count', 'Node count') }}</strong>{{ selectedDialogue.node_count }}</span>
              <span><strong>{{ t('ending.runtime', 'Runtime') }}</strong>{{ accessLabel(selectedDialogue.access) }}</span>
            </div>
          </section>

          <section class="form-section coverage-section">
            <div class="section-heading">
              <div>
                <span class="eyebrow">{{ t('ending.progression', 'Progression') }}</span>
                <h2>{{ t('ending.unlock-coverage', 'Unlock coverage') }}</h2>
              </div>
              <button class="btn btn-secondary btn-sm" @click="router.push('/story-events')">{{ t('nav.events', 'Story Events') }}</button>
            </div>
            <div class="coverage-grid">
              <div class="coverage-row">
                <span>{{ t('ending.ending', 'Ending') }}</span>
                <strong>{{ endingEvents.join(', ') || t('ending.available-from-start', 'Available from start') }}</strong>
              </div>
              <div class="coverage-row">
                <span>{{ t('ending.scene', 'Scene') }}</span>
                <strong>{{ selectedScene?.access.unlock_event_ids.join(', ') || t('ending.available-from-start', 'Available from start') }}</strong>
              </div>
              <div class="coverage-row">
                <span>{{ t('ending.dialogue', 'Dialogue') }}</span>
                <strong>{{ selectedDialogue?.access.unlock_event_ids.join(', ') || t('ending.available-from-start', 'Available from start') }}</strong>
              </div>
            </div>
          </section>
        </div>

        <footer class="form-footer">
          <div class="footer-status">
            <span :class="validationIssues.length ? 'invalid-text' : 'valid-text'">
              {{ validationIssues.length ? t('authoring.issues-count', '{count} issues', { count: validationIssues.length }) : t('authoring.ready-to-save', 'Ready to save') }}
            </span>
            <small>{{ originalEndingId ? t('authoring.existing-asset', 'Existing asset') : t('authoring.new-asset', 'New asset') }}</small>
          </div>
          <div class="footer-actions">
            <button class="btn btn-danger btn-sm" :disabled="originalEndingId === null || busy" @click="removeEnding"><Trash2 :size="14" />{{ t('common.delete', 'Delete') }}</button>
            <button class="btn btn-primary" :disabled="!canSave || busy" @click="saveEnding"><Save :size="15" />{{ t('ending.save-ending', 'Save Ending') }}</button>
          </div>
        </footer>
      </section>

      <section v-else class="empty-editor">
        <span class="empty-mark">ER</span>
        <h2>{{ t('ending.no-selection', 'No ending selected') }}</h2>
      </section>
    </main>

    <Transition name="fade">
      <button v-if="notice" class="notice" :class="notice.type" @click="notice = null">
        <strong>{{ notice.title }}</strong>
        <span>{{ notice.message }}</span>
      </button>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { onBeforeRouteLeave, useRouter } from 'vue-router'
import { Copy, Play, Plus, RotateCcw, Save, Trash2 } from '@lucide/vue'
import { useI18n } from '../lib/i18n'
import { hasTauriRuntime, invokeCommand } from '../lib/tauri'
import {
  deleteStoryEnding,
  loadStoryEndingCatalog,
  saveStoryEnding,
  STORY_ENDING_SCHEMA,
  validateStoryEndingDefinition,
  type StoryEndingAuthoringEntry,
  type StoryEndingCatalogSnapshot,
} from '../lib/storyEndings'
import {
  loadStoryDialogues,
  loadStoryScenes,
  type StoryDialogueInfo,
  type StoryEndingDefinition,
  type StorySceneInfo,
} from '../lib/storyContent'
import type { StoryContentAccessEntry } from '../lib/storyAccess'

const router = useRouter()
const { t } = useI18n()
const snapshot = ref<StoryEndingCatalogSnapshot | null>(null)
const scenes = ref<StorySceneInfo[]>([])
const dialogues = ref<StoryDialogueInfo[]>([])
const draft = ref<StoryEndingDefinition | null>(null)
const originalEndingId = ref<string | null>(null)
const baseline = ref('')
const search = ref('')
const busy = ref(false)
const notice = ref<{ type: 'success' | 'error'; title: string; message: string } | null>(null)

const filteredEndings = computed(() => {
  const query = search.value.toLowerCase()
  return (snapshot.value?.endings || []).filter((ending) => !query
    || ending.id.toLowerCase().includes(query)
    || ending.title.toLowerCase().includes(query)
    || ending.description.toLowerCase().includes(query))
})

const gatedCount = computed(() => (snapshot.value?.endings || []).filter((ending) => ending.access.gated).length)
const openCount = computed(() => (snapshot.value?.ending_count || 0) - gatedCount.value)
const serializedDraft = computed(() => draft.value ? JSON.stringify(draft.value) : '')
const dirty = computed(() => serializedDraft.value !== baseline.value)
const selectedScene = computed(() => scenes.value.find((scene) => scene.id === draft.value?.scene_id) || null)
const selectedDialogue = computed(() => dialogues.value.find((dialogue) => dialogue.id === draft.value?.dialogue_id) || null)
const selectedEntry = computed(() => (snapshot.value?.endings || [])
  .find((ending) => ending.id === originalEndingId.value) || null)
const endingEvents = computed(() => selectedEntry.value?.access.unlock_event_ids || [])
const sourcePath = computed(() => selectedEntry.value?.source_path || `endings/${draft.value?.id || 'new'}.json`)

const validationIssues = computed(() => {
  if (!draft.value) return [t('ending.error.no-selection', 'No ending selected.')]
  const issues = validateStoryEndingDefinition(draft.value)
  if (draft.value.scene_id && !selectedScene.value) issues.push(t('ending.error.scene-missing', 'Scene "{id}" does not exist.', { id: draft.value.scene_id }))
  if (draft.value.dialogue_id && !selectedDialogue.value) issues.push(t('ending.error.dialogue-missing', 'Dialogue "{id}" does not exist.', { id: draft.value.dialogue_id }))
  if (!originalEndingId.value && snapshot.value?.endings.some((ending) => ending.id === draft.value?.id)) {
    issues.push(t('ending.error.already-exists', 'Ending "{id}" already exists.', { id: draft.value.id }))
  }
  return issues
})

const warnings = computed(() => {
  if (!draft.value) return []
  const result: string[] = []
  if (originalEndingId.value && endingEvents.value.length === 0) {
    result.push(t('ending.warning.no-unlock', 'No Story Event unlocks this ending, so it is available from the start.'))
  }
  for (const eventId of endingEvents.value) {
    if (!selectedScene.value?.access.unlock_event_ids.includes(eventId)) {
      result.push(t('ending.warning.scene-not-unlocked', 'Event "{event}" does not unlock scene "{scene}".', { event: eventId, scene: draft.value.scene_id }))
    }
    if (!selectedDialogue.value?.access.unlock_event_ids.includes(eventId)) {
      result.push(t('ending.warning.dialogue-not-unlocked', 'Event "{event}" does not unlock dialogue "{dialogue}".', { event: eventId, dialogue: draft.value.dialogue_id }))
    }
  }
  return result
})

const canSave = computed(() => Boolean(draft.value && snapshot.value && dirty.value && validationIssues.value.length === 0))
const canDuplicate = computed(() => Boolean(draft.value))
const canPreview = computed(() => Boolean(originalEndingId.value && !dirty.value && validationIssues.value.length === 0))

function definitionFrom(entry: StoryEndingAuthoringEntry): StoryEndingDefinition {
  return {
    schema: STORY_ENDING_SCHEMA,
    id: entry.id,
    title: entry.title,
    description: entry.description,
    scene_id: entry.scene_id,
    dialogue_id: entry.dialogue_id,
  }
}

function setDraft(definition: StoryEndingDefinition, originalId: string | null) {
  draft.value = { ...definition }
  originalEndingId.value = originalId
  baseline.value = JSON.stringify(draft.value)
}

function confirmDiscard(): boolean {
  return !dirty.value || window.confirm(t('ending.confirm.discard', 'Discard unsaved ending changes?'))
}

function selectEnding(entry: StoryEndingAuthoringEntry) {
  if (entry.id === originalEndingId.value) return
  if (!confirmDiscard()) return
  setDraft(definitionFrom(entry), entry.id)
}

function nextEndingId(base = 'new_ending'): string {
  const ids = new Set(snapshot.value?.endings.map((ending) => ending.id) || [])
  if (!ids.has(base)) return base
  let index = 2
  while (ids.has(`${base}_${index}`)) index += 1
  return `${base}_${index}`
}

function createEnding() {
  if (!confirmDiscard()) return
  setDraft({
    schema: STORY_ENDING_SCHEMA,
    id: nextEndingId(),
    title: t('ending.new-ending', 'New Ending'),
    description: t('ending.new-description', 'A new story conclusion.'),
    scene_id: scenes.value[0]?.id || '',
    dialogue_id: dialogues.value[0]?.id || '',
  }, null)
  baseline.value = ''
}

function duplicateEnding() {
  if (!draft.value || !confirmDiscard()) return
  const copy = { ...draft.value, id: nextEndingId(`${draft.value.id}_copy`), title: t('authoring.copy-name', '{name} Copy', { name: draft.value.title }) }
  setDraft(copy, null)
  baseline.value = ''
}

async function loadCatalog(preferredId?: string | null) {
  busy.value = true
  try {
    const [nextSnapshot, nextScenes, nextDialogues] = await Promise.all([
      loadStoryEndingCatalog(),
      loadStoryScenes(),
      loadStoryDialogues(),
    ])
    snapshot.value = nextSnapshot
    scenes.value = nextScenes.sort((left, right) => left.name.localeCompare(right.name))
    dialogues.value = nextDialogues.sort((left, right) => left.title.localeCompare(right.title))
    const target = nextSnapshot.endings.find((ending) => ending.id === preferredId)
      || nextSnapshot.endings[0]
    if (target) setDraft(definitionFrom(target), target.id)
    else {
      draft.value = null
      originalEndingId.value = null
      baseline.value = ''
    }
  } catch (error) {
    showNotice('error', t('authoring.catalog-unavailable', 'Catalog unavailable'), String(error))
  } finally {
    busy.value = false
  }
}

async function reloadCatalog() {
  if (!confirmDiscard()) return
  await loadCatalog(originalEndingId.value)
  showNotice('success', t('authoring.catalog-reloaded', 'Catalog reloaded'), t('ending.notice.reloaded', 'Ending definitions and project references are current.'))
}

async function saveEnding() {
  if (!draft.value || !snapshot.value || !canSave.value) return
  busy.value = true
  try {
    const ending = {
      ...draft.value,
      id: draft.value.id.trim(),
      title: draft.value.title.trim(),
      description: draft.value.description.trim(),
      scene_id: draft.value.scene_id.trim(),
      dialogue_id: draft.value.dialogue_id.trim(),
    }
    const next = await saveStoryEnding(ending, originalEndingId.value, snapshot.value.catalog_fingerprint)
    snapshot.value = next
    const saved = next.endings.find((entry) => entry.id === ending.id)
    if (saved) setDraft(definitionFrom(saved), saved.id)
    showNotice('success', t('ending.notice.saved-title', 'Ending saved'), t('ending.notice.saved-message', '{title} passed project reload and reference validation.', { title: ending.title }))
  } catch (error) {
    showNotice('error', t('authoring.save-rejected', 'Save rejected'), String(error))
  } finally {
    busy.value = false
  }
}

async function removeEnding() {
  if (!originalEndingId.value || !snapshot.value) return
  const endingId = originalEndingId.value
  if (!window.confirm(t('ending.confirm.delete', 'Delete ending "{id}"?', { id: endingId }))) return
  busy.value = true
  try {
    const next = await deleteStoryEnding(endingId, snapshot.value.catalog_fingerprint)
    snapshot.value = next
    const target = next.endings[0]
    if (target) setDraft(definitionFrom(target), target.id)
    else {
      draft.value = null
      originalEndingId.value = null
      baseline.value = ''
    }
    showNotice('success', t('ending.notice.deleted-title', 'Ending deleted'), t('ending.notice.deleted-message', '{id} was removed from the project catalog.', { id: endingId }))
  } catch (error) {
    showNotice('error', t('authoring.delete-rejected', 'Delete rejected'), String(error))
  } finally {
    busy.value = false
  }
}

async function previewEnding() {
  if (!originalEndingId.value || !canPreview.value) return
  busy.value = true
  try {
    if (hasTauriRuntime()) {
      await invokeCommand('preview_story_ending', { endingId: originalEndingId.value })
      await router.push('/game')
    } else {
      await router.push({ path: '/game', query: { previewEnding: originalEndingId.value, authoring: '1' } })
    }
  } catch (error) {
    showNotice('error', t('authoring.preview-unavailable', 'Preview unavailable'), String(error))
  } finally {
    busy.value = false
  }
}

function accessLabel(access: StoryContentAccessEntry): string {
  if (!access.gated) return t('authoring.open', 'Open')
  return access.unlocked ? t('authoring.unlocked', 'Unlocked') : t('authoring.locked', 'Locked')
}

function showNotice(type: 'success' | 'error', title: string, message: string) {
  notice.value = { type, title, message }
}

function handleBeforeUnload(event: BeforeUnloadEvent) {
  if (!dirty.value) return
  event.preventDefault()
  event.returnValue = ''
}

function handleKeydown(event: KeyboardEvent) {
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 's') {
    event.preventDefault()
    void saveEnding()
  }
}

onBeforeRouteLeave(() => confirmDiscard())

onMounted(async () => {
  window.addEventListener('beforeunload', handleBeforeUnload)
  window.addEventListener('keydown', handleKeydown)
  await loadCatalog()
})

onUnmounted(() => {
  window.removeEventListener('beforeunload', handleBeforeUnload)
  window.removeEventListener('keydown', handleKeydown)
})
</script>

<style scoped>
.ending-editor {
  height: 100vh;
  height: 100svh;
  min-height: 0;
  display: flex;
  flex-direction: column;
  background: var(--surface-0);
  color: var(--text-primary);
}

.editor-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 20px;
  padding: 24px 28px 18px;
  border-bottom: 1px solid var(--border);
  background: var(--surface-1);
}

.header-copy { min-width: 0; }
.header-copy h1 { margin-top: 3px; font-size: 26px; line-height: 1.15; }
.header-copy p { margin-top: 5px; color: var(--text-tertiary); font-size: 13px; }
.header-actions { display: flex; flex-wrap: wrap; justify-content: flex-end; gap: 8px; }

.eyebrow {
  display: block;
  color: var(--text-tertiary);
  font-size: 10px;
  font-weight: 800;
  letter-spacing: 0;
  text-transform: uppercase;
}

.catalog-strip {
  min-height: 42px;
  display: flex;
  align-items: center;
  gap: 22px;
  padding: 8px 28px;
  border-bottom: 1px solid var(--border);
  color: var(--text-tertiary);
  font-size: 12px;
}
.catalog-strip strong { color: var(--text-secondary); font-family: var(--font-mono); }
.dirty-indicator { margin-left: auto; color: #f5b942; font-weight: 700; }

.editor-workspace {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: minmax(240px, 290px) minmax(0, 1fr);
}

.ending-list {
  min-height: 0;
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--border);
  background: var(--surface-1);
}

.list-toolbar {
  min-height: 58px;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border-bottom: 1px solid var(--border);
  color: var(--text-tertiary);
  font-size: 12px;
}
.search-field { flex: 1; min-width: 0; }
.search-field .input { width: 100%; }
.ending-items { min-height: 0; overflow-y: auto; padding: 8px; }

.ending-item {
  width: 100%;
  min-height: 62px;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px;
  border: 1px solid transparent;
  border-radius: 6px;
  background: transparent;
  color: var(--text-primary);
  text-align: left;
  cursor: pointer;
}
.ending-item:hover { background: var(--surface-2); }
.ending-item.active { border-color: color-mix(in srgb, var(--brand) 42%, var(--border)); background: color-mix(in srgb, var(--brand) 7%, var(--surface-1)); }
.ending-item-copy { flex: 1; min-width: 0; }
.ending-item-copy strong, .ending-item-copy small { display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.ending-item-copy strong { font-size: 13px; }
.ending-item-copy small { margin-top: 4px; color: var(--text-tertiary); font-family: var(--font-mono); font-size: 10px; }
.access-dot { flex-shrink: 0; font-size: 10px; font-weight: 800; }
.access-dot.gated { color: #f5b942; }
.access-dot.open { color: #52c98f; }
.empty-list { padding: 30px 12px; color: var(--text-tertiary); text-align: center; }

.ending-form { min-width: 0; min-height: 0; display: flex; flex-direction: column; }
.route-map {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 36px minmax(0, 1fr) 36px minmax(0, 1fr);
  align-items: center;
  padding: 18px 22px;
  border-bottom: 1px solid var(--border);
  background: var(--surface-1);
}
.route-step {
  min-width: 0;
  min-height: 76px;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--surface-2);
}
.scene-step { border-top: 3px solid #4da3ff; }
.dialogue-step { border-top: 3px solid #d18cff; }
.ending-step { border-top: 3px solid #f5b942; }
.step-number {
  width: 28px;
  height: 28px;
  flex: 0 0 28px;
  display: grid;
  place-items: center;
  border-radius: 50%;
  background: var(--surface-3);
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: 11px;
  font-weight: 800;
}
.step-copy { min-width: 0; }
.step-copy small, .step-copy strong, .step-copy span { display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.step-copy small { color: var(--text-tertiary); font-size: 9px; font-weight: 800; text-transform: uppercase; }
.step-copy strong { margin-top: 3px; font-size: 13px; }
.step-copy span { margin-top: 3px; color: var(--text-tertiary); font-size: 10px; }
.route-connector { height: 1px; background: var(--border-strong, var(--border)); }

.validation-banner {
  min-height: 48px;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 22px;
  border-bottom: 1px solid var(--border);
  font-size: 12px;
}
.validation-banner strong { flex-shrink: 0; }
.validation-banner span { color: var(--text-secondary); overflow-wrap: anywhere; }
.validation-banner.error { background: rgba(239, 91, 91, 0.08); color: #ff8d8d; }
.validation-banner.warning { background: rgba(245, 185, 66, 0.08); color: #f5c15a; }
.validation-banner.valid { background: rgba(82, 201, 143, 0.07); color: #65d9a0; }

.form-scroll { min-height: 0; overflow-y: auto; }
.form-section { padding: 22px; border-bottom: 1px solid var(--border); }
.section-heading { display: flex; align-items: flex-start; justify-content: space-between; gap: 16px; margin-bottom: 16px; }
.section-heading h2 { margin-top: 3px; font-size: 16px; }
.source-path { max-width: 50%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--text-tertiary); font-family: var(--font-mono); font-size: 10px; }
.field-grid { display: grid; grid-template-columns: minmax(180px, 0.7fr) minmax(240px, 1.3fr); gap: 14px; }
.description-field { grid-column: 1 / -1; }
.form-field { min-width: 0; display: flex; flex-direction: column; gap: 7px; color: var(--text-secondary); font-size: 12px; }
.form-field > small { align-self: flex-end; color: var(--text-tertiary); font-size: 10px; }
.input { width: 100%; }
.mono { font-family: var(--font-mono); }
.state-label { font-size: 10px; font-weight: 800; text-transform: uppercase; }
.state-label.ready { color: #65d9a0; }
.state-label.locked { color: #f5b942; }

.reference-details {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 1px;
  margin-top: 14px;
  border: 1px solid var(--border);
  background: var(--border);
}
.reference-details span {
  min-width: 0;
  padding: 10px;
  background: var(--surface-1);
  color: var(--text-secondary);
  font-size: 11px;
  overflow-wrap: anywhere;
}
.reference-details strong { display: block; margin-bottom: 5px; color: var(--text-tertiary); font-size: 9px; text-transform: uppercase; }
.coverage-grid { border-top: 1px solid var(--border); }
.coverage-row { display: grid; grid-template-columns: 120px minmax(0, 1fr); gap: 12px; padding: 11px 0; border-bottom: 1px solid var(--border); font-size: 12px; }
.coverage-row span { color: var(--text-tertiary); }
.coverage-row strong { min-width: 0; overflow-wrap: anywhere; color: var(--text-secondary); }

.form-footer {
  min-height: 68px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 12px 22px;
  border-top: 1px solid var(--border);
  background: var(--surface-1);
}
.footer-status span, .footer-status small { display: block; }
.footer-status span { font-size: 12px; font-weight: 800; }
.footer-status small { margin-top: 3px; color: var(--text-tertiary); font-size: 10px; }
.valid-text { color: #65d9a0; }
.invalid-text { color: #ff8d8d; }
.footer-actions { display: flex; gap: 8px; }

.empty-editor { min-height: 420px; display: grid; place-content: center; justify-items: center; color: var(--text-tertiary); }
.empty-mark { width: 54px; height: 54px; display: grid; place-items: center; border: 1px solid var(--border); border-radius: 6px; font-family: var(--font-mono); font-weight: 800; }
.empty-editor h2 { margin-top: 12px; color: var(--text-secondary); font-size: 16px; }

.notice {
  position: fixed;
  right: 22px;
  bottom: 22px;
  z-index: 100;
  width: min(390px, calc(100vw - 32px));
  display: block;
  padding: 13px 15px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--surface-2);
  color: var(--text-primary);
  text-align: left;
  box-shadow: var(--shadow-lg);
  cursor: pointer;
}
.notice.success { border-left: 4px solid #52c98f; }
.notice.error { border-left: 4px solid #ef5b5b; }
.notice strong, .notice span { display: block; }
.notice strong { font-size: 12px; }
.notice span { margin-top: 4px; color: var(--text-secondary); font-size: 11px; overflow-wrap: anywhere; }
.sr-only { position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px; overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; border: 0; }

@media (max-width: 980px) {
  .editor-workspace { grid-template-columns: 230px minmax(0, 1fr); }
  .route-map { grid-template-columns: 1fr; gap: 8px; }
  .route-connector { width: 1px; height: 12px; justify-self: center; }
  .reference-details { grid-template-columns: 1fr; }
}

@media (max-width: 720px) {
  .ending-editor { height: auto; min-height: 100svh; }
  .editor-header { padding: 16px; flex-direction: column; }
  .header-actions { width: 100%; justify-content: flex-start; }
  .catalog-strip { padding: 8px 16px; gap: 12px; overflow-x: auto; white-space: nowrap; }
  .dirty-indicator { margin-left: 0; }
  .editor-workspace { display: flex; flex-direction: column; }
  .ending-list { flex: 0 0 auto; max-height: 220px; border-right: none; border-bottom: 1px solid var(--border); }
  .ending-items { display: flex; overflow-x: auto; overflow-y: hidden; }
  .ending-item { min-width: 220px; }
  .ending-form { overflow: visible; }
  .route-map, .form-section { padding: 16px; }
  .validation-banner { padding: 10px 16px; align-items: flex-start; flex-direction: column; gap: 3px; }
  .field-grid { grid-template-columns: 1fr; }
  .description-field { grid-column: auto; }
  .form-footer { position: sticky; bottom: 0; padding: 10px 16px; }
}

@media (max-width: 480px) {
  .header-copy h1 { font-size: 22px; }
  .header-actions .btn { flex: 1 1 calc(33.333% - 8px); }
  .section-heading { flex-direction: column; gap: 8px; }
  .source-path { max-width: 100%; }
  .coverage-row { grid-template-columns: 80px minmax(0, 1fr); }
  .form-footer { align-items: flex-end; }
  .footer-actions { flex-direction: column; }
}
</style>
