<template>
  <div class="gallery-page">
    <header class="gallery-header">
      <h1>Character Gallery</h1>
      <div class="gallery-actions">
        <button class="btn btn-secondary btn-sm" @click="loadChars">Refresh</button>
      </div>
    </header>
    <div v-if="loading" class="loading-state">
      <div class="spinner"></div>
      <span>Loading characters...</span>
    </div>
    <div v-else-if="characters.length === 0" class="empty-state">
      <p>No characters found. Add character JSON files to the characters directory.</p>
    </div>
    <div v-else class="char-grid">
      <div v-for="c in characters" :key="c.id" class="char-card" @click="selected = c">
        <div class="char-avatar" :style="{ background: avatarColor(c.id) }">{{ c.name.charAt(0) }}</div>
        <div class="char-info">
          <h3>{{ c.name }}</h3>
          <p class="char-desc">{{ c.description || 'No description' }}</p>
          <div class="char-tags">
            <span v-if="c.personality?.speech_style" class="tag">{{ c.personality.speech_style }}</span>
            <span class="tag">Emotion: {{ c.emotion || 'neutral' }}</span>
          </div>
        </div>
      </div>
    </div>
    <div v-if="selected" class="detail-overlay" @click.self="selected = null">
      <div class="detail-panel">
        <div class="detail-header">
          <div class="char-avatar lg" :style="{ background: avatarColor(selected.id) }">{{ selected.name.charAt(0) }}</div>
          <div>
            <h2>{{ selected.name }}</h2>
            <p class="muted">{{ selected.id }}</p>
          </div>
          <button class="btn btn-ghost btn-sm" style="margin-left:auto" @click="selected = null">Close</button>
        </div>
        <div class="detail-body">
          <div class="field"><label>Description</label><p>{{ selected.description || 'None' }}</p></div>
          <div class="field"><label>Background</label><p>{{ selected.background || 'None' }}</p></div>
          <div class="field" v-if="selected.personality">
            <label>Personality</label>
            <div class="trait-grid">
              <div v-for="t in personalityTraits" :key="t.key" class="trait">
                <span class="trait-label">{{ t.label }}</span>
                <div class="trait-bar"><div class="trait-fill" :style="{ width: ((selected.personality[t.key] || 0) * 100) + '%' }"></div></div>
              </div>
            </div>
          </div>
          <div class="field"><label>Speech Style</label><p>{{ selected.personality?.speech_style || 'Default' }}</p></div>
          <div class="field"><label>Current Emotion</label><p>{{ selected.emotion || 'neutral' }}</p></div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invokeCommand } from '../lib/tauri'

interface Character {
  id: string
  name: string
  description: string
  background: string
  emotion: string
  personality: Record<string, any>
}

const characters = ref<Character[]>([])
const selected = ref<Character | null>(null)
const loading = ref(true)

const personalityTraits = [
  { key: 'openness', label: 'Openness' },
  { key: 'conscientiousness', label: 'Conscientiousness' },
  { key: 'extraversion', label: 'Extraversion' },
  { key: 'agreeableness', label: 'Agreeableness' },
  { key: 'neuroticism', label: 'Neuroticism' },
]

const colors = ['#2dd4bf', '#a78bfa', '#f472b6', '#fb923c', '#4ade80', '#60a5fa', '#fbbf24', '#f87171']
function avatarColor(id: string): string {
  let hash = 0
  for (let i = 0; i < id.length; i++) hash = ((hash << 5) - hash + id.charCodeAt(i)) | 0
  return colors[Math.abs(hash) % colors.length]
}

async function loadChars() {
  loading.value = true
  try {
    const ids: string[] = await invokeCommand('get_characters', {}, [])
    const chars: Character[] = []
    for (const id of ids) {
      try {
        const c = await invokeCommand<Character>('get_character', { characterId: id })
        if (c) chars.push(c)
      } catch {}
    }
    characters.value = chars
  } catch { characters.value = [] }
  loading.value = false
}

onMounted(loadChars)
</script>

<style scoped>
.gallery-page { padding: 24px; }
.gallery-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 24px; }
.gallery-header h1 { font-size: 24px; font-weight: 700; }
.loading-state, .empty-state { display: flex; align-items: center; gap: 12px; justify-content: center; padding: 80px 0; color: var(--text-secondary); }
.char-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 16px; }
.char-card { display: flex; gap: 16px; padding: 20px; background: var(--surface-1); border: 1px solid var(--border); border-radius: var(--radius); cursor: pointer; transition: all var(--transition-fast); }
.char-card:hover { border-color: var(--brand); background: var(--surface-2); }
.char-avatar { width: 48px; height: 48px; border-radius: 50%; display: flex; align-items: center; justify-content: center; font-size: 20px; font-weight: 700; color: white; flex-shrink: 0; }
.char-avatar.lg { width: 64px; height: 64px; font-size: 28px; }
.char-info { flex: 1; min-width: 0; }
.char-info h3 { font-size: 15px; font-weight: 700; margin-bottom: 4px; }
.char-desc { font-size: 12px; color: var(--text-secondary); margin-bottom: 8px; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; }
.char-tags { display: flex; gap: 6px; flex-wrap: wrap; }
.detail-overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.6); display: flex; align-items: center; justify-content: center; z-index: 100; }
.detail-panel { background: var(--surface-1); border: 1px solid var(--border); border-radius: var(--radius-lg); width: 90%; max-width: 600px; max-height: 80vh; overflow-y: auto; }
.detail-header { display: flex; align-items: center; gap: 16px; padding: 24px; border-bottom: 1px solid var(--border); }
.detail-header h2 { font-size: 20px; font-weight: 700; }
.detail-body { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.field label { display: block; font-size: 11px; font-weight: 700; color: var(--text-tertiary); text-transform: uppercase; margin-bottom: 4px; }
.field p { font-size: 13px; line-height: 1.6; color: var(--text-secondary); }
.trait-grid { display: flex; flex-direction: column; gap: 8px; }
.trait { display: flex; align-items: center; gap: 12px; }
.trait-label { font-size: 12px; width: 130px; color: var(--text-secondary); }
.trait-bar { flex: 1; height: 6px; background: var(--surface-3); border-radius: 3px; overflow: hidden; }
.trait-fill { height: 100%; background: var(--brand); border-radius: 3px; transition: width 0.3s ease; }
</style>
