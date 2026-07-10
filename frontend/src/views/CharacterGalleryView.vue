<template>
  <div class="gallery-workbench">
    <header class="gallery-header">
      <div>
        <span class="eyebrow">{{ t('characters.eyebrow', 'Project cast') }}</span>
        <h1>{{ t('characters.gallery', 'Character Gallery') }}</h1>
        <p>{{ t('characters.loaded-count', '{count} characters loaded from project data.', { count: characters.length }) }}</p>
      </div>
      <div class="gallery-actions">
        <label class="search-field">
          <Search :size="15" />
          <input v-model="searchQuery" class="input" :placeholder="t('characters.search', 'Search characters...')" />
        </label>
        <button class="btn btn-secondary btn-sm" :disabled="loading" @click="loadChars">
          <RefreshCw :size="14" />
          {{ t('common.refresh', 'Refresh') }}
        </button>
        <button class="btn btn-primary btn-sm" @click="createCharacter">
          <Plus :size="14" />
          {{ t('characters.create', 'Create Character') }}
        </button>
      </div>
    </header>

    <div v-if="loading" class="loading-state">
      <LoaderCircle class="spinner" :size="24" />
      <span>{{ t('characters.loading', 'Loading characters...') }}</span>
    </div>

    <div v-else-if="filteredCharacters.length === 0" class="empty-state">
      <UsersRound :size="32" />
      <h2>{{ t('characters.no-results', 'No characters found') }}</h2>
      <p>{{ t('characters.empty-copy', 'Create a character to begin building your project cast.') }}</p>
      <button class="btn btn-primary" @click="createCharacter">
        <Plus :size="15" />
        {{ t('characters.open-editor', 'Open editor') }}
      </button>
    </div>

    <div v-else class="gallery-layout">
      <main class="character-list" :aria-label="t('characters.gallery', 'Character Gallery')">
        <article
          v-for="character in filteredCharacters"
          :key="character.id"
          class="character-row"
          :class="{ selected: selected?.id === character.id }"
          @click="selectChar(character)"
        >
          <div class="character-media" :style="{ backgroundColor: `${avatarColor(character.id)}22` }">
            <img v-if="characterImage(character)" :src="characterImage(character) || ''" :alt="character.name" />
            <span v-else>{{ initials(character.name) }}</span>
          </div>
          <div class="character-copy">
            <div class="character-title">
              <strong>{{ character.name }}</strong>
              <small>{{ character.id }}</small>
            </div>
            <p>{{ character.description || t('characters.no-description', 'No description') }}</p>
            <div class="character-tags">
              <span v-if="character.personality?.speech_style">{{ character.personality.speech_style }}</span>
              <span>{{ character.emotion || 'neutral' }}</span>
            </div>
          </div>
          <div class="row-actions">
            <button class="icon-btn" :title="t('characters.chat', 'Chat')" :aria-label="t('characters.chat', 'Chat')" @click.stop="startChat(character)"><MessageCircle :size="15" /></button>
            <button class="icon-btn" :title="t('characters.edit', 'Edit')" :aria-label="t('characters.edit', 'Edit')" @click.stop="editChar(character)"><Pencil :size="15" /></button>
          </div>
        </article>
      </main>

      <aside v-if="selected" class="detail-panel" :aria-label="t('characters.inspector', 'Character inspector')">
        <div class="detail-header">
          <div class="detail-media" :style="{ backgroundColor: `${avatarColor(selected.id)}22` }">
            <img v-if="characterImage(selected)" :src="characterImage(selected) || ''" :alt="selected.name" />
            <span v-else>{{ initials(selected.name) }}</span>
          </div>
          <div class="detail-heading">
            <span class="eyebrow">{{ t('characters.inspector', 'Character inspector') }}</span>
            <h2>{{ selected.name }}</h2>
            <p>{{ selected.id }}</p>
          </div>
        </div>

        <div class="detail-tabs" role="tablist" :aria-label="t('characters.detail-view', 'Character detail view')">
          <button :class="{ active: activeTab === 'profile' }" @click="activeTab = 'profile'">{{ t('characters.profile', 'Profile') }}</button>
          <button :class="{ active: activeTab === 'personality' }" @click="activeTab = 'personality'">{{ t('characters.personality', 'Personality') }}</button>
        </div>

        <div class="detail-content">
          <div v-if="activeTab === 'profile'" class="profile-fields">
            <section>
              <span>{{ t('characters.description', 'Description') }}</span>
              <p>{{ selected.description || t('characters.none', 'None') }}</p>
            </section>
            <section>
              <span>{{ t('characters.background', 'Background') }}</span>
              <p>{{ selected.background || t('characters.none', 'None') }}</p>
            </section>
            <div class="compact-fields">
              <section>
                <span>{{ t('characters.speech-style', 'Speech style') }}</span>
                <p>{{ selected.personality?.speech_style || t('characters.default', 'Default') }}</p>
              </section>
              <section>
                <span>{{ t('characters.current-emotion', 'Current emotion') }}</span>
                <p>{{ selected.emotion || 'neutral' }}</p>
              </section>
            </div>
          </div>

          <div v-else class="personality-panel">
            <svg viewBox="0 0 260 260" class="radar-svg" :aria-label="t('characters.personality-chart', 'Personality chart')">
              <polygon v-for="ring in 5" :key="ring" :points="radarRing(ring * 20)" fill="none" stroke="var(--border)" stroke-width="1" />
              <polygon :points="radarPolygon" fill="var(--selection)" stroke="var(--brand)" stroke-width="2" />
              <circle v-for="(point, index) in radarPoints" :key="index" :cx="point.x" :cy="point.y" r="4" fill="var(--brand)" />
              <text
                v-for="(label, index) in radarLabels"
                :key="label"
                :x="radarLabelPos(index).x"
                :y="radarLabelPos(index).y"
                text-anchor="middle"
                fill="var(--text-secondary)"
                font-size="10"
                font-weight="700"
              >{{ label }}</text>
            </svg>
            <div class="trait-list">
              <div v-for="trait in personalityTraits" :key="trait.key" class="trait-row">
                <span>{{ trait.label }}</span>
                <div class="trait-track"><div :style="{ width: `${personalityValue(trait.key) * 100}%` }"></div></div>
                <b>{{ personalityValue(trait.key).toFixed(2) }}</b>
              </div>
            </div>
          </div>
        </div>

        <div class="detail-actions">
          <button class="btn btn-primary" @click="startChat(selected)"><MessageCircle :size="15" />{{ t('characters.start-chat', 'Start chat') }}</button>
          <button class="btn btn-secondary" @click="editChar(selected)"><Pencil :size="15" />{{ t('characters.edit', 'Edit') }}</button>
        </div>
      </aside>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { LoaderCircle, MessageCircle, Pencil, Plus, RefreshCw, Search, UsersRound } from '@lucide/vue'
import { resolveAssetUrl } from '../lib/assets'
import { useI18n } from '../lib/i18n'
import { loadStoryCharacters } from '../lib/storyContent'

type DetailTab = 'profile' | 'personality'
type PersonalityTrait = 'openness' | 'conscientiousness' | 'extraversion' | 'agreeableness' | 'neuroticism'

interface CharacterPersonality extends Partial<Record<PersonalityTrait, number>> {
  speech_style?: string
}

interface Character {
  id: string
  name: string
  description: string
  background: string
  emotion: string
  personality: CharacterPersonality
  live2d_model_path: string | null
  model_3d_path: string | null
  portrait_path: string | null
  sprite_path: string | null
}

const router = useRouter()
const { t } = useI18n()
const characters = ref<Character[]>([])
const selected = ref<Character | null>(null)
const loading = ref(true)
const searchQuery = ref('')
const activeTab = ref<DetailTab>('profile')

const personalityTraits = computed(() => [
  { key: 'openness' as const, label: t('characters.trait.openness', 'Openness') },
  { key: 'conscientiousness' as const, label: t('characters.trait.conscientiousness', 'Conscientiousness') },
  { key: 'extraversion' as const, label: t('characters.trait.extraversion', 'Extraversion') },
  { key: 'agreeableness' as const, label: t('characters.trait.agreeableness', 'Agreeableness') },
  { key: 'neuroticism' as const, label: t('characters.trait.neuroticism', 'Neuroticism') },
])
const radarLabels = ['O', 'A', 'E', 'N', 'C']
const filteredCharacters = computed(() => {
  const query = searchQuery.value.trim().toLowerCase()
  if (!query) return characters.value
  return characters.value.filter(character => (
    character.name.toLowerCase().includes(query)
    || character.id.toLowerCase().includes(query)
    || character.description.toLowerCase().includes(query)
    || character.personality.speech_style?.toLowerCase().includes(query)
  ))
})
const radarTraits = computed(() => selected.value
  ? [
      personalityValue('openness'),
      personalityValue('agreeableness'),
      personalityValue('extraversion'),
      personalityValue('neuroticism'),
      personalityValue('conscientiousness'),
    ]
  : [0, 0, 0, 0, 0])
const radarPoints = computed(() => radarTraits.value.map((value, index) => {
  const angle = (Math.PI * 2 * index) / 5 - Math.PI / 2
  return { x: 130 + Math.cos(angle) * 100 * value, y: 130 + Math.sin(angle) * 100 * value }
}))
const radarPolygon = computed(() => radarPoints.value.map(point => `${point.x},${point.y}`).join(' '))
const colors = ['#2f3133', '#4c4f52', '#666a6d', '#7d8184', '#969a9c', '#aaaead', '#bfc1bf', '#d1d2cf']

function initials(name: string): string {
  return name.trim().slice(0, 2).toUpperCase() || 'AI'
}

function avatarColor(id: string): string {
  let hash = 0
  for (const character of id) hash = ((hash << 5) - hash + character.charCodeAt(0)) | 0
  return colors[Math.abs(hash) % colors.length]
}

function characterImage(character: Character): string | null {
  return resolveAssetUrl(character.portrait_path || character.sprite_path)
}

function personalityValue(key: PersonalityTrait): number {
  const value = Number(selected.value?.personality?.[key] || 0)
  return Math.max(0, Math.min(1, Number.isFinite(value) ? value : 0))
}

function radarRing(radius: number): string {
  return Array.from({ length: 5 }, (_, index) => {
    const angle = (Math.PI * 2 * index) / 5 - Math.PI / 2
    return `${130 + Math.cos(angle) * radius},${130 + Math.sin(angle) * radius}`
  }).join(' ')
}

function radarLabelPos(index: number) {
  const angle = (Math.PI * 2 * index) / 5 - Math.PI / 2
  return { x: 130 + Math.cos(angle) * 118, y: 134 + Math.sin(angle) * 118 }
}

function selectChar(character: Character) {
  selected.value = character
  activeTab.value = 'profile'
}

function startChat(character: Character) {
  void router.push({ path: '/chat', query: { character: character.id } })
}

function editChar(character: Character) {
  void router.push({ path: '/character-editor', query: { character: character.id } })
}

function createCharacter() {
  void router.push({ path: '/character-editor', query: { create: '1' } })
}

async function loadChars() {
  loading.value = true
  try {
    characters.value = (await loadStoryCharacters()).map(character => ({
      ...character,
      background: character.background || '',
      personality: (character.personality || {}) as CharacterPersonality,
      live2d_model_path: character.live2d_model_path ?? null,
      model_3d_path: character.model_3d_path ?? null,
      portrait_path: character.portrait_path ?? null,
      sprite_path: character.sprite_path ?? null,
    }))
    selected.value = characters.value.find(character => character.id === selected.value?.id) || characters.value[0] || null
  } catch {
    characters.value = []
    selected.value = null
  } finally {
    loading.value = false
  }
}

onMounted(loadChars)
</script>

<style scoped>
.gallery-workbench { max-width: 1280px; margin: 0 auto; padding: 32px 36px 48px; }
.gallery-header { display: flex; align-items: flex-start; justify-content: space-between; gap: 18px; margin-bottom: 20px; }
.gallery-header h1 { margin: 3px 0 0; color: var(--text-primary); font-size: 28px; line-height: 1.15; }
.gallery-header p { margin: 7px 0 0; color: var(--text-secondary); font-size: 13px; }
.eyebrow { display: block; color: var(--text-tertiary); font-size: 11px; font-weight: 800; text-transform: uppercase; }
.gallery-actions { display: flex; align-items: center; gap: 8px; flex-shrink: 0; }
.search-field { display: flex; align-items: center; gap: 7px; min-width: 230px; height: 34px; padding: 0 10px; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--surface-1); color: var(--text-tertiary); }
.search-field .input { min-width: 0; height: 30px; padding: 0; border: 0; background: transparent; }
.btn { display: inline-flex; align-items: center; justify-content: center; gap: 7px; }
.loading-state, .empty-state { display: grid; place-items: center; gap: 10px; min-height: 360px; color: var(--text-tertiary); text-align: center; }
.empty-state h2 { margin: 0; color: var(--text-primary); font-size: 18px; }
.empty-state p { max-width: 440px; margin: 0; font-size: 12px; }
.spinner { animation: spin 0.8s linear infinite; }
.gallery-layout { display: grid; grid-template-columns: minmax(0, 1fr) 360px; gap: 16px; align-items: start; }
.character-list { display: grid; gap: 8px; }
.character-row { display: grid; grid-template-columns: 72px minmax(0, 1fr) auto; gap: 13px; align-items: center; min-height: 96px; padding: 11px; border: 1px solid var(--border); border-radius: var(--radius); background: var(--surface-1); cursor: pointer; transition: border-color var(--transition-fast), background var(--transition-fast); }
.character-row:hover, .character-row.selected { border-color: var(--brand); background: var(--surface-2); }
.character-media, .detail-media { display: grid; place-items: center; overflow: hidden; color: var(--brand-light); font-weight: 850; }
.character-media { width: 72px; height: 72px; border-radius: var(--radius); }
.character-media img, .detail-media img { width: 100%; height: 100%; object-fit: cover; }
.character-copy { display: grid; gap: 5px; min-width: 0; }
.character-title { display: flex; align-items: baseline; gap: 8px; min-width: 0; }
.character-title strong { color: var(--text-primary); font-size: 14px; }
.character-title small { overflow: hidden; color: var(--text-tertiary); font-family: var(--font-mono); font-size: 10px; text-overflow: ellipsis; white-space: nowrap; }
.character-copy > p { display: -webkit-box; overflow: hidden; margin: 0; color: var(--text-tertiary); font-size: 11px; line-height: 1.45; -webkit-box-orient: vertical; -webkit-line-clamp: 2; }
.character-tags { display: flex; gap: 5px; overflow: hidden; }
.character-tags span { max-width: 210px; overflow: hidden; padding: 2px 7px; border: 1px solid var(--border); border-radius: 999px; color: var(--text-secondary); font-size: 9px; text-overflow: ellipsis; white-space: nowrap; }
.row-actions { display: flex; gap: 5px; }
.icon-btn { display: grid; place-items: center; width: 32px; height: 32px; padding: 0; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--surface-2); color: var(--text-secondary); cursor: pointer; }
.icon-btn:hover { border-color: var(--brand); color: var(--brand-light); }
.detail-panel { position: sticky; top: 16px; overflow: hidden; border: 1px solid var(--border); border-radius: var(--radius); background: var(--surface-1); }
.detail-header { display: grid; grid-template-columns: 64px minmax(0, 1fr); gap: 13px; align-items: center; padding: 16px; border-bottom: 1px solid var(--border); }
.detail-media { width: 64px; height: 64px; border-radius: var(--radius); }
.detail-heading { min-width: 0; }
.detail-heading h2 { overflow: hidden; margin: 3px 0 0; color: var(--text-primary); font-size: 18px; text-overflow: ellipsis; white-space: nowrap; }
.detail-heading p { overflow: hidden; margin: 3px 0 0; color: var(--text-tertiary); font-family: var(--font-mono); font-size: 10px; text-overflow: ellipsis; white-space: nowrap; }
.detail-tabs { display: grid; grid-template-columns: repeat(2, 1fr); border-bottom: 1px solid var(--border); }
.detail-tabs button { min-height: 38px; border: 0; border-bottom: 2px solid transparent; background: transparent; color: var(--text-secondary); cursor: pointer; font: inherit; font-size: 11px; font-weight: 750; }
.detail-tabs button.active { border-bottom-color: var(--brand); color: var(--brand-light); }
.detail-content { min-height: 330px; padding: 16px; }
.profile-fields { display: grid; gap: 15px; }
.profile-fields section > span { display: block; margin-bottom: 4px; color: var(--text-tertiary); font-size: 10px; font-weight: 800; text-transform: uppercase; }
.profile-fields section > p { margin: 0; color: var(--text-secondary); font-size: 12px; line-height: 1.55; white-space: pre-wrap; }
.compact-fields { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 12px; }
.personality-panel { display: grid; justify-items: center; gap: 12px; }
.radar-svg { width: 190px; height: 190px; }
.trait-list { display: grid; gap: 8px; width: 100%; }
.trait-row { display: grid; grid-template-columns: 110px minmax(0, 1fr) 34px; gap: 8px; align-items: center; }
.trait-row > span { color: var(--text-secondary); font-size: 10px; }
.trait-row > b { color: var(--brand-light); font-family: var(--font-mono); font-size: 10px; text-align: right; }
.trait-track { height: 5px; overflow: hidden; border-radius: 999px; background: var(--surface-3); }
.trait-track > div { height: 100%; border-radius: inherit; background: var(--brand); }
.detail-actions { display: grid; grid-template-columns: 1fr auto; gap: 7px; padding: 13px 16px; border-top: 1px solid var(--border); }
@keyframes spin { to { transform: rotate(360deg); } }

@media (max-width: 1040px) {
  .gallery-layout { grid-template-columns: 1fr; }
  .detail-panel { position: static; grid-row: 1; }
  .character-list { grid-row: 2; }
}

@media (max-width: 720px) {
  .gallery-workbench { padding: 22px 16px 96px; }
  .gallery-header { flex-direction: column; }
  .gallery-actions { display: grid; grid-template-columns: 1fr auto; width: 100%; }
  .search-field { grid-column: 1 / -1; min-width: 0; }
  .gallery-actions .btn-primary { min-width: 0; }
  .character-row { grid-template-columns: 58px minmax(0, 1fr); min-height: 84px; }
  .character-media { width: 58px; height: 58px; }
  .row-actions { grid-column: 1 / -1; justify-content: flex-end; }
}
</style>
