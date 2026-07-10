<template>
  <div class="scene-editor">
    <header class="editor-header">
      <div class="header-copy">
        <span class="eyebrow">{{ t('scene.editor-eyebrow', 'World Design') }}</span>
        <h1>{{ t('scene.catalog-title', 'Scene Catalog') }}</h1>
        <p>{{ t('scene.catalog-summary', '{total} scenes · {authored} authored · {inferred} inferred', {
          total: snapshot?.scene_count || 0,
          authored: snapshot?.metadata_scene_count || 0,
          inferred: snapshot?.inferred_scene_count || 0,
        }) }}</p>
      </div>
      <div class="header-actions">
        <button class="btn btn-secondary btn-sm" :disabled="busy" @click="createScene"><Plus :size="14" />{{ t('authoring.new', 'New') }}</button>
        <button class="btn btn-secondary btn-sm" :disabled="!draft || busy" @click="duplicateScene"><Copy :size="14" />{{ t('authoring.duplicate', 'Duplicate') }}</button>
        <button class="btn btn-secondary btn-sm" :disabled="busy" @click="reloadCatalog"><RotateCcw :size="14" />{{ t('authoring.reload', 'Reload') }}</button>
        <button class="btn btn-secondary btn-sm" :disabled="!canPreview || busy" @click="previewScene"><Play :size="14" />{{ t('authoring.story-mode', 'Story Mode') }}</button>
        <button class="btn btn-primary btn-sm" :disabled="!canSave || busy" @click="saveScene">
          <Save :size="14" />
          {{ busy ? t('authoring.working', 'Working') : selectedEntry && !selectedEntry.metadata_authored ? t('scene.promote', 'Promote') : t('common.save', 'Save') }}
        </button>
      </div>
    </header>

    <div class="catalog-strip">
      <span><strong>{{ filteredScenes.length }}</strong> {{ t('authoring.visible', 'visible') }}</span>
      <span><strong>{{ gatedCount }}</strong> {{ t('authoring.gated', 'gated') }}</span>
      <span><strong>{{ errorCount }}</strong> {{ t('authoring.errors', 'errors') }}</span>
      <span><strong>{{ snapshot?.catalog_fingerprint.slice(0, 12) || t('authoring.unavailable', 'unavailable') }}</strong> {{ t('authoring.catalog', 'catalog') }}</span>
      <span v-if="dirty" class="dirty-indicator">{{ t('authoring.unsaved-changes', 'Unsaved changes') }}</span>
    </div>

    <main class="editor-workspace">
      <aside class="scene-list" :aria-label="t('scene.catalog-aria', 'Scene catalog')">
        <div class="list-toolbar">
          <label class="search-field">
            <span class="sr-only">{{ t('scene.search', 'Search scenes') }}</span>
            <input v-model.trim="search" class="input" type="search" :placeholder="t('scene.search', 'Search scenes')" />
          </label>
          <select v-model="filter" class="input filter-select" :aria-label="t('scene.source-filter', 'Scene source filter')">
            <option value="all">{{ t('knowledge.all', 'All') }}</option>
            <option value="authored">{{ t('authoring.authored', 'Authored') }}</option>
            <option value="inferred">{{ t('authoring.inferred', 'Inferred') }}</option>
          </select>
        </div>

        <div class="scene-items">
          <button
            v-for="scene in filteredScenes"
            :key="scene.id"
            class="scene-item"
            :class="{ active: selectedCatalogId === scene.id }"
            @click="selectScene(scene)"
          >
            <span class="scene-thumb">
              <img v-if="sceneImage(scene)" :src="sceneImage(scene) || ''" :alt="scene.name" />
              <span v-else>SC</span>
            </span>
            <span class="scene-item-copy">
              <strong>{{ scene.name }}</strong>
              <small>{{ scene.id }}</small>
              <span class="item-meta">
                <b :class="scene.metadata_authored ? 'authored' : 'inferred'">
                  {{ scene.metadata_authored ? t('authoring.authored', 'Authored') : t('authoring.inferred', 'Inferred') }}
                </b>
                <b :class="scene.access.gated ? 'gated' : 'open'">
                  {{ scene.access.gated ? t('authoring.gated-label', 'Gated') : t('authoring.open', 'Open') }}
                </b>
              </span>
            </span>
          </button>
          <div v-if="filteredScenes.length === 0" class="empty-list">{{ t('scene.no-scenes', 'No scenes') }}</div>
        </div>
      </aside>

      <section v-if="draft" class="scene-form">
        <div class="scene-stage">
          <img
            v-if="previewUrl && !previewFailed"
            :src="previewUrl"
            :alt="draft.name || draft.id"
            @error="previewFailed = true"
          />
          <div v-else class="stage-empty">
            <span>SC</span>
            <strong>{{ draft.background_path ? t('scene.background-unavailable', 'Background unavailable') : t('scene.no-background-assigned', 'No background assigned') }}</strong>
          </div>
          <div class="stage-shade"></div>
          <div class="stage-caption">
            <span>{{ draft.time_of_day || t('scene.any-time', 'Any time') }}</span>
            <h2>{{ draft.name || t('scene.untitled', 'Untitled scene') }}</h2>
            <p>{{ draft.background_path || t('scene.no-background-path', 'No background path') }}</p>
          </div>
          <span class="source-badge" :class="selectedEntry?.metadata_authored ? 'authored' : 'inferred'">
            {{ selectedEntry?.metadata_authored ? t('scene.metadata', 'Metadata') : selectedEntry ? t('scene.background-inferred', 'Background inferred') : t('scene.new-scene', 'New scene') }}
          </span>
        </div>

        <div v-if="validationIssues.length" class="validation-banner error" role="alert">
          <strong>{{ t('authoring.blocking-issues', '{count} blocking issues', { count: validationIssues.length }) }}</strong>
          <span>{{ validationIssues[0] }}</span>
        </div>
        <div v-else-if="warnings.length" class="validation-banner warning">
          <strong>{{ t('authoring.warnings-count', '{count} warnings', { count: warnings.length }) }}</strong>
          <span>{{ warnings[0] }}</span>
        </div>
        <div v-else class="validation-banner valid">
          <strong>{{ t('scene.valid', 'Scene valid') }}</strong>
          <span>{{ selectedEntry?.background_exists ? t('scene.background-resolved', 'Background resolved') : t('scene.ready-validation', 'Ready for project validation') }}</span>
        </div>

        <div class="form-scroll">
          <section class="form-section">
            <div class="section-heading">
              <div>
                <span class="eyebrow">{{ t('authoring.identity', 'Identity') }}</span>
                <h2>{{ t('scene.definition', 'Scene definition') }}</h2>
              </div>
              <span class="source-path">{{ sourcePath }}</span>
            </div>
            <div class="field-grid identity-grid">
              <label class="form-field">
                <span>{{ t('scene.id', 'Scene ID') }}</span>
                <input v-model.trim="draft.id" class="input mono" :disabled="selectedCatalogId !== null" maxlength="128" />
              </label>
              <label class="form-field">
                <span>{{ t('common.name', 'Name') }}</span>
                <input v-model="draft.name" class="input" maxlength="256" />
              </label>
            </div>
          </section>

          <section class="form-section">
            <div class="section-heading">
              <div>
                <span class="eyebrow">{{ t('scene.assets', 'Assets') }}</span>
                <h2>{{ t('scene.stage-media', 'Stage media') }}</h2>
              </div>
              <span class="state-label" :class="selectedEntry?.background_exists ? 'ready' : 'pending'">
                {{ selectedEntry?.background_exists ? t('authoring.resolved', 'Resolved') : t('authoring.pending', 'Pending') }}
              </span>
            </div>
            <div class="field-grid">
              <label class="form-field full-field">
                <span>{{ t('scene.background-path', 'Background path') }}</span>
                <input v-model="draft.background_path" class="input mono" placeholder="assets/backgrounds/scene.svg" />
              </label>
              <label class="form-field full-field">
                <span>{{ t('scene.bgm', 'BGM path') }}</span>
                <input v-model="draft.bgm_path" class="input mono" placeholder="assets/audio/theme.ogg" />
              </label>
            </div>
          </section>

          <section class="form-section">
            <div class="section-heading">
              <div>
                <span class="eyebrow">{{ t('scene.atmosphere', 'Atmosphere') }}</span>
                <h2>{{ t('scene.environment-metadata', 'Environment metadata') }}</h2>
              </div>
            </div>
            <div class="field-grid atmosphere-grid">
              <label class="form-field">
                <span>{{ t('scene.weather', 'Weather') }}</span>
                <input v-model="draft.weather" class="input" maxlength="64" placeholder="clear" />
              </label>
              <label class="form-field">
                <span>{{ t('scene.time-of-day', 'Time of day') }}</span>
                <input v-model="draft.time_of_day" class="input" maxlength="64" placeholder="golden_hour" />
              </label>
              <label class="form-field full-field">
                <span>{{ t('scene.tags', 'Tags') }}</span>
                <input v-model="tagsText" class="input" placeholder="outdoor, calm, route-a" />
                <small>{{ t('scene.tags-count', '{count} tags', { count: draft.tags.length }) }}</small>
              </label>
            </div>
          </section>
        </div>

        <footer class="form-footer">
          <div class="footer-status">
            <span :class="validationIssues.length ? 'invalid-text' : 'valid-text'">
              {{ validationIssues.length ? t('authoring.issues-count', '{count} issues', { count: validationIssues.length }) : dirty ? t('authoring.unsaved-changes', 'Unsaved changes') : t('authoring.saved', 'Saved') }}
            </span>
            <small>{{ selectedEntry?.metadata_authored ? t('scene.project-metadata', 'Project metadata') : selectedEntry ? t('scene.inferred-asset', 'Inferred asset') : t('authoring.new-asset', 'New asset') }}</small>
          </div>
          <div class="footer-actions">
            <button
              class="btn btn-danger btn-sm"
              :disabled="!selectedEntry?.metadata_authored || busy"
              :title="selectedEntry && !selectedEntry.metadata_authored ? t('scene.inferred-delete-help', 'Inferred scenes have no metadata file to delete') : t('scene.delete-metadata', 'Delete scene metadata')"
              @click="removeScene"
            ><Trash2 :size="14" />{{ t('common.delete', 'Delete') }}</button>
            <button class="btn btn-primary" :disabled="!canSave || busy" @click="saveScene">
              <Save :size="15" />
              {{ selectedEntry && !selectedEntry.metadata_authored ? t('scene.promote-scene', 'Promote Scene') : t('scene.save-scene', 'Save Scene') }}
            </button>
          </div>
        </footer>
      </section>

      <section v-else class="empty-editor">
        <span class="empty-mark">SC</span>
        <h2>{{ t('scene.no-selection', 'No scene selected') }}</h2>
      </section>

      <aside class="scene-inspector" :aria-label="t('scene.diagnostics-aria', 'Scene diagnostics')">
        <section class="inspector-section">
          <span class="eyebrow">{{ t('authoring.runtime-access', 'Runtime Access') }}</span>
          <div class="metric-row"><span>{{ t('common.status', 'Status') }}</span><strong>{{ accessStatus }}</strong></div>
          <div class="metric-row"><span>{{ t('authoring.unlock-events', 'Unlock events') }}</span><strong>{{ selectedEntry?.access.unlock_event_ids.length || 0 }}</strong></div>
          <p v-if="selectedEntry?.access.unlock_event_ids.length" class="event-list">
            {{ selectedEntry.access.unlock_event_ids.join(', ') }}
          </p>
        </section>
        <section class="inspector-section">
          <span class="eyebrow">{{ t('authoring.document', 'Document') }}</span>
          <div class="metric-row"><span>{{ t('authoring.source', 'Source') }}</span><strong>{{ selectedEntry?.metadata_authored ? 'JSON' : selectedEntry ? t('authoring.asset', 'Asset') : t('authoring.draft', 'Draft') }}</strong></div>
          <div class="metric-row"><span>{{ t('scene.background', 'Background') }}</span><strong>{{ selectedEntry?.background_exists ? t('authoring.found', 'Found') : t('authoring.unchecked', 'Unchecked') }}</strong></div>
          <div class="fingerprint">{{ selectedEntry?.content_fingerprint || t('authoring.not-saved', 'Not saved') }}</div>
        </section>
        <section class="inspector-section issue-section">
          <span class="eyebrow">{{ t('authoring.diagnostics', 'Diagnostics') }}</span>
          <div v-for="issue in relevantIssues" :key="`${issue.code}:${issue.path}`" class="diagnostic" :class="issue.severity">
            <strong>{{ issue.code }}</strong>
            <span>{{ issue.message }}</span>
          </div>
          <p v-if="relevantIssues.length === 0" class="clean-state">{{ t('scene.no-diagnostics', 'No catalog diagnostics') }}</p>
        </section>
      </aside>
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
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { onBeforeRouteLeave, useRouter } from 'vue-router'
import { Copy, Play, Plus, RotateCcw, Save, Trash2 } from '@lucide/vue'
import { resolveAssetUrl } from '../lib/assets'
import { useI18n } from '../lib/i18n'
import {
  deleteSceneDefinition,
  loadSceneAuthoringCatalog,
  normalizeSceneDefinition,
  saveSceneDefinition,
  validateSceneDefinition,
  type SceneAuthoringCatalogSnapshot,
  type SceneAuthoringEntry,
} from '../lib/sceneAuthoring'
import type { SceneDefinition } from '../lib/storyContent'
import { hasTauriRuntime, invokeCommand } from '../lib/tauri'

interface BackgroundAsset {
  relative_path: string
  absolute_path: string
}

interface SceneAssetCatalog {
  backgrounds: BackgroundAsset[]
}

const router = useRouter()
const { t } = useI18n()
const snapshot = ref<SceneAuthoringCatalogSnapshot | null>(null)
const draft = ref<SceneDefinition | null>(null)
const selectedCatalogId = ref<string | null>(null)
const baseline = ref('')
const search = ref('')
const filter = ref<'all' | 'authored' | 'inferred'>('all')
const busy = ref(false)
const previewFailed = ref(false)
const backgroundPaths = ref<Record<string, string>>({})
const notice = ref<{ type: 'success' | 'error'; title: string; message: string } | null>(null)

const filteredScenes = computed(() => {
  const query = search.value.toLowerCase()
  return (snapshot.value?.scenes || []).filter((scene) => {
    const sourceMatches = filter.value === 'all'
      || (filter.value === 'authored' && scene.metadata_authored)
      || (filter.value === 'inferred' && !scene.metadata_authored)
    return sourceMatches && (!query
      || scene.id.toLowerCase().includes(query)
      || scene.name.toLowerCase().includes(query)
      || scene.tags.some((tag) => tag.toLowerCase().includes(query)))
  })
})

const selectedEntry = computed(() => (snapshot.value?.scenes || [])
  .find((scene) => scene.id === selectedCatalogId.value) || null)
const serializedDraft = computed(() => draft.value ? JSON.stringify(draft.value) : '')
const dirty = computed(() => serializedDraft.value !== baseline.value)
const gatedCount = computed(() => (snapshot.value?.scenes || []).filter((scene) => scene.access.gated).length)
const errorCount = computed(() => (snapshot.value?.issues || []).filter((issue) => issue.severity === 'error').length)
const sourcePath = computed(() => selectedEntry.value?.source_path || `scenes/${draft.value?.id || 'new'}.json`)
const accessStatus = computed(() => {
  const access = selectedEntry.value?.access
  if (!access) return t('authoring.draft', 'Draft')
  if (!access.gated) return t('authoring.open', 'Open')
  return access.unlocked ? t('authoring.unlocked', 'Unlocked') : t('authoring.locked', 'Locked')
})
const tagsText = computed({
  get: () => draft.value?.tags.join(', ') || '',
  set: (value: string) => {
    if (draft.value) draft.value.tags = value.split(',').map((tag) => tag.trim()).filter(Boolean)
  },
})
const validationIssues = computed(() => {
  if (!draft.value) return [t('scene.error.no-selection', 'No scene selected.')]
  const normalized = normalizeSceneDefinition(draft.value)
  const issues = validateSceneDefinition(normalized)
  if (!selectedCatalogId.value && snapshot.value?.scenes.some((scene) => scene.id === normalized.id)) {
    issues.push(t('scene.error.already-exists', 'Scene "{id}" already exists.', { id: normalized.id }))
  }
  return issues
})
const warnings = computed(() => {
  if (!draft.value) return []
  const result: string[] = []
  if (!draft.value.background_path?.trim()) result.push(t('scene.warning.no-background', 'No background is assigned to this scene.'))
  if (selectedEntry.value?.background_path === draft.value.background_path && !selectedEntry.value.background_exists) {
    result.push(t('scene.warning.unresolved-background', 'The saved background path does not resolve to a project file.'))
  }
  return result
})
const relevantIssues = computed(() => (snapshot.value?.issues || []).filter((issue) =>
  !issue.scene_id || issue.scene_id === selectedCatalogId.value))
const previewUrl = computed(() => {
  const path = draft.value?.background_path?.trim()
  if (!path) return null
  return resolveAssetUrl(backgroundPaths.value[path] || selectedEntry.value?.absolute_background_path || path)
})
const canSave = computed(() => Boolean(draft.value && snapshot.value && dirty.value && validationIssues.value.length === 0))
const canPreview = computed(() => Boolean(selectedCatalogId.value && !dirty.value && validationIssues.value.length === 0))

watch(previewUrl, () => { previewFailed.value = false })

function sceneImage(scene: SceneAuthoringEntry): string | null {
  const path = scene.background_path
  if (!path) return null
  return resolveAssetUrl(backgroundPaths.value[path] || scene.absolute_background_path || path)
}

function definitionFrom(entry: SceneAuthoringEntry): SceneDefinition {
  return {
    id: entry.id,
    name: entry.name,
    background_path: entry.background_path,
    bgm_path: entry.bgm_path,
    weather: entry.weather,
    time_of_day: entry.time_of_day,
    tags: [...entry.tags],
  }
}

function setDraft(definition: SceneDefinition, catalogId: string | null, isSaved = true) {
  draft.value = { ...definition, tags: [...definition.tags] }
  selectedCatalogId.value = catalogId
  baseline.value = isSaved ? JSON.stringify(draft.value) : ''
  previewFailed.value = false
}

function confirmDiscard(): boolean {
  return !dirty.value || window.confirm(t('scene.confirm.discard', 'Discard unsaved scene changes?'))
}

function selectScene(entry: SceneAuthoringEntry) {
  if (entry.id === selectedCatalogId.value) return
  if (!confirmDiscard()) return
  setDraft(definitionFrom(entry), entry.id)
}

function nextSceneId(base = 'new_scene'): string {
  const ids = new Set(snapshot.value?.scenes.map((scene) => scene.id) || [])
  if (!ids.has(base)) return base
  let index = 2
  while (ids.has(`${base}_${index}`)) index += 1
  return `${base}_${index}`
}

function createScene() {
  if (!confirmDiscard()) return
  setDraft({
    id: nextSceneId(),
    name: t('scene.new-scene', 'New scene'),
    background_path: null,
    bgm_path: null,
    weather: null,
    time_of_day: null,
    tags: [],
  }, null, false)
}

function duplicateScene() {
  if (!draft.value || !confirmDiscard()) return
  setDraft({
    ...draft.value,
    id: nextSceneId(`${draft.value.id}_copy`),
    name: t('authoring.copy-name', '{name} Copy', { name: draft.value.name }),
    tags: [...draft.value.tags],
  }, null, false)
}

async function loadBackgroundPaths() {
  if (!hasTauriRuntime()) {
    backgroundPaths.value = {}
    return
  }
  const catalog = await invokeCommand<SceneAssetCatalog>('list_scene_assets')
  backgroundPaths.value = Object.fromEntries(catalog.backgrounds.map((asset) => [asset.relative_path, asset.absolute_path]))
}

async function loadCatalog(preferredId?: string | null) {
  busy.value = true
  try {
    const [nextSnapshot] = await Promise.all([
      loadSceneAuthoringCatalog(),
      loadBackgroundPaths(),
    ])
    snapshot.value = nextSnapshot
    const target = nextSnapshot.scenes.find((scene) => scene.id === preferredId) || nextSnapshot.scenes[0]
    if (target) setDraft(definitionFrom(target), target.id)
    else {
      draft.value = null
      selectedCatalogId.value = null
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
  await loadCatalog(selectedCatalogId.value)
  showNotice('success', t('authoring.catalog-reloaded', 'Catalog reloaded'), t('scene.notice.reloaded', 'Scene definitions and asset diagnostics are current.'))
}

async function saveScene() {
  if (!draft.value || !snapshot.value || !canSave.value) return
  busy.value = true
  try {
    const scene = normalizeSceneDefinition(draft.value)
    const originalId = selectedEntry.value?.metadata_authored ? selectedEntry.value.id : null
    const next = await saveSceneDefinition(scene, originalId, snapshot.value.catalog_fingerprint)
    snapshot.value = next
    await loadBackgroundPaths()
    const saved = next.scenes.find((entry) => entry.id === scene.id)
    if (saved) setDraft(definitionFrom(saved), saved.id)
    showNotice('success', originalId ? t('scene.notice.saved-title', 'Scene saved') : t('scene.notice.created-title', 'Scene created'), t('scene.notice.saved-message', '{name} passed project reload validation.', { name: scene.name }))
  } catch (error) {
    showNotice('error', t('authoring.save-rejected', 'Save rejected'), String(error))
  } finally {
    busy.value = false
  }
}

async function removeScene() {
  if (!selectedEntry.value?.metadata_authored || !snapshot.value) return
  const sceneId = selectedEntry.value.id
  if (!window.confirm(t('scene.confirm.delete', 'Delete scene metadata "{id}"?', { id: sceneId }))) return
  busy.value = true
  try {
    const next = await deleteSceneDefinition(sceneId, snapshot.value.catalog_fingerprint)
    snapshot.value = next
    await loadBackgroundPaths()
    const target = next.scenes.find((scene) => scene.id === sceneId) || next.scenes[0]
    if (target) setDraft(definitionFrom(target), target.id)
    else {
      draft.value = null
      selectedCatalogId.value = null
      baseline.value = ''
    }
    showNotice('success', t('scene.notice.deleted-title', 'Metadata deleted'), target?.id === sceneId
      ? t('scene.notice.asset-remains', '{id} remains available from its background asset.', { id: sceneId })
      : t('scene.notice.removed', '{id} was removed from the scene catalog.', { id: sceneId }))
  } catch (error) {
    showNotice('error', t('authoring.delete-rejected', 'Delete rejected'), String(error))
  } finally {
    busy.value = false
  }
}

async function previewScene() {
  if (!selectedCatalogId.value || !canPreview.value) return
  busy.value = true
  try {
    if (hasTauriRuntime()) {
      await invokeCommand('set_scene', { sceneId: selectedCatalogId.value })
      await router.push('/game')
    } else {
      await router.push({ path: '/game', query: { previewScene: selectedCatalogId.value, authoring: '1' } })
    }
  } catch (error) {
    showNotice('error', t('authoring.preview-unavailable', 'Preview unavailable'), String(error))
  } finally {
    busy.value = false
  }
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
    void saveScene()
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
.scene-editor {
  height: 100vh;
  height: 100svh;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--surface-0);
  color: var(--text-primary);
}

.editor-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 20px;
  padding: 22px 26px 16px;
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
  min-height: 40px;
  display: flex;
  align-items: center;
  gap: 20px;
  padding: 7px 26px;
  border-bottom: 1px solid var(--border);
  color: var(--text-tertiary);
  font-size: 12px;
}
.catalog-strip strong { color: var(--text-secondary); font-family: var(--font-mono); }
.dirty-indicator { margin-left: auto; color: var(--warning); font-weight: 800; }

.editor-workspace {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: minmax(230px, 270px) minmax(440px, 1fr) minmax(230px, 270px);
}

.scene-list {
  min-height: 0;
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--border);
  background: var(--surface-1);
}
.list-toolbar {
  min-height: 58px;
  display: grid;
  grid-template-columns: minmax(0, 1fr) 92px;
  align-items: center;
  gap: 8px;
  padding: 10px;
  border-bottom: 1px solid var(--border);
}
.search-field { min-width: 0; }
.filter-select { padding-left: 9px; padding-right: 6px; }
.scene-items { min-height: 0; overflow-y: auto; padding: 7px; }
.scene-item {
  width: 100%;
  min-height: 78px;
  display: grid;
  grid-template-columns: 72px minmax(0, 1fr);
  gap: 10px;
  align-items: center;
  padding: 7px;
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-primary);
  text-align: left;
  cursor: pointer;
}
.scene-item:hover { background: var(--surface-2); border-color: var(--border); }
.scene-item.active { background: var(--surface-3); border-color: var(--brand); }
.scene-thumb {
  width: 72px;
  aspect-ratio: 16 / 10;
  display: grid;
  place-items: center;
  overflow: hidden;
  border: 1px solid var(--border);
  border-radius: 4px;
  background: var(--surface-0);
  color: var(--text-tertiary);
  font-family: var(--font-mono);
  font-size: 11px;
  font-weight: 800;
}
.scene-thumb img { width: 100%; height: 100%; object-fit: cover; }
.scene-item-copy { min-width: 0; display: block; }
.scene-item-copy strong,
.scene-item-copy small { display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.scene-item-copy strong { font-size: 12px; }
.scene-item-copy small { margin-top: 2px; color: var(--text-tertiary); font-family: var(--font-mono); font-size: 10px; }
.item-meta { display: flex; gap: 5px; margin-top: 7px; }
.item-meta b,
.source-badge {
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 9px;
  line-height: 1.4;
  text-transform: uppercase;
}
.authored { background: rgba(45, 212, 191, 0.14); color: var(--brand-light); }
.inferred { background: rgba(96, 165, 250, 0.15); color: var(--info); }
.gated { background: rgba(245, 158, 11, 0.14); color: var(--warning); }
.open { background: var(--surface-4); color: var(--text-secondary); }
.empty-list { padding: 28px 12px; color: var(--text-tertiary); text-align: center; }

.scene-form { min-width: 0; min-height: 0; display: flex; flex-direction: column; background: var(--surface-0); }
.scene-stage {
  position: relative;
  flex: 0 0 clamp(190px, 31vh, 330px);
  min-height: 190px;
  overflow: hidden;
  border-bottom: 1px solid var(--border);
  background: #11151b;
}
.scene-stage > img { width: 100%; height: 100%; object-fit: cover; }
.stage-empty { position: absolute; inset: 0; display: grid; place-content: center; gap: 8px; color: var(--text-tertiary); text-align: center; }
.stage-empty span { font-family: var(--font-mono); font-size: 26px; font-weight: 900; }
.stage-empty strong { font-size: 12px; }
.stage-shade { position: absolute; inset: 0; background: linear-gradient(0deg, rgba(8, 10, 14, 0.88), transparent 65%); pointer-events: none; }
.stage-caption { position: absolute; left: 24px; right: 24px; bottom: 20px; min-width: 0; }
.stage-caption span { color: var(--brand-light); font-size: 10px; font-weight: 800; text-transform: uppercase; }
.stage-caption h2 { margin-top: 3px; overflow: hidden; color: white; font-size: 22px; line-height: 1.2; text-overflow: ellipsis; white-space: nowrap; }
.stage-caption p { margin-top: 3px; overflow: hidden; color: rgba(255, 255, 255, 0.68); font-family: var(--font-mono); font-size: 10px; text-overflow: ellipsis; white-space: nowrap; }
.source-badge { position: absolute; top: 14px; right: 14px; font-weight: 800; }

.validation-banner {
  min-height: 46px;
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  align-items: center;
  gap: 12px;
  padding: 9px 20px;
  border-bottom: 1px solid var(--border);
  font-size: 11px;
}
.validation-banner strong { font-size: 11px; }
.validation-banner span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.validation-banner.error { background: rgba(239, 68, 68, 0.08); color: #fca5a5; }
.validation-banner.warning { background: rgba(245, 158, 11, 0.08); color: #fcd34d; }
.validation-banner.valid { background: rgba(34, 197, 94, 0.07); color: #86efac; }

.form-scroll { flex: 1; min-height: 0; overflow-y: auto; }
.form-section { padding: 20px 24px 22px; border-bottom: 1px solid var(--border); }
.section-heading { display: flex; align-items: flex-start; justify-content: space-between; gap: 16px; margin-bottom: 14px; }
.section-heading h2 { margin-top: 2px; font-size: 16px; line-height: 1.25; }
.source-path { max-width: 52%; overflow: hidden; color: var(--text-tertiary); font-family: var(--font-mono); font-size: 10px; text-overflow: ellipsis; white-space: nowrap; }
.field-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 13px; }
.identity-grid { grid-template-columns: minmax(180px, 0.75fr) minmax(240px, 1.25fr); }
.atmosphere-grid { grid-template-columns: 1fr 1fr; }
.form-field { min-width: 0; display: grid; gap: 5px; }
.form-field > span { color: var(--text-secondary); font-size: 11px; font-weight: 800; }
.form-field small { color: var(--text-tertiary); font-size: 10px; }
.full-field { grid-column: 1 / -1; }
.mono { font-family: var(--font-mono); }
.input:disabled { opacity: 0.68; cursor: not-allowed; }
.state-label { padding: 3px 8px; border-radius: 4px; font-size: 10px; font-weight: 800; }
.state-label.ready { background: rgba(34, 197, 94, 0.12); color: #86efac; }
.state-label.pending { background: var(--surface-3); color: var(--text-tertiary); }

.form-footer {
  min-height: 66px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 10px 20px;
  border-top: 1px solid var(--border);
  background: var(--surface-1);
}
.footer-status span,
.footer-status small { display: block; }
.footer-status span { font-size: 11px; font-weight: 800; }
.footer-status small { margin-top: 2px; color: var(--text-tertiary); font-size: 10px; }
.footer-actions { display: flex; gap: 8px; }
.valid-text { color: var(--success); }
.invalid-text { color: var(--danger); }

.scene-inspector {
  min-height: 0;
  overflow-y: auto;
  border-left: 1px solid var(--border);
  background: var(--surface-1);
}
.inspector-section { padding: 18px 16px; border-bottom: 1px solid var(--border); }
.metric-row { display: flex; align-items: center; justify-content: space-between; gap: 12px; margin-top: 11px; color: var(--text-tertiary); font-size: 11px; }
.metric-row strong { color: var(--text-secondary); font-size: 11px; }
.event-list { margin-top: 10px; overflow-wrap: anywhere; color: var(--warning); font-family: var(--font-mono); font-size: 10px; }
.fingerprint { margin-top: 12px; overflow-wrap: anywhere; color: var(--text-tertiary); font-family: var(--font-mono); font-size: 9px; line-height: 1.5; }
.diagnostic { display: grid; gap: 3px; margin-top: 10px; padding-left: 9px; border-left: 2px solid var(--border-light); }
.diagnostic strong { overflow-wrap: anywhere; font-family: var(--font-mono); font-size: 9px; }
.diagnostic span { color: var(--text-tertiary); font-size: 10px; line-height: 1.45; }
.diagnostic.error { border-color: var(--danger); }
.diagnostic.warning { border-color: var(--warning); }
.clean-state { margin-top: 12px; color: var(--text-tertiary); font-size: 11px; }

.empty-editor { min-width: 0; display: grid; place-content: center; gap: 10px; color: var(--text-tertiary); text-align: center; }
.empty-editor h2 { color: var(--text-secondary); font-size: 17px; }
.empty-mark { display: inline-grid; min-width: 44px; height: 44px; place-items: center; border: 1px solid var(--border); border-radius: var(--radius); background: var(--surface-2); color: var(--brand-light); font-family: var(--font-mono); font-weight: 900; }
.sr-only { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; }

.notice {
  position: fixed;
  right: 22px;
  bottom: 22px;
  z-index: 80;
  width: min(360px, calc(100vw - 44px));
  display: grid;
  gap: 3px;
  padding: 13px 15px;
  border: 1px solid var(--border-light);
  border-radius: var(--radius);
  background: var(--surface-3);
  color: var(--text-primary);
  box-shadow: var(--shadow-lg);
  text-align: left;
  cursor: pointer;
}
.notice strong { font-size: 12px; }
.notice span { color: var(--text-secondary); font-size: 11px; }
.notice.success { border-color: rgba(34, 197, 94, 0.55); }
.notice.error { border-color: rgba(239, 68, 68, 0.65); }

@media (max-width: 1180px) {
  .editor-workspace { grid-template-columns: minmax(220px, 250px) minmax(420px, 1fr); grid-template-rows: minmax(0, 1fr) auto; }
  .scene-inspector { grid-column: 1 / -1; max-height: 170px; display: grid; grid-template-columns: repeat(3, 1fr); border-top: 1px solid var(--border); border-left: none; }
  .inspector-section { border-right: 1px solid var(--border); border-bottom: none; }
}

@media (max-width: 760px) {
  .scene-editor { height: auto; min-height: 100svh; overflow: visible; }
  .editor-header { flex-direction: column; padding: 18px 16px 14px; }
  .header-actions { width: 100%; justify-content: flex-start; }
  .catalog-strip { flex-wrap: wrap; gap: 8px 16px; padding: 7px 16px; }
  .dirty-indicator { width: 100%; margin-left: 0; }
  .editor-workspace { display: block; }
  .scene-list { height: 244px; border-right: none; border-bottom: 1px solid var(--border); }
  .scene-form { min-height: 720px; }
  .scene-stage { flex-basis: 230px; }
  .scene-inspector { max-height: none; display: block; border-top: 1px solid var(--border); }
  .inspector-section { border-right: none; border-bottom: 1px solid var(--border); }
  .field-grid,
  .identity-grid,
  .atmosphere-grid { grid-template-columns: 1fr; }
  .full-field { grid-column: auto; }
  .form-footer { align-items: flex-start; }
  .footer-actions { flex-wrap: wrap; justify-content: flex-end; }
}

@media (max-width: 480px) {
  .header-actions .btn { flex: 1 1 auto; justify-content: center; }
  .catalog-strip span:nth-child(4) { display: none; }
  .scene-stage { min-height: 190px; flex-basis: 190px; }
  .stage-caption { left: 16px; right: 16px; bottom: 14px; }
  .stage-caption h2 { font-size: 18px; }
  .validation-banner { grid-template-columns: 1fr; gap: 2px; padding: 9px 14px; }
  .validation-banner span { white-space: normal; }
  .form-section { padding: 18px 16px 20px; }
  .section-heading { flex-direction: column; gap: 5px; }
  .source-path { max-width: 100%; }
  .form-footer { flex-direction: column; }
  .footer-actions { width: 100%; }
  .footer-actions .btn { flex: 1; justify-content: center; }
  .notice { bottom: calc(74px + env(safe-area-inset-bottom, 0px)); }
}
</style>
