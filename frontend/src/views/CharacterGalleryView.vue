<template>
  <div class="gallery-workbench">
    <header class="gallery-header">
      <div>
        <span class="eyebrow">Cast</span>
        <h1>Character Gallery</h1>
        <p>{{ characters.length }} characters loaded from project data.</p>
      </div>
      <div class="gallery-actions">
        <div class="search-bar">
          <input class="input" v-model="searchQuery" :placeholder="t('characters.search', 'Search characters...')" />
        </div>
        <button class="btn btn-secondary btn-sm" @click="loadChars">Refresh</button>
        <button class="btn btn-primary btn-sm" @click="$router.push('/character-editor')">+ New</button>
      </div>
    </header>

    <div v-if="loading" class="loading-state">
      <div class="spinner"></div>
      <span>Loading characters...</span>
    </div>

    <div v-else-if="filteredCharacters.length === 0" class="empty-state">
      <span class="empty-mark">CG</span>
      <h2>No characters found</h2>
      <p>Add character JSON files to the characters directory, or create new ones in the Character Editor.</p>
      <button class="btn btn-primary" @click="$router.push('/character-editor')">Open Editor</button>
    </div>

    <div v-else class="gallery-layout">
      <div class="char-grid">
        <div
          v-for="c in filteredCharacters"
          :key="c.id"
          class="char-card"
          :class="{ selected: selected?.id === c.id }"
          @click="selectChar(c)"
        >
          <div class="card-top">
            <div class="char-avatar" :style="{ background: avatarColor(c.id) }">{{ c.name.charAt(0) }}</div>
            <div class="card-info">
              <h3>{{ c.name }}</h3>
              <p class="char-desc">{{ c.description || 'No description' }}</p>
            </div>
          </div>
          <div class="card-bottom">
            <div class="char-tags">
              <span v-if="c.personality?.speech_style" class="tag">{{ c.personality.speech_style }}</span>
              <span class="tag emotion-tag">{{ c.emotion || 'neutral' }}</span>
            </div>
            <div class="card-actions">
              <button class="btn-icon" title="Chat" @click.stop="startChat(c)">C</button>
              <button class="btn-icon" title="Edit" @click.stop="editChar(c)">E</button>
            </div>
          </div>
        </div>
      </div>

      <aside v-if="selected" class="detail-panel">
        <div class="detail-header">
          <div class="char-avatar lg" :style="{ background: avatarColor(selected.id) }">{{ selected.name.charAt(0) }}</div>
          <div>
            <h2>{{ selected.name }}</h2>
            <p class="muted">{{ selected.id }}</p>
          </div>
        </div>

        <div class="detail-tabs">
          <button
            v-for="tab in detailTabs"
            :key="tab.key"
            class="tab-btn"
            :class="{ active: activeTab === tab.key }"
            @click="activeTab = tab.key"
          >{{ tab.label }}</button>
        </div>

        <div class="detail-content">
          <!-- Profile Tab -->
          <div v-if="activeTab === 'profile'">
            <div class="field"><label>Description</label><p>{{ selected.description || 'None' }}</p></div>
            <div class="field"><label>Background</label><p>{{ selected.background || 'None' }}</p></div>
            <div class="field"><label>Speech Style</label><p>{{ selected.personality?.speech_style || 'Default' }}</p></div>
            <div class="field"><label>Current Emotion</label><p>{{ selected.emotion || 'neutral' }}</p></div>
          </div>

          <!-- Personality Tab -->
          <div v-if="activeTab === 'personality'">
            <div class="radar-section">
              <svg viewBox="0 0 260 260" class="radar-svg">
                <polygon v-for="ring in 5" :key="ring"
                  :points="radarRing(ring * 20)"
                  fill="none" stroke="var(--border)" stroke-width="1"
                />
                <polygon :points="radarPolygon" fill="rgba(45,212,191,0.18)" stroke="var(--brand)" stroke-width="2" />
                <circle v-for="(pt, i) in radarPoints" :key="i" :cx="pt.x" :cy="pt.y" r="4" fill="var(--brand)" />
                <text v-for="(label, i) in radarLabels" :key="label"
                  :x="radarLabelPos(i).x" :y="radarLabelPos(i).y"
                  text-anchor="middle" fill="var(--text-secondary)" font-size="10" font-weight="700"
                >{{ label }}</text>
              </svg>
            </div>
            <div class="trait-list">
              <div v-for="t in personalityTraits" :key="t.key" class="trait-row">
                <span class="trait-label">{{ t.label }}</span>
                <div class="trait-bar"><div class="trait-fill" :style="{ width: ((selected.personality?.[t.key] || 0) * 100) + '%' }"></div></div>
                <span class="trait-val">{{ (selected.personality?.[t.key] || 0).toFixed(2) }}</span>
              </div>
            </div>
          </div>
        </div>

        <div class="detail-actions">
          <button class="btn btn-primary" @click="startChat(selected)">Start Chat</button>
          <button class="btn btn-secondary" @click="editChar(selected)">{{ t('characters.edit', 'Edit') }}</button>
        </div>
      </aside>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { invokeCommand } from '../lib/tauri'
import { useI18n } from '../lib/i18n'

const { t } = useI18n()

interface Character {
  id: string
  name: string
  description: string
  background: string
  emotion: string
  personality: Record<string, any>
  live2d_model_path: string | null
}

const router = useRouter()
const characters = ref<Character[]>([])
const selected = ref<Character | null>(null)
const loading = ref(true)
const searchQuery = ref('')
const activeTab = ref('profile')

const detailTabs = [
  { key: 'profile', label: 'Profile' },
  { key: 'personality', label: 'Personality' },
]

const personalityTraits = [
  { key: 'openness', label: 'Openness' },
  { key: 'conscientiousness', label: 'Conscientiousness' },
  { key: 'extraversion', label: 'Extraversion' },
  { key: 'agreeableness', label: 'Agreeableness' },
  { key: 'neuroticism', label: 'Neuroticism' },
]

const radarLabels = ['O', 'A', 'E', 'N', 'C']

const filteredCharacters = computed(() => {
  if (!searchQuery.value.trim()) return characters.value
  const q = searchQuery.value.toLowerCase()
  return characters.value.filter(c =>
    c.name.toLowerCase().includes(q) ||
    c.description?.toLowerCase().includes(q) ||
    c.personality?.speech_style?.toLowerCase().includes(q)
  )
})

const radarTraits = computed(() => {
  if (!selected.value) return [0, 0, 0, 0, 0]
  const p = selected.value.personality || {}
  return [p.openness || 0, p.agreeableness || 0, p.extraversion || 0, p.neuroticism || 0, p.conscientiousness || 0]
})

const radarPoints = computed(() => {
  const cx = 130, cy = 130, maxR = 100
  return radarTraits.value.map((val, i) => {
    const angle = (Math.PI * 2 * i) / 5 - Math.PI / 2
    return { x: cx + Math.cos(angle) * maxR * val, y: cy + Math.sin(angle) * maxR * val }
  })
})

const radarPolygon = computed(() => radarPoints.value.map(p => p.x + ',' + p.y).join(' '))

function radarRing(radius: number): string {
  const cx = 130, cy = 130
  return Array.from({ length: 5 }, (_, i) => {
    const angle = (Math.PI * 2 * i) / 5 - Math.PI / 2
    return (cx + Math.cos(angle) * radius) + ',' + (cy + Math.sin(angle) * radius)
  }).join(' ')
}

function radarLabelPos(i: number) {
  const cx = 130, cy = 130, r = 118
  const angle = (Math.PI * 2 * i) / 5 - Math.PI / 2
  return { x: cx + Math.cos(angle) * r, y: cy + Math.sin(angle) * r + 4 }
}

const colors = ['#2dd4bf', '#a78bfa', '#f472b6', '#fb923c', '#4ade80', '#60a5fa', '#fbbf24', '#f87171']
function avatarColor(id: string): string {
  let hash = 0
  for (let i = 0; i < id.length; i++) hash = ((hash << 5) - hash + id.charCodeAt(i)) | 0
  return colors[Math.abs(hash) % colors.length]
}

function selectChar(c: Character) {
  selected.value = c
  activeTab.value = 'profile'
}

function startChat(c: Character) {
  router.push('/chat')
}

function editChar(c: Character) {
  router.push('/character-editor')
}

async function loadChars() {
  loading.value = true
  try {
    const result = await invokeCommand<Character[]>('get_characters', undefined, [])
    characters.value = result
  } catch { characters.value = [] }
  loading.value = false
}

onMounted(loadChars)
</script>

<style scoped>
.gallery-workbench {
  max-width: 1400px;
  margin: 0 auto;
  padding: 34px 40px;
}

.gallery-header {
  display: flex;
  justify-content: space-between;
  gap: 18px;
  align-items: flex-start;
  margin-bottom: 24px;
}

.gallery-header h1 {
  color: var(--text-primary);
  font-size: 28px;
  line-height: 1.15;
  margin-top: 3px;
}

.gallery-header p {
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

.gallery-actions {
  display: flex;
  gap: 8px;
  align-items: center;
  flex-shrink: 0;
}

.search-bar .input {
  min-width: 200px;
}

.loading-state, .empty-state {
  display: grid;
  place-items: center;
  gap: 12px;
  padding: 80px 20px;
  text-align: center;
  color: var(--text-secondary);
}

.empty-state h2 {
  color: var(--text-primary);
  font-size: 22px;
  margin-top: 12px;
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

.gallery-layout {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 380px;
  gap: 18px;
  align-items: start;
}

.char-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 14px;
}

.char-card {
  padding: 18px;
  background: var(--surface-1);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.char-card:hover, .char-card.selected {
  border-color: var(--brand);
  background: var(--surface-2);
}

.card-top {
  display: flex;
  gap: 14px;
  margin-bottom: 12px;
}

.char-avatar {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 20px;
  font-weight: 700;
  color: white;
  flex-shrink: 0;
}

.char-avatar.lg {
  width: 56px;
  height: 56px;
  font-size: 24px;
}

.card-info {
  flex: 1;
  min-width: 0;
}

.card-info h3 {
  color: var(--text-primary);
  font-size: 15px;
  font-weight: 700;
  margin-bottom: 4px;
}

.char-desc {
  font-size: 12px;
  color: var(--text-tertiary);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  line-height: 1.4;
}

.card-bottom {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.char-tags {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.tag {
  padding: 2px 10px;
  border: 1px solid var(--border);
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  color: var(--text-secondary);
}

.emotion-tag {
  border-color: var(--brand);
  color: var(--brand-light);
}

.card-actions {
  display: flex;
  gap: 4px;
}

.btn-icon {
  width: 30px;
  height: 30px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
  color: var(--text-secondary);
  cursor: pointer;
  font-weight: 800;
  font-size: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.btn-icon:hover {
  border-color: var(--brand);
  color: var(--brand-light);
}

.detail-panel {
  position: sticky;
  top: 18px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-1);
  overflow: hidden;
}

.detail-header {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 20px;
  border-bottom: 1px solid var(--border);
}

.detail-header h2 {
  color: var(--text-primary);
  font-size: 18px;
  font-weight: 750;
}

.muted {
  color: var(--text-tertiary);
  font-size: 12px;
}

.detail-tabs {
  display: flex;
  border-bottom: 1px solid var(--border);
  padding: 0 16px;
}

.tab-btn {
  padding: 10px 14px;
  border: none;
  border-bottom: 2px solid transparent;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  font-weight: 700;
  font-size: 12px;
}

.tab-btn:hover { color: var(--text-primary); }
.tab-btn.active { color: var(--brand-light); border-bottom-color: var(--brand); }

.detail-content {
  padding: 18px;
}

.field {
  margin-bottom: 14px;
}

.field label {
  display: block;
  font-size: 11px;
  font-weight: 700;
  color: var(--text-tertiary);
  text-transform: uppercase;
  margin-bottom: 4px;
}

.field p {
  font-size: 13px;
  line-height: 1.6;
  color: var(--text-secondary);
}

.radar-section {
  display: flex;
  justify-content: center;
  margin-bottom: 16px;
}

.radar-svg {
  width: 220px;
  height: 220px;
}

.trait-list {
  display: grid;
  gap: 10px;
}

.trait-row {
  display: grid;
  grid-template-columns: 120px 1fr 40px;
  gap: 10px;
  align-items: center;
}

.trait-label {
  font-size: 12px;
  color: var(--text-secondary);
}

.trait-bar {
  height: 6px;
  background: var(--surface-3);
  border-radius: 3px;
  overflow: hidden;
}

.trait-fill {
  height: 100%;
  background: var(--brand);
  border-radius: 3px;
  transition: width 0.3s ease;
}

.trait-val {
  font-size: 12px;
  font-family: var(--font-mono);
  color: var(--brand-light);
  text-align: right;
  font-weight: 700;
}

.detail-actions {
  display: flex;
  gap: 8px;
  padding: 16px 18px;
  border-top: 1px solid var(--border);
}

.spinner {
  width: 20px;
  height: 20px;
  border: 2px solid var(--border);
  border-top-color: var(--brand);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin { to { transform: rotate(360deg); } }

@media (max-width: 1060px) {
  .gallery-layout {
    grid-template-columns: 1fr;
  }
  .detail-panel {
    position: static;
  }
}

@media (max-width: 640px) {
  .gallery-workbench { padding: 22px; }
  .gallery-header { flex-direction: column; }
  .char-grid { grid-template-columns: 1fr; }
}
</style>
