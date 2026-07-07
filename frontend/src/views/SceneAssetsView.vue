<template>
  <div class="scene-page">
    <header class="page-header">
      <div>
        <span class="eyebrow">Scene Library</span>
        <h1>{{ t("assets.title", "Scene Assets") }}</h1>
        <p>{{ catalog?.project_path || 'Preview project' }}</p>
      </div>
      <div class="header-actions">
        <button class="btn btn-secondary btn-sm" @click="refreshCatalog">Refresh</button>
      </div>
    </header>

    <section class="status-strip">
      <div class="status-cell">
        <span>Scenes</span>
        <strong>{{ catalog?.scenes.length ?? 0 }}</strong>
      </div>
      <div class="status-cell">
        <span>Backgrounds</span>
        <strong>{{ catalog?.backgrounds.length ?? 0 }}</strong>
      </div>
      <div class="status-cell" :class="{ danger: (catalog?.error_count || 0) > 0 }">
        <span>Errors</span>
        <strong>{{ catalog?.error_count ?? 0 }}</strong>
      </div>
      <div class="status-cell" :class="{ warning: (catalog?.warning_count || 0) > 0 }">
        <span>Warnings</span>
        <strong>{{ catalog?.warning_count ?? 0 }}</strong>
      </div>
    </section>

    <main class="asset-layout">
      <section class="scene-grid">
        <article
          v-for="scene in scenes"
          :key="scene.id"
          class="scene-card"
          :class="{ active: activeScene?.id === scene.id, broken: scene.background_path && !scene.background_exists }"
        >
          <div class="scene-preview" :style="scenePreviewStyle(scene)">
            <span>{{ scene.source }}</span>
          </div>
          <div class="scene-card-body">
            <div class="scene-title">
              <div>
                <strong>{{ scene.name }}</strong>
                <span>{{ scene.id }}</span>
              </div>
              <b v-if="activeScene?.id === scene.id">Active</b>
            </div>
            <div class="scene-meta">
              <span v-if="scene.time_of_day">{{ scene.time_of_day }}</span>
              <span v-if="scene.weather">{{ scene.weather }}</span>
              <span v-for="tag in scene.tags" :key="tag">{{ tag }}</span>
            </div>
            <p class="asset-path">{{ scene.background_path || 'No background path' }}</p>
            <div class="scene-actions">
              <button class="btn btn-primary btn-sm" :disabled="settingSceneId === scene.id" @click="activateScene(scene)">
                {{ settingSceneId === scene.id ? 'Setting' : 'Set Active' }}
              </button>
              <span v-if="scene.background_path && !scene.background_exists" class="inline-error">Missing file</span>
            </div>
          </div>
        </article>

        <div v-if="!isLoading && scenes.length === 0" class="empty-state">
          <span class="empty-mark">SC</span>
          <strong>No scene assets found</strong>
          <p>The active project has no scene metadata or background images.</p>
        </div>
      </section>

      <aside class="inspector">
        <section class="panel">
          <div class="panel-head">
            <span class="eyebrow">Runtime</span>
            <strong>{{ activeScene?.name || 'No active scene' }}</strong>
          </div>
          <p class="muted">{{ activeScene?.background_path || 'Scene runtime is waiting for selection.' }}</p>
          <div v-if="activeState?.scene_history.length" class="history-list">
            <span v-for="sceneId in activeState.scene_history.slice().reverse()" :key="sceneId">{{ sceneId }}</span>
          </div>
        </section>

        <section class="panel">
          <div class="panel-head">
            <span class="eyebrow">Diagnostics</span>
            <strong :class="catalog?.valid ? 'ok' : 'bad'">{{ catalog?.valid ? 'Clean' : 'Attention' }}</strong>
          </div>
          <div v-if="catalog?.issues.length" class="issue-list">
            <div v-for="(issue, index) in catalog.issues" :key="`${issue.code}-${index}`" class="issue-item" :class="issue.severity">
              <span>{{ issue.severity }} · {{ issue.code }}</span>
              <strong>{{ issue.scene_id || issue.path || 'catalog' }}</strong>
              <p>{{ issue.message }}</p>
            </div>
          </div>
          <p v-else class="muted">No catalog issues detected.</p>
        </section>

        <section class="panel">
          <div class="panel-head">
            <span class="eyebrow">Backgrounds</span>
            <strong>{{ catalog?.backgrounds.length ?? 0 }}</strong>
          </div>
          <div class="background-list">
            <div v-for="asset in catalog?.backgrounds" :key="asset.relative_path" class="background-row">
              <span>{{ asset.file_name }}</span>
              <small>{{ formatBytes(asset.file_size) }} · {{ asset.linked_scene_id || 'unlinked' }}</small>
            </div>
          </div>
        </section>
      </aside>
    </main>

    <Transition name="fade">
      <div v-if="toast" class="toast" @click="toast = ''">{{ toast }}</div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { invokeCommand } from '../lib/tauri'
import { useI18n } from '../lib/i18n'

const { t } = useI18n()

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

const catalog = ref<SceneAssetCatalog | null>(null)
const activeState = ref<ActiveScene | null>(null)
const isLoading = ref(false)
const settingSceneId = ref('')
const toast = ref('')
const activeSceneStorageKey = 'monogatari.activeScene'

const previewCatalog: SceneAssetCatalog = {
  project_path: 'Browser preview',
  valid: true,
  error_count: 0,
  warning_count: 0,
  scenes: [
    {
      id: 'sakura_park',
      name: 'Sakura Park',
      background_path: 'assets/backgrounds/sakura_park.png',
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
      background_path: 'assets/backgrounds/studio_night.webp',
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
      file_name: 'sakura_park.png',
      relative_path: 'assets/backgrounds/sakura_park.png',
      absolute_path: '',
      extension: 'png',
      file_size: 1260000,
      linked_scene_id: 'sakura_park',
    },
    {
      id: 'studio_night',
      file_name: 'studio_night.webp',
      relative_path: 'assets/backgrounds/studio_night.webp',
      absolute_path: '',
      extension: 'webp',
      file_size: 884000,
      linked_scene_id: 'studio_night',
    },
  ],
  issues: [],
}

const scenes = computed(() => catalog.value?.scenes || [])
const activeScene = computed(() => activeState.value?.scene || null)

async function refreshCatalog() {
  isLoading.value = true
  try {
    catalog.value = await invokeCommand<SceneAssetCatalog>('list_scene_assets', undefined, previewCatalog)
    activeState.value = await invokeCommand<ActiveScene>(
      'get_current_scene',
      undefined,
      () => previewActiveState(catalog.value || previewCatalog)
    )
  } catch (e) {
    toast.value = String(e)
    catalog.value = previewCatalog
    activeState.value = previewActiveState(previewCatalog)
  } finally {
    isLoading.value = false
  }
}

async function activateScene(scene: SceneInfo) {
  settingSceneId.value = scene.id
  try {
    const selected = await invokeCommand<SceneInfo>(
      'set_scene',
      { sceneId: scene.id },
      () => scene
    )
    activeState.value = {
      scene: selected,
      scene_history: [...(activeState.value?.scene_history || []), selected.id].slice(-24),
    }
    localStorage.setItem(activeSceneStorageKey, JSON.stringify(selected))
    toast.value = `Active scene: ${selected.name}`
  } catch (e) {
    toast.value = String(e)
  } finally {
    settingSceneId.value = ''
  }
}

function previewActiveState(sourceCatalog: SceneAssetCatalog): ActiveScene {
  const stored = localStorage.getItem(activeSceneStorageKey)
  if (stored) {
    try {
      const scene = JSON.parse(stored) as SceneInfo
      return { scene, scene_history: [scene.id] }
    } catch {
      localStorage.removeItem(activeSceneStorageKey)
    }
  }
  const scene = sourceCatalog.scenes[0] || null
  return { scene, scene_history: scene ? [scene.id] : [] }
}

function scenePreviewStyle(scene: SceneInfo) {
  const seed = Array.from(scene.id).reduce((sum, char) => sum + char.charCodeAt(0), 0)
  const hueA = (seed * 17) % 360
  const hueB = (hueA + 48) % 360
  const hueC = (hueA + 172) % 360
  return {
    background:
      `linear-gradient(145deg, hsl(${hueA} 54% 22%), hsl(${hueB} 46% 16%)), ` +
      `radial-gradient(circle at 72% 24%, hsl(${hueC} 62% 42% / 0.56), transparent 34%)`,
  }
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

onMounted(refreshCatalog)
</script>

<style scoped>
.scene-page {
  max-width: 1280px;
  margin: 0 auto;
  padding: 34px 40px;
}

.page-header {
  display: flex;
  justify-content: space-between;
  gap: 18px;
  align-items: flex-start;
  margin-bottom: 22px;
}

.page-header h1 {
  color: var(--text-primary);
  font-size: 28px;
  line-height: 1.15;
}

.page-header p {
  overflow: hidden;
  max-width: 760px;
  color: var(--text-tertiary);
  font-size: 13px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.eyebrow {
  display: block;
  color: var(--text-tertiary);
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0;
  text-transform: uppercase;
}

.header-actions {
  flex-shrink: 0;
}

.status-strip {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12px;
  margin-bottom: 18px;
}

.status-cell,
.panel,
.scene-card,
.empty-state {
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-1);
}

.status-cell {
  display: grid;
  gap: 4px;
  min-height: 78px;
  padding: 15px;
}

.status-cell span {
  color: var(--text-tertiary);
  font-size: 11px;
  font-weight: 800;
  text-transform: uppercase;
}

.status-cell strong {
  color: var(--brand-light);
  font-size: 25px;
  line-height: 1;
}

.status-cell.warning strong {
  color: var(--warning);
}

.status-cell.danger strong {
  color: var(--danger);
}

.asset-layout {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 340px;
  gap: 18px;
  align-items: start;
}

.scene-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: 14px;
}

.scene-card {
  overflow: hidden;
  transition: border-color var(--transition-fast), transform var(--transition-fast), box-shadow var(--transition-fast);
}

.scene-card:hover {
  border-color: var(--border-light);
  transform: translateY(-1px);
}

.scene-card.active {
  border-color: var(--brand);
  box-shadow: var(--shadow-brand);
}

.scene-card.broken {
  border-color: rgba(239,68,68,0.38);
}

.scene-preview {
  position: relative;
  min-height: 132px;
  overflow: hidden;
}

.scene-preview::after {
  content: '';
  position: absolute;
  inset: auto 0 0;
  height: 42%;
  background: linear-gradient(180deg, transparent, rgba(0,0,0,0.46));
}

.scene-preview span {
  position: absolute;
  z-index: 1;
  left: 12px;
  bottom: 10px;
  color: rgba(255,255,255,0.82);
  font-size: 11px;
  font-weight: 900;
  text-transform: uppercase;
}

.scene-card-body {
  display: grid;
  gap: 11px;
  padding: 14px;
}

.scene-title {
  display: flex;
  justify-content: space-between;
  gap: 10px;
  align-items: flex-start;
}

.scene-title div {
  min-width: 0;
  display: grid;
  gap: 2px;
}

.scene-title strong {
  overflow: hidden;
  color: var(--text-primary);
  font-size: 15px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.scene-title span,
.asset-path,
.muted,
.background-row small {
  color: var(--text-tertiary);
  font-size: 12px;
}

.scene-title b {
  flex-shrink: 0;
  color: var(--brand-light);
  font-size: 11px;
  text-transform: uppercase;
}

.scene-meta,
.history-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.scene-meta span,
.history-list span {
  border-radius: 4px;
  background: var(--surface-3);
  color: var(--text-secondary);
  font-size: 11px;
  font-weight: 700;
  padding: 2px 7px;
}

.asset-path {
  overflow: hidden;
  min-height: 18px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.scene-actions {
  display: flex;
  gap: 10px;
  align-items: center;
}

.inline-error,
.bad {
  color: var(--danger);
}

.ok {
  color: var(--success);
}

.inline-error {
  font-size: 12px;
  font-weight: 800;
}

.inspector {
  display: grid;
  gap: 14px;
  position: sticky;
  top: 18px;
}

.panel {
  display: grid;
  gap: 12px;
  padding: 15px;
}

.panel-head {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: center;
}

.panel-head strong {
  color: var(--text-primary);
}

.issue-list,
.background-list {
  display: grid;
  gap: 8px;
}

.issue-item,
.background-row {
  display: grid;
  gap: 3px;
  padding: 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
}

.issue-item span {
  color: var(--text-tertiary);
  font-size: 10px;
  font-weight: 900;
  text-transform: uppercase;
}

.issue-item.error span {
  color: var(--danger);
}

.issue-item.warning span {
  color: var(--warning);
}

.issue-item strong,
.background-row span {
  overflow: hidden;
  color: var(--text-primary);
  font-size: 12px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.issue-item p {
  color: var(--text-secondary);
  font-size: 12px;
  line-height: 1.35;
}

.empty-state {
  grid-column: 1 / -1;
  display: grid;
  place-items: center;
  gap: 9px;
  min-height: 320px;
  color: var(--text-tertiary);
  text-align: center;
}

.empty-state strong {
  color: var(--text-primary);
}

.empty-mark {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 46px;
  height: 46px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-2);
  color: var(--brand-light);
  font-family: var(--font-mono);
  font-weight: 900;
}

.toast {
  position: fixed;
  z-index: 80;
  left: 50%;
  bottom: 18px;
  transform: translateX(-50%);
  min-width: min(420px, calc(100vw - 32px));
  border: 1px solid rgba(45,212,191,0.36);
  border-radius: var(--radius);
  background: rgba(15,118,110,0.96);
  box-shadow: var(--shadow-lg);
  color: white;
  padding: 12px 14px;
  text-align: center;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

@media (max-width: 1060px) {
  .asset-layout {
    grid-template-columns: 1fr;
  }

  .inspector {
    position: static;
  }
}

@media (max-width: 720px) {
  .scene-page {
    padding: 22px 16px;
  }

  .page-header {
    flex-direction: column;
  }

  .status-strip {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>
