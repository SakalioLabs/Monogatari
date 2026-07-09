<template>
  <div class="scene-editor">
    <header class="editor-header">
      <div>
        <span class="eyebrow">Authoring</span>
        <h1>{{ t("scene.title", "Scene Editor") }}</h1>
        <p>{{ scenes.length }} scenes in project</p>
      </div>
      <div class="header-actions">
        <div class="search-bar">
          <input class="input" v-model="searchQuery" placeholder="Search scenes..." />
        </div>
        <select class="input input-sm" v-model="viewMode">
          <option value="grid">Grid</option>
          <option value="list">List</option>
        </select>
        <button class="btn btn-primary btn-sm" @click="createScene">+ New Scene</button>
      </div>
    </header>

    <main class="editor-layout">
      <section class="scene-grid" :class="viewMode">
        <div
          v-for="scene in filteredScenes"
          :key="scene.id"
          class="scene-card"
          :class="{ selected: selectedScene?.id === scene.id }"
          @click="selectScene(scene)"
        >
          <div class="scene-preview" :style="{ background: sceneBackground(scene) }">
            <div class="scene-overlay">
              <span class="scene-badge">{{ scene.time_of_day || 'any' }}</span>
              <span v-if="scene.weather" class="scene-badge weather">{{ scene.weather }}</span>
            </div>
          </div>
          <div class="scene-info">
            <strong>{{ scene.name }}</strong>
            <small>{{ scene.id }}</small>
            <div class="scene-tags">
              <span v-for="tag in (scene.tags || []).slice(0, 3)" :key="tag" class="tag">{{ tag }}</span>
            </div>
          </div>
        </div>
        <div v-if="filteredScenes.length === 0" class="empty-state">
          <span class="empty-mark">SE</span>
          <h2>No scenes found</h2>
          <p>Create a new scene or adjust your search filter.</p>
        </div>
      </section>

      <aside v-if="selectedScene" class="scene-detail">
        <div class="detail-header">
          <span class="eyebrow">Scene Properties</span>
          <button class="btn-icon" @click="selectedScene = null" title="Close">x</button>
        </div>

        <div class="detail-preview" :style="{ background: sceneBackground(selectedScene) }">
          <div class="preview-label">{{ selectedScene.name }}</div>
        </div>

        <div class="detail-form">
          <label class="form-field">
            <span>Scene ID</span>
            <input class="input" v-model="selectedScene.id" disabled />
          </label>
          <label class="form-field">
            <span>Name</span>
            <input class="input" v-model="selectedScene.name" />
          </label>
          <label class="form-field">
            <span>Background Path</span>
            <input class="input" v-model="selectedScene.background_path" placeholder="assets/backgrounds/scene.svg" />
          </label>
          <label class="form-field">
            <span>BGM Path</span>
            <input class="input" v-model="selectedScene.bgm_path" placeholder="assets/bgm/theme.mp3" />
          </label>
          <div class="form-row">
            <label class="form-field">
              <span>Weather</span>
              <select class="input" v-model="selectedScene.weather">
                <option value="">None</option>
                <option value="clear">Clear</option>
                <option value="spring">Spring</option>
                <option value="rain">Rain</option>
                <option value="snow">Snow</option>
                <option value="fog">Fog</option>
                <option value="storm">Storm</option>
              </select>
            </label>
            <label class="form-field">
              <span>Time of Day</span>
              <select class="input" v-model="selectedScene.time_of_day">
                <option value="">Any</option>
                <option value="dawn">Dawn</option>
                <option value="day">Day</option>
                <option value="sunset">Sunset</option>
                <option value="night">Night</option>
                <option value="midnight">Midnight</option>
              </select>
            </label>
          </div>
          <label class="form-field">
            <span>Tags (comma-separated)</span>
            <input class="input" v-model="tagsInput" placeholder="outdoor, calm, forest" />
          </label>
        </div>

        <div class="detail-actions">
          <button class="btn btn-primary" @click="saveScene">Save Changes</button>
          <button class="btn btn-danger btn-sm" @click="deleteScene">Delete</button>
        </div>
      </aside>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { invokeCommand } from '../lib/tauri'
import { useI18n } from '../lib/i18n'

const { t } = useI18n()

interface Scene {
  id: string
  name: string
  background_path: string | null
  bgm_path: string | null
  weather: string | null
  time_of_day: string | null
  tags: string[]
  source?: string
  background_exists?: boolean
  absolute_background_path?: string | null
}

const scenes = ref<Scene[]>([])
const selectedScene = ref<Scene | null>(null)
const searchQuery = ref('')
const viewMode = ref<'grid' | 'list'>('grid')
const tagsInput = ref('')

const filteredScenes = computed(() => {
  let result = scenes.value
  if (searchQuery.value.trim()) {
    const q = searchQuery.value.toLowerCase()
    result = result.filter(s =>
      s.name.toLowerCase().includes(q) ||
      s.id.toLowerCase().includes(q) ||
      (s.tags || []).some(t => t.toLowerCase().includes(q))
    )
  }
  return result
})

watch(selectedScene, (scene) => {
  tagsInput.value = scene ? (scene.tags || []).join(', ') : ''
})

function sceneBackground(scene: Scene): string {
  const hue = Array.from(scene.id).reduce((s, c) => s + c.charCodeAt(0), 0) * 37 % 360
  return 'linear-gradient(180deg, hsl(' + hue + ' 40% 22%), hsl(' + ((hue + 40) % 360) + ' 35% 12%))'
}

function selectScene(scene: Scene) {
  selectedScene.value = scene
}

function createScene() {
  const id = 'scene_' + Date.now()
  const scene: Scene = {
    id,
    name: 'New Scene',
    background_path: null,
    bgm_path: null,
    weather: null,
    time_of_day: 'day',
    tags: [],
  }
  scenes.value.push(scene)
  selectedScene.value = scene
}

function saveScene() {
  if (!selectedScene.value) return
  selectedScene.value.tags = tagsInput.value.split(',').map(s => s.trim()).filter(Boolean)
}

function deleteScene() {
  if (!selectedScene.value) return
  scenes.value = scenes.value.filter(s => s.id !== selectedScene.value!.id)
  selectedScene.value = null
}

async function loadScenes() {
  try {
    const result = await invokeCommand<Scene[]>('list_scene_assets', undefined, [])
    scenes.value = result
  } catch (e) {
    console.error('Failed to load scenes:', e)
    scenes.value = []
  }
}

onMounted(loadScenes)
</script>

<style scoped>
.scene-editor {
  max-width: 1400px;
  margin: 0 auto;
  padding: 34px 40px;
}

.editor-header {
  display: flex;
  justify-content: space-between;
  gap: 18px;
  align-items: flex-start;
  margin-bottom: 24px;
}

.editor-header h1 {
  color: var(--text-primary);
  font-size: 28px;
  line-height: 1.15;
  margin-top: 3px;
}

.editor-header p {
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
  gap: 8px;
  align-items: center;
  flex-shrink: 0;
}

.search-bar .input {
  min-width: 200px;
}

.input-sm {
  min-width: 80px;
}

.editor-layout {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 360px;
  gap: 18px;
  align-items: start;
}

.scene-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: 14px;
}

.scene-grid.list {
  grid-template-columns: 1fr;
}

.scene-card {
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-1);
  cursor: pointer;
  overflow: hidden;
  transition: all var(--transition-fast);
}

.scene-card:hover, .scene-card.selected {
  border-color: var(--brand);
  transform: translateY(-1px);
  box-shadow: var(--shadow);
}

.scene-preview {
  height: 120px;
  position: relative;
  border-radius: var(--radius) var(--radius) 0 0;
}

.scene-overlay {
  position: absolute;
  top: 8px;
  right: 8px;
  display: flex;
  gap: 6px;
}

.scene-badge {
  padding: 2px 8px;
  border-radius: 999px;
  background: rgba(0,0,0,0.5);
  color: white;
  font-size: 10px;
  font-weight: 700;
  backdrop-filter: blur(4px);
}

.scene-badge.weather {
  background: rgba(96,165,250,0.4);
}

.scene-info {
  padding: 12px;
}

.scene-info strong {
  display: block;
  color: var(--text-primary);
  font-size: 13px;
  margin-bottom: 2px;
}

.scene-info small {
  color: var(--text-tertiary);
  font-size: 11px;
  font-family: var(--font-mono);
}

.scene-tags {
  display: flex;
  gap: 4px;
  margin-top: 8px;
  flex-wrap: wrap;
}

.tag {
  padding: 2px 8px;
  border: 1px solid var(--border);
  border-radius: 999px;
  font-size: 10px;
  color: var(--text-secondary);
}

.scene-detail {
  position: sticky;
  top: 18px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-1);
  overflow: hidden;
}

.detail-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 14px 16px;
  border-bottom: 1px solid var(--border);
}

.detail-preview {
  height: 140px;
  display: flex;
  align-items: flex-end;
  padding: 12px;
  position: relative;
}

.preview-label {
  padding: 4px 12px;
  background: rgba(0,0,0,0.5);
  border-radius: var(--radius-sm);
  color: white;
  font-weight: 700;
  font-size: 14px;
  backdrop-filter: blur(4px);
}

.detail-form {
  padding: 16px;
  display: grid;
  gap: 12px;
}

.form-field {
  display: grid;
  gap: 5px;
}

.form-field span {
  color: var(--text-secondary);
  font-size: 12px;
  font-weight: 700;
}

.form-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.detail-actions {
  display: flex;
  gap: 8px;
  padding: 14px 16px;
  border-top: 1px solid var(--border);
}

.btn-icon {
  width: 28px;
  height: 28px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
  color: var(--text-secondary);
  cursor: pointer;
  font-weight: 800;
  font-size: 16px;
}

.btn-icon:hover {
  border-color: var(--brand);
  color: var(--brand-light);
}

.empty-state {
  grid-column: 1 / -1;
  text-align: center;
  padding: 60px 20px;
}

.empty-state h2 {
  color: var(--text-primary);
  font-size: 22px;
  margin-top: 12px;
}

.empty-state p {
  color: var(--text-tertiary);
  font-size: 13px;
  margin-top: 6px;
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

@media (max-width: 1060px) {
  .editor-layout {
    grid-template-columns: 1fr;
  }
  .scene-detail {
    position: static;
  }
}

@media (max-width: 640px) {
  .scene-editor { padding: 22px; }
  .editor-header { flex-direction: column; }
  .scene-grid { grid-template-columns: 1fr; }
}
</style>
