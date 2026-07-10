<template>
  <div class="asset-workbench">
    <header class="workbench-header">
      <div class="title-lockup">
        <span class="title-icon"><Images :size="17" /></span>
        <div>
          <span class="eyebrow">{{ t('assets.eyebrow', 'Runtime catalog') }}</span>
          <h1>{{ t('assets.title', 'Scene Assets') }}</h1>
        </div>
      </div>

      <div class="header-metrics">
        <span><strong>{{ catalog?.scenes.length ?? 0 }}</strong>{{ t('assets.scenes', 'Scenes') }}</span>
        <span><strong>{{ catalog?.backgrounds.length ?? 0 }}</strong>{{ t('assets.backgrounds', 'Backgrounds') }}</span>
        <span :class="{ warning: issueCount > 0 }"><strong>{{ issueCount }}</strong>{{ t('assets.issues', 'Issues') }}</span>
      </div>

      <button
        class="icon-command"
        :disabled="isLoading"
        :title="t('common.refresh', 'Refresh')"
        :aria-label="t('common.refresh', 'Refresh')"
        @click="refreshCatalog(true)"
      ><RefreshCw :class="{ spinner: isLoading }" :size="15" /></button>
    </header>

    <aside class="scene-rail" :aria-label="t('assets.scenes', 'Scenes')">
      <div class="rail-search">
        <label class="search-field">
          <Search :size="14" />
          <input v-model="searchQuery" :placeholder="t('assets.search', 'Search scenes...')" />
          <button
            v-if="searchQuery"
            :title="t('assets.clear-search', 'Clear search')"
            :aria-label="t('assets.clear-search', 'Clear search')"
            @click="searchQuery = ''"
          ><X :size="12" /></button>
        </label>
      </div>

      <div class="scene-filters" :aria-label="t('assets.filters', 'Scene filters')">
        <button :class="{ active: sceneFilter === 'all' }" @click="sceneFilter = 'all'">{{ t('assets.filter-all', 'All') }}</button>
        <button :class="{ active: sceneFilter === 'active' }" @click="sceneFilter = 'active'">{{ t('assets.filter-active', 'Active') }}</button>
        <button :class="{ active: sceneFilter === 'missing' }" @click="sceneFilter = 'missing'">{{ t('assets.filter-missing', 'Missing') }}</button>
      </div>

      <div class="rail-summary">
        <span>{{ t('assets.visible-count', '{visible} of {total}', { visible: filteredScenes.length, total: scenes.length }) }}</span>
        <strong v-if="brokenSceneCount > 0"><AlertTriangle :size="11" />{{ brokenSceneCount }}</strong>
      </div>

      <div class="scene-list">
        <div v-if="isLoading && !catalog" class="rail-empty"><LoaderCircle class="spinner" :size="19" /><span>{{ t('common.loading', 'Loading...') }}</span></div>
        <div v-else-if="filteredScenes.length === 0" class="rail-empty"><ImageOff :size="20" /><span>{{ t('assets.no-assets', 'No assets found') }}</span></div>

        <button
          v-for="scene in filteredScenes"
          v-else
          :key="scene.id"
          class="scene-row"
          :class="{ selected: selectedScene?.id === scene.id, active: activeScene?.id === scene.id, broken: scene.background_path && !scene.background_exists }"
          :aria-current="selectedScene?.id === scene.id ? 'true' : undefined"
          @click="selectScene(scene)"
        >
          <span class="scene-thumb">
            <img v-if="previewAvailable(scene)" :src="scenePreviewUrl(scene) || ''" :alt="scene.name" @error="markPreviewFailed(scene)" />
            <ImageOff v-else :size="15" />
          </span>
          <span class="scene-copy">
            <strong>{{ scene.name }}</strong>
            <small>{{ scene.id }}</small>
            <span>{{ scene.background_path || t('assets.no-path', 'No background path') }}</span>
          </span>
          <span class="scene-state">
            <Play v-if="activeScene?.id === scene.id" :size="12" />
            <AlertTriangle v-else-if="scene.background_path && !scene.background_exists" :size="12" />
          </span>
        </button>
      </div>
    </aside>

    <main class="preview-panel">
      <header class="preview-header">
        <div>
          <span class="eyebrow">{{ t('assets.selected-scene', 'Selected scene') }}</span>
          <strong>{{ selectedScene?.name || t('assets.no-selection', 'No scene selected') }}</strong>
        </div>
        <div class="preview-actions">
          <span v-if="selectedScene && activeScene?.id === selectedScene.id" class="active-badge"><Play :size="11" />{{ t('assets.filter-active', 'Active') }}</span>
          <button
            class="icon-command inspector-toggle"
            :title="t('assets.open-inspector', 'Open inspector')"
            :aria-label="t('assets.open-inspector', 'Open inspector')"
            @click="compactInspectorOpen = true"
          ><PanelRightOpen :size="15" /></button>
        </div>
      </header>

      <template v-if="selectedScene">
        <div class="preview-stage">
          <img v-if="previewAvailable(selectedScene)" :src="scenePreviewUrl(selectedScene) || ''" :alt="selectedScene.name" @error="markPreviewFailed(selectedScene)" />
          <div v-else class="preview-unavailable">
            <ImageOff :size="28" />
            <strong>{{ t('assets.image-unavailable', 'Background unavailable') }}</strong>
            <code>{{ selectedScene.background_path || t('assets.no-path', 'No background path') }}</code>
          </div>
          <span class="source-badge">{{ selectedScene.source }}</span>
        </div>

        <footer class="preview-details">
          <div class="scene-identity">
            <div>
              <h2>{{ selectedScene.name }}</h2>
              <code>{{ selectedScene.id }}</code>
            </div>
            <button
              class="primary-command"
              :disabled="settingSceneId === selectedScene.id || !selectedScene.background_exists || activeScene?.id === selectedScene.id"
              :title="activeScene?.id === selectedScene.id ? t('assets.filter-active', 'Active') : t('assets.set-active', 'Set Active')"
              :aria-label="activeScene?.id === selectedScene.id ? t('assets.filter-active', 'Active') : t('assets.set-active', 'Set Active')"
              @click="activateScene(selectedScene)"
            >
              <LoaderCircle v-if="settingSceneId === selectedScene.id" class="spinner" :size="14" />
              <CheckCircle2 v-else-if="activeScene?.id === selectedScene.id" :size="14" />
              <Play v-else :size="14" />
              {{ activeScene?.id === selectedScene.id ? t('assets.filter-active', 'Active') : settingSceneId === selectedScene.id ? t('assets.setting', 'Setting') : t('assets.set-active', 'Set Active') }}
            </button>
          </div>
          <p class="asset-path">{{ selectedScene.background_path || t('assets.no-path', 'No background path') }}</p>
          <div class="scene-meta">
            <span v-if="selectedScene.time_of_day"><Clock3 :size="11" />{{ selectedScene.time_of_day }}</span>
            <span v-if="selectedScene.weather"><CloudSun :size="11" />{{ selectedScene.weather }}</span>
            <span v-for="tag in selectedScene.tags" :key="tag"><Hash :size="10" />{{ tag }}</span>
            <span v-if="selectedScene.background_path && !selectedScene.background_exists" class="missing-badge"><AlertTriangle :size="11" />{{ t('assets.missing-file', 'Missing file') }}</span>
          </div>
        </footer>
      </template>

      <div v-else class="preview-empty"><Images :size="27" /><span>{{ t('assets.no-selection', 'No scene selected') }}</span></div>
    </main>

    <aside class="inspector-panel" :class="{ 'compact-open': compactInspectorOpen }" :aria-label="t('assets.inspector', 'Asset inspector')">
      <header class="inspector-header">
        <div>
          <span class="eyebrow">{{ t('assets.inspector', 'Asset inspector') }}</span>
          <strong>{{ catalog?.project_path || t('assets.project-preview', 'Preview project') }}</strong>
        </div>
        <button
          class="icon-command inspector-close"
          :title="t('assets.close-inspector', 'Close inspector')"
          :aria-label="t('assets.close-inspector', 'Close inspector')"
          @click="compactInspectorOpen = false"
        ><X :size="14" /></button>
      </header>

      <nav class="inspector-tabs" :aria-label="t('assets.inspector-tabs', 'Inspector views')">
        <button :class="{ active: inspectorTab === 'runtime' }" :title="t('assets.runtime', 'Runtime')" :aria-label="t('assets.runtime', 'Runtime')" @click="inspectorTab = 'runtime'"><Play :size="12" /><span>{{ t('assets.runtime', 'Runtime') }}</span></button>
        <button :class="{ active: inspectorTab === 'diagnostics' }" :title="t('assets.diagnostics', 'Diagnostics')" :aria-label="t('assets.diagnostics', 'Diagnostics')" @click="inspectorTab = 'diagnostics'"><ListChecks :size="12" /><span>{{ t('assets.diagnostics', 'Diagnostics') }}</span></button>
        <button :class="{ active: inspectorTab === 'backgrounds' }" :title="t('assets.backgrounds', 'Backgrounds')" :aria-label="t('assets.backgrounds', 'Backgrounds')" @click="inspectorTab = 'backgrounds'"><Images :size="12" /><span>{{ t('assets.backgrounds', 'Backgrounds') }}</span></button>
      </nav>

      <div class="inspector-scroll">
        <section v-if="inspectorTab === 'runtime'" class="inspector-section">
          <div class="section-heading"><span>{{ t('assets.active', 'Active Scene') }}</span><strong>{{ activeScene ? t('assets.ready', 'Ready') : t('assets.idle', 'Idle') }}</strong></div>
          <div v-if="activeScene" class="runtime-scene">
            <span class="runtime-icon"><Play :size="16" /></span>
            <div><strong>{{ activeScene.name }}</strong><code>{{ activeScene.id }}</code></div>
          </div>
          <div v-else class="compact-empty"><Play :size="19" /><span>{{ t('assets.no-active', 'No active scene') }}</span></div>
          <p v-if="activeScene" class="runtime-path">{{ activeScene.background_path || t('assets.no-path', 'No background path') }}</p>

          <div class="section-heading history-heading"><span>{{ t('assets.history', 'Scene history') }}</span><strong>{{ activeState?.scene_history.length ?? 0 }}</strong></div>
          <div v-if="activeState?.scene_history.length" class="history-list">
            <button v-for="(sceneId, index) in activeState.scene_history.slice().reverse()" :key="`${sceneId}-${index}`" @click="selectSceneById(sceneId)"><Clock3 :size="11" /><span>{{ sceneName(sceneId) }}</span><code>{{ sceneId }}</code></button>
          </div>
          <div v-else class="compact-empty"><Clock3 :size="18" /><span>{{ t('assets.no-history', 'No scene history') }}</span></div>
        </section>

        <section v-else-if="inspectorTab === 'diagnostics'" class="inspector-section">
          <div class="diagnostic-summary" :class="{ warning: !catalog?.valid }">
            <CheckCircle2 v-if="catalog?.valid" :size="18" />
            <AlertTriangle v-else :size="18" />
            <div><strong>{{ catalog?.valid ? t('assets.clean', 'Clean') : t('assets.attention', 'Attention') }}</strong><span>{{ t('assets.issue-summary', '{errors} errors, {warnings} warnings', { errors: catalog?.error_count ?? 0, warnings: catalog?.warning_count ?? 0 }) }}</span></div>
          </div>
          <div v-if="catalog?.issues.length" class="issue-list">
            <article v-for="(issue, index) in catalog.issues" :key="`${issue.code}-${index}`" class="issue-row" :class="issue.severity">
              <div><strong>{{ severityLabel(issue.severity) }}</strong><code>{{ issue.code }}</code></div>
              <span>{{ issueTarget(issue) }}</span>
              <p>{{ issue.message }}</p>
            </article>
          </div>
          <div v-else class="compact-empty"><CheckCircle2 :size="19" /><span>{{ t('assets.no-issues', 'No catalog issues') }}</span></div>
        </section>

        <section v-else class="inspector-section">
          <div class="section-heading"><span>{{ t('assets.backgrounds', 'Backgrounds') }}</span><strong>{{ catalog?.backgrounds.length ?? 0 }}</strong></div>
          <div class="background-list">
            <button v-for="asset in catalog?.backgrounds" :key="asset.relative_path" @click="selectSceneById(asset.linked_scene_id)">
              <span class="background-thumb">
                <img v-if="backgroundPreviewAvailable(asset)" :src="backgroundPreviewUrl(asset) || ''" :alt="asset.file_name" @error="markBackgroundFailed(asset)" />
                <ImageOff v-else :size="14" />
              </span>
              <span class="background-copy"><strong>{{ asset.file_name }}</strong><small>{{ asset.relative_path }}</small></span>
              <span class="background-meta"><strong>{{ formatBytes(asset.file_size) }}</strong><small>{{ asset.linked_scene_id ? t('assets.linked', 'Linked') : t('assets.unlinked', 'Unlinked') }}</small></span>
            </button>
          </div>
        </section>
      </div>
    </aside>

    <Transition name="fade"><button v-if="toast" class="status-toast" :class="toastType" @click="toast = ''">{{ toast }}</button></Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { AlertTriangle, CheckCircle2, Clock3, CloudSun, Hash, ImageOff, Images, ListChecks, LoaderCircle, PanelRightOpen, Play, RefreshCw, Search, X } from '@lucide/vue'
import { resolveAssetUrl } from '../lib/assets'
import { useI18n } from '../lib/i18n'
import { invokeCommand } from '../lib/tauri'

interface SceneInfo {
  id: string
  name: string
  background_path: string | null
  bgm_path: string | null
  weather: string | null
  time_of_day: string | null
  tags: string[]
  source: string
  background_exists: boolean
  absolute_background_path: string | null
}

interface BackgroundAsset {
  id: string
  file_name: string
  relative_path: string
  absolute_path: string
  extension: string
  file_size: number
  linked_scene_id: string | null
}

interface SceneAssetIssue {
  severity: string
  code: string
  scene_id: string | null
  path: string | null
  message: string
}

interface SceneAssetCatalog {
  project_path: string | null
  valid: boolean
  error_count: number
  warning_count: number
  scenes: SceneInfo[]
  backgrounds: BackgroundAsset[]
  issues: SceneAssetIssue[]
}

interface ActiveScene {
  scene: SceneInfo | null
  scene_history: string[]
}

type SceneFilter = 'all' | 'active' | 'missing'
type InspectorTab = 'runtime' | 'diagnostics' | 'backgrounds'

const { t } = useI18n()
const catalog = ref<SceneAssetCatalog | null>(null)
const activeState = ref<ActiveScene | null>(null)
const selectedScene = ref<SceneInfo | null>(null)
const searchQuery = ref('')
const sceneFilter = ref<SceneFilter>('all')
const inspectorTab = ref<InspectorTab>('runtime')
const compactInspectorOpen = ref(false)
const isLoading = ref(false)
const settingSceneId = ref('')
const failedPreviewUrls = ref<string[]>([])
const toast = ref('')
const toastType = ref<'success' | 'error'>('success')
const activeSceneStorageKey = 'monogatari.activeScene'
let toastTimer: number | null = null

const previewCatalog: SceneAssetCatalog = {
  project_path: null,
  valid: true,
  error_count: 0,
  warning_count: 0,
  scenes: [
    {
      id: 'sakura_park',
      name: 'Sakura Park',
      background_path: 'assets/backgrounds/sakura_park.svg',
      bgm_path: null,
      weather: 'spring',
      time_of_day: 'day',
      tags: ['outdoor', 'calm'],
      source: 'preview',
      background_exists: true,
      absolute_background_path: null,
    },
    {
      id: 'studio_night',
      name: 'Studio Night',
      background_path: 'assets/backgrounds/studio_night.svg',
      bgm_path: null,
      weather: 'clear',
      time_of_day: 'night',
      tags: ['indoor', 'focus'],
      source: 'preview',
      background_exists: true,
      absolute_background_path: null,
    },
  ],
  backgrounds: [
    {
      id: 'sakura_park',
      file_name: 'sakura_park.svg',
      relative_path: 'assets/backgrounds/sakura_park.svg',
      absolute_path: '',
      extension: 'svg',
      file_size: 2257,
      linked_scene_id: 'sakura_park',
    },
    {
      id: 'studio_night',
      file_name: 'studio_night.svg',
      relative_path: 'assets/backgrounds/studio_night.svg',
      absolute_path: '',
      extension: 'svg',
      file_size: 1916,
      linked_scene_id: 'studio_night',
    },
  ],
  issues: [],
}

const scenes = computed(() => catalog.value?.scenes || [])
const activeScene = computed(() => activeState.value?.scene || null)
const issueCount = computed(() => (catalog.value?.error_count || 0) + (catalog.value?.warning_count || 0))
const brokenSceneCount = computed(() => scenes.value.filter(scene => scene.background_path && !scene.background_exists).length)
const filteredScenes = computed(() => {
  const query = searchQuery.value.trim().toLowerCase()
  return scenes.value.filter(scene => {
    if (sceneFilter.value === 'active' && activeScene.value?.id !== scene.id) return false
    if (sceneFilter.value === 'missing' && (!scene.background_path || scene.background_exists)) return false
    if (!query) return true
    return scene.id.toLowerCase().includes(query)
      || scene.name.toLowerCase().includes(query)
      || (scene.background_path || '').toLowerCase().includes(query)
      || scene.tags.some(tag => tag.toLowerCase().includes(query))
  })
})

async function refreshCatalog(showNotice = false) {
  isLoading.value = true
  try {
    const nextCatalog = await invokeCommand<SceneAssetCatalog>('list_scene_assets', undefined, previewCatalog)
    const nextActiveState = await invokeCommand<ActiveScene>(
      'get_current_scene',
      undefined,
      () => previewActiveState(nextCatalog),
    )
    catalog.value = nextCatalog
    activeState.value = normalizeActiveState(nextActiveState, nextCatalog)
    const selectedId = selectedScene.value?.id || activeState.value.scene?.id
    selectedScene.value = nextCatalog.scenes.find(scene => scene.id === selectedId) || nextCatalog.scenes[0] || null
    failedPreviewUrls.value = []
    if (showNotice) notify('success', t('assets.notice.refreshed', 'Scene catalog refreshed.'))
  } catch (error) {
    catalog.value = previewCatalog
    activeState.value = previewActiveState(previewCatalog)
    selectedScene.value = previewCatalog.scenes[0] || null
    notify('error', t('assets.notice.load-failed', 'Scene catalog could not be loaded: {error}', { error: String(error) }))
  } finally {
    isLoading.value = false
  }
}

function selectScene(scene: SceneInfo) {
  selectedScene.value = scene
  trackView(scene.name, scene.id, 'scenes')
}

function selectSceneById(sceneId: string | null) {
  if (!sceneId) return
  const scene = scenes.value.find(item => item.id === sceneId)
  if (scene) selectScene(scene)
}

async function activateScene(scene: SceneInfo) {
  if (!scene.background_exists || settingSceneId.value) return
  settingSceneId.value = scene.id
  try {
    const selected = await invokeCommand<SceneInfo>('set_scene', { sceneId: scene.id }, () => scene)
    const catalogScene = scenes.value.find(item => item.id === selected.id) || selected
    activeState.value = {
      scene: catalogScene,
      scene_history: [...(activeState.value?.scene_history || []), catalogScene.id].slice(-24),
    }
    localStorage.setItem(activeSceneStorageKey, JSON.stringify(catalogScene))
    notify('success', t('assets.notice.active', 'Active scene: {name}', { name: catalogScene.name }))
  } catch (error) {
    notify('error', t('assets.notice.activate-failed', 'Scene could not be activated: {error}', { error: String(error) }))
  } finally {
    settingSceneId.value = ''
  }
}

function previewActiveState(sourceCatalog: SceneAssetCatalog): ActiveScene {
  const stored = localStorage.getItem(activeSceneStorageKey)
  if (stored) {
    try {
      const storedScene = JSON.parse(stored) as SceneInfo
      const scene = sourceCatalog.scenes.find(item => item.id === storedScene.id)
      if (scene) return { scene, scene_history: [scene.id] }
    } catch {
      localStorage.removeItem(activeSceneStorageKey)
    }
  }
  const scene = sourceCatalog.scenes[0] || null
  return { scene, scene_history: scene ? [scene.id] : [] }
}

function normalizeActiveState(state: ActiveScene, sourceCatalog: SceneAssetCatalog): ActiveScene {
  const scene = state.scene ? sourceCatalog.scenes.find(item => item.id === state.scene?.id) || state.scene : null
  return { scene, scene_history: state.scene_history || [] }
}

function scenePreviewUrl(scene: SceneInfo): string | null {
  return resolveAssetUrl(scene.absolute_background_path || scene.background_path)
}

function backgroundPreviewUrl(asset: BackgroundAsset): string | null {
  return resolveAssetUrl(asset.absolute_path || asset.relative_path)
}

function previewAvailable(scene: SceneInfo): boolean {
  const url = scenePreviewUrl(scene)
  return Boolean(scene.background_exists && url && !failedPreviewUrls.value.includes(url))
}

function backgroundPreviewAvailable(asset: BackgroundAsset): boolean {
  const url = backgroundPreviewUrl(asset)
  return Boolean(url && !failedPreviewUrls.value.includes(url))
}

function markPreviewFailed(scene: SceneInfo) {
  const url = scenePreviewUrl(scene)
  if (url && !failedPreviewUrls.value.includes(url)) failedPreviewUrls.value = [...failedPreviewUrls.value, url]
}

function markBackgroundFailed(asset: BackgroundAsset) {
  const url = backgroundPreviewUrl(asset)
  if (url && !failedPreviewUrls.value.includes(url)) failedPreviewUrls.value = [...failedPreviewUrls.value, url]
}

function sceneName(id: string): string {
  return scenes.value.find(scene => scene.id === id)?.name || id
}

function issueTarget(issue: SceneAssetIssue): string {
  return issue.scene_id || issue.path || t('assets.catalog', 'Catalog')
}

function severityLabel(severity: string): string {
  if (severity.toLowerCase() === 'error') return t('common.error', 'Error')
  if (severity.toLowerCase() === 'warning') return t('assets.warnings', 'Warnings')
  return severity
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

function trackView(name: string, id: string, type: string) {
  if (typeof window === 'undefined') return
  const trackingWindow = window as typeof window & { __monogatari_track?: (event: Record<string, string>) => void }
  trackingWindow.__monogatari_track?.({ icon: 'S', name, id, type, path: '/assets' })
}

function notify(type: 'success' | 'error', message: string) {
  toastType.value = type
  toast.value = message
  if (toastTimer !== null) window.clearTimeout(toastTimer)
  toastTimer = window.setTimeout(() => { toast.value = '' }, 3600)
}

onMounted(() => refreshCatalog())
onUnmounted(() => {
  if (toastTimer !== null) window.clearTimeout(toastTimer)
})
</script>

<style scoped>
.asset-workbench {
  position: relative;
  display: grid;
  height: calc(100svh - 56px);
  min-height: 0;
  grid-template-columns: 280px minmax(0, 1fr) 310px;
  grid-template-rows: 54px minmax(0, 1fr);
  overflow: hidden;
  background: var(--surface-0);
}

.workbench-header {
  display: grid;
  min-width: 0;
  grid-column: 1 / -1;
  grid-template-columns: minmax(190px, 1fr) auto 32px;
  align-items: center;
  gap: 12px;
  padding: 7px 11px;
  border-bottom: 1px solid var(--border);
  background: var(--surface-1);
}

.title-lockup,
.header-metrics,
.panel-heading,
.scene-filters,
.rail-summary,
.preview-actions,
.active-badge,
.scene-meta,
.inspector-tabs,
.section-heading,
.diagnostic-summary,
.history-list button,
.issue-row > div {
  display: flex;
  align-items: center;
}

.title-lockup { min-width: 0; gap: 9px; }
.title-icon,
.runtime-icon {
  display: inline-grid;
  flex: 0 0 auto;
  place-items: center;
  border-radius: 6px;
  background: color-mix(in srgb, var(--brand) 14%, var(--surface-2));
  color: var(--brand-light);
}
.title-icon { width: 32px; height: 32px; }
.title-lockup > div, .preview-header > div:first-child, .inspector-header > div, .runtime-scene > div { display: grid; min-width: 0; gap: 2px; }
.eyebrow { color: var(--text-tertiary); font-size: 8px; font-weight: 800; letter-spacing: 0; text-transform: uppercase; }
.title-lockup h1 { margin: 0; color: var(--text-primary); font-size: 14px; line-height: 1.2; }
.header-metrics { gap: 4px; }
.header-metrics > span { display: grid; min-width: 62px; gap: 1px; padding: 3px 7px; border-left: 1px solid var(--border); color: var(--text-tertiary); font-size: 7px; text-transform: uppercase; }
.header-metrics strong { color: var(--text-primary); font-family: var(--font-mono); font-size: 10px; }
.header-metrics .warning strong { color: var(--warning); }

.icon-command,
.primary-command {
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
.primary-command { min-height: 32px; padding: 0 10px; border: 1px solid var(--brand); background: var(--brand); color: var(--surface-0); }
.icon-command:hover:not(:disabled) { border-color: var(--border-strong); color: var(--text-primary); }
.primary-command:hover:not(:disabled) { border-color: var(--brand-light); background: var(--brand-light); }
.icon-command:disabled, .primary-command:disabled { cursor: not-allowed; opacity: 0.42; }

.scene-rail,
.preview-panel,
.inspector-panel { min-width: 0; min-height: 0; overflow: hidden; }
.scene-rail { display: grid; grid-template-rows: 42px 36px 28px minmax(0, 1fr); border-right: 1px solid var(--border); background: var(--surface-1); }
.rail-search { padding: 6px 8px; border-bottom: 1px solid var(--border); }
.search-field { display: flex; height: 30px; align-items: center; gap: 6px; padding: 0 7px; border: 1px solid var(--border); border-radius: 6px; background: var(--surface-0); color: var(--text-tertiary); }
.search-field:focus-within { border-color: var(--border-strong); color: var(--text-secondary); }
.search-field input { width: 100%; min-width: 0; border: 0; outline: 0; background: transparent; color: var(--text-primary); font: inherit; font-size: 9px; }
.search-field input::placeholder { color: var(--text-tertiary); }
.search-field button { display: inline-grid; width: 22px; height: 22px; flex: 0 0 22px; place-items: center; padding: 0; border: 0; background: transparent; color: var(--text-tertiary); cursor: pointer; }
.scene-filters { gap: 3px; padding: 4px 7px; border-bottom: 1px solid var(--border); }
.scene-filters button { min-width: 0; height: 27px; flex: 1; padding: 0 5px; border: 1px solid transparent; border-radius: 5px; background: transparent; color: var(--text-tertiary); font: inherit; font-size: 8px; cursor: pointer; }
.scene-filters button.active { border-color: var(--border); background: var(--surface-2); color: var(--text-primary); }
.rail-summary { justify-content: space-between; gap: 8px; padding: 3px 9px; color: var(--text-tertiary); font-size: 8px; }
.rail-summary strong { display: inline-flex; align-items: center; gap: 3px; color: var(--warning); font-family: var(--font-mono); font-size: 8px; }
.scene-list, .inspector-scroll { min-height: 0; overflow-y: auto; scrollbar-width: none; }
.scene-list::-webkit-scrollbar, .inspector-scroll::-webkit-scrollbar { display: none; }
.scene-list { padding: 4px 6px 8px; }
.scene-row { display: grid; width: 100%; min-width: 0; min-height: 64px; grid-template-columns: 54px minmax(0, 1fr) 18px; align-items: center; gap: 8px; margin-bottom: 3px; padding: 6px; border: 1px solid transparent; border-radius: 6px; background: transparent; color: inherit; font: inherit; text-align: left; cursor: pointer; }
.scene-row:hover { background: var(--surface-2); }
.scene-row.selected { border-color: color-mix(in srgb, var(--brand) 34%, var(--border)); background: color-mix(in srgb, var(--brand) 8%, var(--surface-1)); }
.scene-row.broken { border-color: color-mix(in srgb, var(--danger) 30%, transparent); }
.scene-thumb, .background-thumb { display: grid; overflow: hidden; place-items: center; border-radius: 5px; background: var(--surface-3); color: var(--text-tertiary); }
.scene-thumb { width: 54px; height: 42px; }
.scene-thumb img, .background-thumb img { width: 100%; height: 100%; object-fit: cover; }
.scene-copy { display: grid; min-width: 0; gap: 2px; }
.scene-copy strong, .scene-copy small, .scene-copy span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.scene-copy strong { color: var(--text-primary); font-size: 9px; }
.scene-copy small { color: var(--text-tertiary); font-family: var(--font-mono); font-size: 7px; }
.scene-copy span { color: var(--text-secondary); font-size: 7px; }
.scene-state { display: grid; place-items: center; color: var(--text-tertiary); }
.scene-row.active .scene-state { color: var(--brand-light); }
.scene-row.broken .scene-state { color: var(--danger); }
.rail-empty, .preview-empty, .compact-empty { display: grid; min-height: 100%; place-items: center; align-content: center; gap: 7px; padding: 18px; color: var(--text-tertiary); font-size: 9px; text-align: center; }

.preview-panel { display: grid; grid-template-rows: 54px minmax(220px, 1fr) auto; background: var(--surface-0); }
.preview-header, .inspector-header { display: flex; min-width: 0; align-items: center; justify-content: space-between; gap: 9px; padding: 7px 10px; border-bottom: 1px solid var(--border); background: var(--surface-1); }
.preview-header strong, .inspector-header strong { overflow: hidden; color: var(--text-primary); font-size: 10px; text-overflow: ellipsis; white-space: nowrap; }
.preview-actions { min-width: 0; justify-content: flex-end; gap: 5px; }
.active-badge { gap: 4px; padding: 4px 7px; border: 1px solid color-mix(in srgb, var(--brand) 32%, var(--border)); border-radius: 999px; color: var(--brand-light); font-size: 8px; font-weight: 800; }
.inspector-toggle { display: none; }
.preview-stage { position: relative; display: grid; min-height: 0; place-items: center; overflow: hidden; background: var(--surface-2); }
.preview-stage > img { width: 100%; height: 100%; object-fit: contain; }
.preview-unavailable { display: grid; max-width: 380px; justify-items: center; gap: 7px; padding: 24px; color: var(--text-tertiary); text-align: center; }
.preview-unavailable strong { color: var(--text-secondary); font-size: 11px; }
.preview-unavailable code { max-width: 100%; overflow-wrap: anywhere; font-family: var(--font-mono); font-size: 8px; }
.source-badge { position: absolute; top: 10px; left: 10px; padding: 4px 6px; border: 1px solid rgba(255,255,255,0.22); border-radius: 5px; background: rgba(7,12,18,0.72); color: rgba(255,255,255,0.9); font-size: 7px; font-weight: 800; text-transform: uppercase; }
.preview-details { display: grid; gap: 7px; padding: 11px 12px 12px; border-top: 1px solid var(--border); background: var(--surface-1); }
.scene-identity { display: flex; min-width: 0; align-items: center; justify-content: space-between; gap: 10px; }
.scene-identity > div { display: grid; min-width: 0; gap: 2px; }
.scene-identity h2 { margin: 0; overflow: hidden; color: var(--text-primary); font-size: 14px; text-overflow: ellipsis; white-space: nowrap; }
.scene-identity code, .asset-path { overflow: hidden; color: var(--text-tertiary); font-family: var(--font-mono); font-size: 8px; text-overflow: ellipsis; white-space: nowrap; }
.asset-path { margin: 0; }
.scene-meta { flex-wrap: wrap; gap: 4px; }
.scene-meta span { display: inline-flex; min-height: 23px; align-items: center; gap: 3px; padding: 3px 6px; border: 1px solid var(--border); border-radius: 5px; color: var(--text-secondary); font-size: 8px; }
.scene-meta .missing-badge { border-color: color-mix(in srgb, var(--danger) 38%, var(--border)); color: var(--danger); }

.inspector-panel { display: grid; grid-template-rows: 54px 42px minmax(0, 1fr); border-left: 1px solid var(--border); background: var(--surface-1); }
.inspector-close { display: none; }
.inspector-tabs { min-width: 0; gap: 2px; padding: 5px 6px; border-bottom: 1px solid var(--border); }
.inspector-tabs button { display: inline-flex; min-width: 0; height: 30px; flex: 1; align-items: center; justify-content: center; gap: 4px; padding: 0 4px; border: 1px solid transparent; border-radius: 5px; background: transparent; color: var(--text-tertiary); font: inherit; font-size: 8px; cursor: pointer; }
.inspector-tabs button span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.inspector-tabs button.active { border-color: var(--border); background: var(--surface-2); color: var(--text-primary); }
.inspector-section { display: grid; align-content: start; gap: 11px; padding: 12px; }
.section-heading { min-width: 0; justify-content: space-between; gap: 8px; color: var(--text-tertiary); font-size: 8px; font-weight: 800; text-transform: uppercase; }
.section-heading strong { color: var(--text-primary); font-family: var(--font-mono); font-size: 8px; }
.runtime-scene { display: grid; min-width: 0; grid-template-columns: 34px minmax(0, 1fr); align-items: center; gap: 8px; padding: 8px 0; border-bottom: 1px solid var(--border); }
.runtime-icon { width: 34px; height: 34px; }
.runtime-scene strong { overflow: hidden; color: var(--text-primary); font-size: 10px; text-overflow: ellipsis; white-space: nowrap; }
.runtime-scene code, .runtime-path { overflow-wrap: anywhere; color: var(--text-tertiary); font-family: var(--font-mono); font-size: 8px; }
.runtime-path { margin: 0; line-height: 1.5; }
.history-heading { margin-top: 5px; }
.history-list { display: grid; gap: 3px; }
.history-list button { display: grid; min-width: 0; min-height: 34px; grid-template-columns: 15px minmax(0, 1fr); gap: 2px 6px; padding: 5px 6px; border: 1px solid transparent; border-radius: 5px; background: transparent; color: var(--text-tertiary); font: inherit; text-align: left; cursor: pointer; }
.history-list button:hover { border-color: var(--border); background: var(--surface-2); }
.history-list button span { overflow: hidden; color: var(--text-secondary); font-size: 8px; text-overflow: ellipsis; white-space: nowrap; }
.history-list button code { grid-column: 2; overflow: hidden; font-family: var(--font-mono); font-size: 7px; text-overflow: ellipsis; white-space: nowrap; }
.compact-empty { min-height: 130px; }
.diagnostic-summary { gap: 8px; padding: 9px; border: 1px solid color-mix(in srgb, var(--success) 28%, var(--border)); border-radius: 6px; color: var(--success); }
.diagnostic-summary.warning { border-color: color-mix(in srgb, var(--warning) 35%, var(--border)); color: var(--warning); }
.diagnostic-summary > div { display: grid; min-width: 0; gap: 2px; }
.diagnostic-summary strong { color: var(--text-primary); font-size: 9px; }
.diagnostic-summary span { color: var(--text-tertiary); font-size: 7px; }
.issue-list, .background-list { display: grid; gap: 4px; }
.issue-row { display: grid; gap: 5px; padding: 8px; border: 1px solid var(--border); border-radius: 6px; }
.issue-row.error { border-color: color-mix(in srgb, var(--danger) 34%, var(--border)); }
.issue-row.warning { border-color: color-mix(in srgb, var(--warning) 34%, var(--border)); }
.issue-row > div { justify-content: space-between; gap: 6px; }
.issue-row strong { color: var(--warning); font-size: 8px; text-transform: uppercase; }
.issue-row.error strong { color: var(--danger); }
.issue-row code, .issue-row > span { overflow-wrap: anywhere; color: var(--text-tertiary); font-family: var(--font-mono); font-size: 7px; }
.issue-row p { margin: 0; color: var(--text-secondary); font-size: 8px; line-height: 1.45; }
.background-list button { display: grid; min-width: 0; min-height: 49px; grid-template-columns: 52px minmax(0, 1fr) auto; align-items: center; gap: 7px; padding: 5px; border: 1px solid var(--border); border-radius: 6px; background: var(--surface-0); color: inherit; font: inherit; text-align: left; cursor: pointer; }
.background-list button:hover { border-color: var(--border-strong); }
.background-thumb { width: 52px; height: 36px; }
.background-copy, .background-meta { display: grid; min-width: 0; gap: 2px; }
.background-copy strong, .background-copy small { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.background-copy strong { color: var(--text-primary); font-size: 8px; }
.background-copy small { color: var(--text-tertiary); font-family: var(--font-mono); font-size: 6px; }
.background-meta { justify-items: end; }
.background-meta strong { color: var(--text-secondary); font-family: var(--font-mono); font-size: 7px; }
.background-meta small { color: var(--text-tertiary); font-size: 7px; }

.status-toast { position: fixed; z-index: 100; right: 18px; bottom: 18px; max-width: min(440px, calc(100vw - 28px)); padding: 9px 11px; border: 1px solid color-mix(in srgb, var(--success) 38%, var(--border)); border-radius: 6px; background: color-mix(in srgb, var(--success) 18%, var(--surface-1)); color: var(--text-primary); box-shadow: var(--shadow-lg); font: inherit; font-size: 9px; cursor: pointer; }
.status-toast.error { border-color: color-mix(in srgb, var(--danger) 44%, var(--border)); background: color-mix(in srgb, var(--danger) 18%, var(--surface-1)); }
.fade-enter-active, .fade-leave-active { transition: opacity 0.18s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
.spinner { animation: spin 0.8s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }

@media (max-width: 1450px) {
  .asset-workbench { grid-template-columns: 280px minmax(0, 1fr); }
  .inspector-panel { position: absolute; z-index: 40; inset: 54px 0 0 auto; display: none; width: min(330px, 100%); border-left: 1px solid var(--border-strong); box-shadow: var(--shadow-lg); }
  .inspector-panel.compact-open { display: grid; }
  .inspector-toggle, .inspector-close { display: inline-flex; }
}

@media (min-width: 1200px) and (max-width: 1450px) {
  .asset-workbench:has(.inspector-panel.compact-open) .preview-panel { margin-right: 330px; }
}

@media (max-width: 760px) {
  .asset-workbench { height: calc(100svh - 56px - 60px - env(safe-area-inset-bottom, 0px)); grid-template-columns: 1fr; grid-template-rows: 56px 166px minmax(0, 1fr); }
  .workbench-header { grid-template-columns: minmax(0, 1fr) 32px; gap: 7px; padding: 7px 9px; }
  .header-metrics { display: none; }
  .scene-rail { grid-row: 2; grid-template-rows: 38px 34px 24px minmax(0, 1fr); border-right: 0; border-bottom: 1px solid var(--border); }
  .rail-search { padding: 4px 7px; }
  .search-field { height: 28px; }
  .scene-filters { padding: 3px 7px; }
  .scene-filters button { height: 26px; }
  .rail-summary { padding: 2px 8px; }
  .scene-list { display: flex; gap: 5px; padding: 4px 7px; overflow-x: auto; overflow-y: hidden; }
  .scene-row { min-width: 190px; height: 60px; margin: 0; }
  .rail-empty { min-width: 100%; min-height: 60px; grid-auto-flow: column; }
  .preview-panel { grid-row: 3; grid-template-rows: 50px minmax(150px, 1fr) auto; }
  .preview-header { padding: 6px 8px; }
  .preview-details { padding: 9px; }
  .scene-identity h2 { font-size: 12px; }
  .inspector-panel { inset: 56px 0 0; width: 100%; }
  .status-toast { right: 14px; bottom: calc(70px + env(safe-area-inset-bottom, 0px)); }
}

@media (max-width: 430px) {
  .preview-details .primary-command { width: 34px; padding: 0; font-size: 0; }
  .scene-meta span:nth-of-type(n + 5) { display: none; }
  .inspector-tabs button span { display: none; }
}
</style>
