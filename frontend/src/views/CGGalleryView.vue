<template>
  <div class="cg-gallery">
    <header class="cg-header">
      <div>
        <span class="eyebrow">{{ t('cg.title', 'CG Gallery') }}</span>
        <h1>Scene Collection</h1>
      </div>
      <div class="cg-stats">
        <span class="stat-pill">{{ unlockedCount }} / {{ scenes.length }} {{ t('cg.unlocked', 'Unlocked') }}</span>
      </div>
    </header>

    <div class="cg-tabs">
      <button class="tab-btn" :class="{ active: activeTab === 'scenes' }" @click="activeTab = 'scenes'">
        {{ t('cg.scenes', 'Scenes') }}
      </button>
      <button class="tab-btn" :class="{ active: activeTab === 'characters' }" @click="activeTab = 'characters'">
        {{ t('cg.characters', 'Characters') }}
      </button>
    </div>

    <main v-if="activeTab === 'scenes'" class="cg-grid">
      <div
        v-for="scene in scenes"
        :key="scene.id"
        class="cg-card"
        :class="{ locked: !scene.unlocked }"
        @click="openPreview(scene)"
      >
        <div class="cg-thumb" :style="thumbStyle(scene)">
          <div v-if="!scene.unlocked" class="lock-overlay">
            <span class="lock-icon">&#128274;</span>
            <span class="lock-text">{{ t('cg.locked', 'Locked') }}</span>
          </div>
          <div v-else class="cg-badge">&#10003;</div>
        </div>
        <div class="cg-info">
          <strong>{{ scene.unlocked ? scene.name : '???' }}</strong>
          <span class="cg-meta">{{ scene.weather || 'Unknown' }} / {{ scene.time_of_day || 'Any' }}</span>
        </div>
      </div>

      <div v-if="scenes.length === 0" class="empty-state">
        <span class="empty-icon">&#127912;</span>
        <p>{{ t('cg.empty', 'No CGs unlocked yet. Keep playing to discover scenes!') }}</p>
      </div>
    </main>

    <main v-else class="cg-grid">
      <div
        v-for="char in characters"
        :key="char.id"
        class="cg-card"
        :class="{ locked: !char.unlocked }"
        @click="openCharPreview(char)"
      >
        <div class="cg-thumb char-thumb" :style="charThumbStyle(char)">
          <div v-if="!char.unlocked" class="lock-overlay">
            <span class="lock-icon">&#128274;</span>
            <span class="lock-text">{{ t('cg.locked', 'Locked') }}</span>
          </div>
          <div v-else class="cg-initials">{{ initials(char.name) }}</div>
        </div>
        <div class="cg-info">
          <strong>{{ char.unlocked ? char.name : '???' }}</strong>
          <span class="cg-meta">{{ char.description || 'Character' }}</span>
        </div>
      </div>
    </main>

    <Transition name="fade">
      <div v-if="previewScene" class="preview-overlay" @click.self="previewScene = null">
        <div class="preview-panel">
          <div class="preview-bg" :style="previewBgStyle"></div>
          <div class="preview-info">
            <h2>{{ previewScene.name }}</h2>
            <div class="preview-meta">
              <span v-if="previewScene.weather">{{ previewScene.weather }}</span>
              <span v-if="previewScene.time_of_day">{{ previewScene.time_of_day }}</span>
              <span v-for="tag in (previewScene.tags || [])" :key="tag" class="tag-pill">{{ tag }}</span>
            </div>
            <p v-if="previewScene.background_path" class="preview-path">{{ previewScene.background_path }}</p>
          </div>
          <button class="close-btn" @click="previewScene = null">{{ t('common.close', 'Close') }}</button>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from '../lib/i18n'
import { invokeCommand } from '../lib/tauri'

const { t } = useI18n()

interface SceneData {
  id: string
  name: string
  background_path: string | null
  weather: string | null
  time_of_day: string | null
  tags: string[]
  unlocked?: boolean
}

interface CharacterData {
  id: string
  name: string
  description: string
  emotion: string
  unlocked?: boolean
}

const activeTab = ref<'scenes' | 'characters'>('scenes')
const scenes = ref<SceneData[]>([])
const characters = ref<CharacterData[]>([])
const previewScene = ref<SceneData | null>(null)

const unlockedCount = computed(() => scenes.value.filter(s => s.unlocked).length)

const charColors = [
  '#6366f1', '#ec4899', '#14b8a6', '#f59e0b', '#8b5cf6',
  '#ef4444', '#06b6d4', '#84cc16', '#f97316', '#a855f7'
]

function colorForId(id: string): string {
  let hash = 0
  for (const ch of id) hash = ((hash << 5) - hash) + ch.charCodeAt(0)
  return charColors[Math.abs(hash) % charColors.length]
}

function initials(name: string): string {
  return name.trim().slice(0, 2).toUpperCase() || '??'
}

function thumbStyle(scene: SceneData) {
  const c = colorForId(scene.id)
  return {
    background: scene.unlocked
      ? `linear-gradient(135deg, ${c}33, ${c}11)`
      : 'var(--surface-3)',
  }
}

function charThumbStyle(char: CharacterData) {
  const c = colorForId(char.id)
  return {
    background: char.unlocked
      ? `linear-gradient(135deg, ${c}44, ${c}22)`
      : 'var(--surface-3)',
  }
}

const previewBgStyle = computed(() => {
  if (!previewScene.value) return {}
  const c = colorForId(previewScene.value.id)
  return {
    background: `linear-gradient(180deg, ${c}22, transparent 60%), radial-gradient(circle at 50% 60%, ${c}18, transparent 50%)`,
  }
})

function openPreview(scene: SceneData) {
  if (!scene.unlocked) return
  previewScene.value = scene
}

function openCharPreview(char: CharacterData) {
  if (!char.unlocked) return
}

async function loadData() {
  try {
    const sceneList = await invokeCommand<SceneData[]>('list_scene_assets', undefined, [])
    scenes.value = sceneList.map(s => ({ ...s, unlocked: true }))
  } catch {
    scenes.value = []
  }
  try {
    const charList = await invokeCommand<CharacterData[]>('get_characters', undefined, [])
    characters.value = charList.map(c => ({ ...c, unlocked: true }))
  } catch {
    characters.value = []
  }
}

onMounted(loadData)
</script>

<style scoped>
.cg-gallery {
  padding: 32px;
  max-width: 1200px;
  margin: 0 auto;
  min-height: 100vh;
}

.cg-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  margin-bottom: 24px;
}

.cg-header h1 {
  font-size: 28px;
  font-weight: 800;
  color: var(--text-primary);
  margin: 4px 0 0;
}

.eyebrow {
  display: block;
  color: var(--text-tertiary);
  font-size: 11px;
  font-weight: 800;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.stat-pill {
  padding: 6px 14px;
  border: 1px solid var(--border);
  border-radius: 100px;
  font-size: 13px;
  font-weight: 700;
  color: var(--brand-light);
  background: rgba(45,212,191,0.08);
}

.cg-tabs {
  display: flex;
  gap: 4px;
  margin-bottom: 24px;
  border-bottom: 1px solid var(--border);
  padding-bottom: 0;
}

.tab-btn {
  padding: 10px 20px;
  border: none;
  background: none;
  color: var(--text-secondary);
  cursor: pointer;
  font: inherit;
  font-weight: 600;
  font-size: 14px;
  border-bottom: 2px solid transparent;
  margin-bottom: -1px;
  transition: all 0.2s;
}

.tab-btn:hover { color: var(--text-primary); }
.tab-btn.active {
  color: var(--brand-light);
  border-bottom-color: var(--brand);
}

.cg-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: 16px;
}

.cg-card {
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-1);
  overflow: hidden;
  cursor: pointer;
  transition: all 0.2s;
}

.cg-card:hover:not(.locked) {
  border-color: var(--brand);
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0,0,0,0.3);
}

.cg-card.locked { opacity: 0.5; cursor: default; }

.cg-thumb {
  position: relative;
  height: 160px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.char-thumb { border-radius: 0; }

.cg-initials {
  width: 64px;
  height: 64px;
  border-radius: 50%;
  background: var(--brand);
  color: var(--surface-0);
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 800;
  font-size: 22px;
}

.lock-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
  background: rgba(0,0,0,0.4);
}

.lock-icon { font-size: 28px; }
.lock-text { font-size: 12px; font-weight: 700; color: var(--text-tertiary); }

.cg-badge {
  position: absolute;
  top: 8px;
  right: 8px;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  background: var(--brand);
  color: var(--surface-0);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  font-weight: 800;
}

.cg-info {
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.cg-info strong { color: var(--text-primary); font-size: 14px; }
.cg-meta { color: var(--text-tertiary); font-size: 12px; }

.empty-state {
  grid-column: 1 / -1;
  text-align: center;
  padding: 80px 20px;
  color: var(--text-tertiary);
}

.empty-icon { font-size: 48px; display: block; margin-bottom: 12px; }

.preview-overlay {
  position: fixed;
  inset: 0;
  z-index: 50;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0,0,0,0.75);
  backdrop-filter: blur(8px);
}

.preview-panel {
  position: relative;
  width: min(800px, calc(100vw - 48px));
  max-height: calc(100vh - 80px);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  background: var(--surface-1);
  overflow: hidden;
}

.preview-bg { height: 400px; }

.preview-info {
  padding: 24px;
}

.preview-info h2 {
  color: var(--text-primary);
  font-size: 24px;
  margin: 0 0 8px;
}

.preview-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 12px;
}

.preview-meta span {
  padding: 3px 10px;
  border: 1px solid var(--border);
  border-radius: 100px;
  font-size: 12px;
  color: var(--text-secondary);
}

.tag-pill { background: var(--surface-2); }

.preview-path {
  color: var(--text-tertiary);
  font-family: var(--font-mono);
  font-size: 12px;
}

.close-btn {
  position: absolute;
  top: 16px;
  right: 16px;
  padding: 8px 16px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: rgba(21,25,34,0.9);
  color: var(--text-secondary);
  cursor: pointer;
  font: inherit;
  font-weight: 700;
  font-size: 13px;
  backdrop-filter: blur(8px);
}

.close-btn:hover { border-color: var(--brand); color: var(--brand-light); }

.fade-enter-active, .fade-leave-active { transition: opacity 0.2s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }

@media (max-width: 640px) {
  .cg-gallery { padding: 16px; }
  .cg-grid { grid-template-columns: repeat(auto-fill, minmax(160px, 1fr)); }
  .cg-thumb { height: 120px; }
  .preview-bg { height: 250px; }
}
</style>
