<template>
  <div class="editor-workbench">
    <aside class="char-rail">
      <div class="rail-header">
        <div>
          <span class="eyebrow">Content</span>
          <h1>Characters</h1>
        </div>
        <button class="btn btn-primary btn-sm" @click="createNew">+ New</button>
      </div>
      <div class="char-list">
        <button
          v-for="char in characterList"
          :key="char.id"
          class="char-card"
          :class="{ selected: selectedId === char.id }"
          @click="selectChar(char.id)"
        >
          <span class="avatar" :style="{ background: avatarColor(char.id) }">{{ initials(char.name) }}</span>
          <div class="char-info">
            <strong>{{ char.name }}</strong>
            <small>{{ char.description || 'No description' }}</small>
          </div>
          <span class="char-emotion">{{ char.emotion || 'neutral' }}</span>
        </button>
        <div v-if="characterList.length === 0" class="empty-list">
          <span class="empty-mark">--</span>
          <span>No characters loaded</span>
        </div>
      </div>
    </aside>

    <main v-if="editing" class="editor-main">
      <header class="editor-toolbar">
        <div class="toolbar-left">
          <span class="eyebrow">Editing</span>
          <h2>{{ isNew ? 'New Character' : form.name }}</h2>
        </div>
        <div class="toolbar-right">
          <button class="btn btn-secondary btn-sm" @click="exportChar">{{ t("characters.export", "Export JSON") }}</button>
          <button class="btn btn-secondary btn-sm" @click="cancelEdit">Cancel</button>
          <button class="btn btn-primary btn-sm" :disabled="saving" @click="save">
            {{ saving ? 'Saving' : 'Save' }}
          </button>
        </div>
      </header>

      <div class="editor-tabs">
        <button
          v-for="tab in tabs"
          :key="tab.key"
          class="tab-btn"
          :class="{ active: activeTab === tab.key }"
          @click="activeTab = tab.key"
        >{{ tab.label }}</button>
      </div>

      <div class="editor-content">
        <!-- Basic Info Tab -->
        <div v-if="activeTab === 'basic'" class="tab-panel">
          <div class="section">
            <span class="section-title">Identity</span>
            <div class="form-grid">
              <label class="form-field">
                <span>Character ID</span>
                <input class="input" v-model="form.id" placeholder="unique_id" :disabled="!isNew" />
              </label>
              <label class="form-field">
                <span>Display Name</span>
                <input class="input" v-model="form.name" placeholder="Character Name" />
              </label>
              <label class="form-field full">
                <span>Description</span>
                <textarea class="input" v-model="form.description" rows="2" placeholder="Short description shown in character list"></textarea>
              </label>
              <label class="form-field full">
                <span>Background Story</span>
                <textarea class="input" v-model="form.background" rows="5" placeholder="Character background and history"></textarea>
              </label>
            </div>
          </div>
          <div class="section">
            <span class="section-title">Speech and Appearance</span>
            <div class="form-grid">
              <label class="form-field">
                <span>Speech Style</span>
                <input class="input" v-model="form.speech_style" placeholder="e.g. cheerful, formal, mysterious" />
              </label>
              <label class="form-field">
                <span>Default Emotion</span>
                <select class="input" v-model="form.default_emotion">
                  <option v-for="e in emotions" :key="e" :value="e">{{ e }}</option>
                </select>
              </label>
              <label class="form-field" :class="{ invalid: assetFieldIssue('live2d_model_path') }">
                <span>Live2D Model Path</span>
                <input class="input" v-model="form.live2d_model_path" placeholder="live2d/model.model3.json" />
                <small v-if="assetFieldIssue('live2d_model_path')" class="field-warning">{{ assetFieldIssue('live2d_model_path')?.message }}</small>
              </label>
              <label class="form-field" :class="{ invalid: assetFieldIssue('model_3d_path') }">
                <span>3D Model Path (GLB/GLTF)</span>
                <input class="input" v-model="form.model_3d_path" placeholder="models/character.glb" />
                <small v-if="assetFieldIssue('model_3d_path')" class="field-warning">{{ assetFieldIssue('model_3d_path')?.message }}</small>
              </label>
              <label class="form-field" :class="{ invalid: assetFieldIssue('portrait_path') }">
                <span>Portrait Image</span>
                <input class="input" v-model="form.portrait_path" placeholder="assets/portraits/char.png" />
                <small v-if="assetFieldIssue('portrait_path')" class="field-warning">{{ assetFieldIssue('portrait_path')?.message }}</small>
              </label>
              <label class="form-field" :class="{ invalid: assetFieldIssue('sprite_path') }">
                <span>Fallback Sprite</span>
                <input class="input" v-model="form.sprite_path" placeholder="assets/sprites/char.png" />
                <small v-if="assetFieldIssue('sprite_path')" class="field-warning">{{ assetFieldIssue('sprite_path')?.message }}</small>
              </label>
            </div>
            <div class="asset-diagnostics" :class="{ warning: assetIssueCount > 0 }">
              <div class="asset-diag-head">
                <span>Asset Contract</span>
                <strong :class="{ warning: assetIssueCount > 0 }">{{ rendererAssetSummary }}</strong>
              </div>
              <div v-if="assetDiagnostics.length > 0" class="asset-diag-list">
                <span v-for="diag in assetDiagnostics" :key="diag.key" class="asset-diag-row" :class="diag.state">
                  <b>{{ diag.label }}</b>
                  <span>{{ diag.message }}</span>
                </span>
              </div>
              <div v-else class="asset-diag-empty">Generated 3D fallback</div>
            </div>
          </div>
          <div class="section">
            <div class="section-heading">
              <span class="section-title">Renderer Preview</span>
              <span class="preview-mode">{{ rendererPreviewMode }}</span>
            </div>
            <div class="renderer-preview-stage">
              <Live2DCanvas
                v-if="previewLive2dPath"
                :model-path="previewLive2dPath"
                :expression="previewExpression"
                motion="idle"
                @load-error="markPreviewRendererAssetFailed"
              />
              <CharacterModelView
                v-else-if="previewModel3dPath"
                :model-path="previewModel3dPath"
                :expression="previewExpression"
                motion="idle"
                @load-error="markPreviewRendererAssetFailed"
              />
              <div v-else-if="previewSpritePath" class="sprite-preview">
                <img :src="previewSpritePath" :alt="form.name || 'Character sprite preview'" />
              </div>
              <CharacterModelView
                v-else
                :model-path="null"
                :expression="previewExpression"
                motion="idle"
              />
            </div>
          </div>
        </div>

        <!-- Personality Tab -->
        <div v-if="activeTab === 'personality'" class="tab-panel">
          <div class="section">
            <span class="section-title">Big Five Personality Traits</span>
            <p class="section-desc">These traits shape how the LLM generates character responses. Higher values amplify the trait in conversations.</p>
            <div class="trait-columns">
              <div class="trait-list">
                <div v-for="trait in personalityTraits" :key="trait.key" class="trait-item">
                  <div class="trait-header">
                    <label>{{ trait.label }}</label>
                    <span class="trait-val">{{ form[trait.key].toFixed(2) }}</span>
                  </div>
                  <input type="range" v-model.number="form[trait.key]" min="0" max="1" step="0.05" />
                  <p class="trait-desc">{{ trait.desc }}</p>
                </div>
              </div>
              <div class="radar-chart">
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
            </div>
          </div>
        </div>

        <!-- Emotions Tab -->
        <div v-if="activeTab === 'emotions'" class="tab-panel">
          <div class="section">
            <span class="section-title">Emotion Configuration</span>
            <p class="section-desc">Configure how the character expresses different emotions in dialogue and chat.</p>
            <div class="emotion-grid">
              <div v-for="em in emotionConfigs" :key="em.name" class="emotion-card">
                <div class="emotion-header">
                  <span class="emotion-icon">{{ em.icon }}</span>
                  <strong>{{ em.name }}</strong>
                </div>
                <label class="form-field">
                  <span>Speech Modifier</span>
                  <input class="input" v-model="form.emotion_modifiers[em.key]" :placeholder="em.default_modifier" />
                </label>
              </div>
            </div>
          </div>
          <div class="section">
            <div class="section-heading">
              <span class="section-title">Expression Sprites</span>
              <button class="btn btn-secondary btn-sm" :disabled="!form.sprite_path.trim()" @click="fillDefaultSpritePaths">Copy Fallback</button>
            </div>
            <div class="sprite-path-grid">
              <label
                v-for="emotion in emotions"
                :key="emotion"
                class="form-field sprite-path-row"
                :class="{ invalid: spritePathIssue(emotion) }"
              >
                <span>{{ emotion }}</span>
                <input class="input" v-model="form.sprite_paths[emotion]" :placeholder="spritePlaceholder(emotion)" />
                <small v-if="spritePathIssue(emotion)" class="field-warning">{{ spritePathIssue(emotion)?.message }}</small>
              </label>
            </div>
          </div>
        </div>

        <!-- Relationships Tab -->
        <div v-if="activeTab === 'relationships'" class="tab-panel">
          <div class="section">
            <span class="section-title">Default Relationships</span>
            <p class="section-desc">Set initial relationship scores with other characters. Range from -1.0 (hostile) to 1.0 (close bond).</p>
            <div class="rel-list">
              <div v-for="other in otherCharacters" :key="other.id" class="rel-item">
                <span class="rel-avatar" :style="{ background: avatarColor(other.id) }">{{ initials(other.name) }}</span>
                <span class="rel-name">{{ other.name }}</span>
                <input type="range" v-model.number="form.relationships[other.id]" min="-1" max="1" step="0.1" />
                <span class="rel-val">{{ (form.relationships[other.id] || 0).toFixed(1) }}</span>
              </div>
              <p v-if="otherCharacters.length === 0" class="muted">No other characters available. Create more characters first.</p>
            </div>
          </div>
        </div>

        <!-- Knowledge Tab -->
        <div v-if="activeTab === 'knowledge'" class="tab-panel">
          <div class="section">
            <span class="section-title">Character Knowledge</span>
            <p class="section-desc">Define what this character knows. These entries are injected into the LLM context during conversations.</p>
            <label class="form-field full">
              <span>Pinned Knowledge Refs</span>
              <input class="input" v-model="form.knowledge_refs" placeholder="sakura_nature, sakura_art_knowledge" />
            </label>
            <div class="knowledge-editor">
              <div v-for="(entry, i) in form.knowledge_entries" :key="i" class="knowledge-item">
                <label class="form-field">
                  <span>Topic</span>
                  <input class="input" v-model="entry.topic" placeholder="e.g. My hometown" />
                </label>
                <label class="form-field full">
                  <span>Content</span>
                  <textarea class="input" v-model="entry.content" rows="2" placeholder="What the character knows"></textarea>
                </label>
                <button class="btn-icon danger" @click="removeKnowledge(i)" title="Remove">x</button>
              </div>
              <button class="btn btn-secondary btn-sm" @click="addKnowledge">+ Add Knowledge Entry</button>
            </div>
          </div>
        </div>
      </div>
    </main>

    <main v-else class="editor-empty">
      <div class="empty-state">
        <span class="empty-mark">CE</span>
        <h2>Character Editor</h2>
        <p>Select a character from the list to edit, or create a new one. Configure personality, appearance, knowledge, and relationships.</p>
        <div class="quick-stats">
          <span><strong>{{ characterList.length }}</strong> characters loaded</span>
        </div>
      </div>
    </main>

    <Transition name="fade">
      <div v-if="statusMsg" class="status-toast" :class="{ error: !statusOk }" @click="statusMsg = null">
        {{ statusMsg }}
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, reactive, ref, onMounted, watch } from 'vue'
import { useRoute } from 'vue-router'
import Live2DCanvas from '../components/Live2DCanvas.vue'
import CharacterModelView from '../components/CharacterModelView.vue'
import {
  cleanRendererPathMap,
  imageAssetExtensions,
  rendererAssetSpecs,
  rendererAssetValidationMessage,
  selectCharacterRendererAsset,
  type RendererAssetSpec,
} from '../lib/rendererAssets'
import { hasTauriRuntime, invokeCommand } from '../lib/tauri'
import { useI18n } from '../lib/i18n'
import { loadStoryCharacters, type StoryCharacterInfo } from '../lib/storyContent'

const { t } = useI18n()
const route = useRoute()

interface KnowledgeEntry {
  topic: string
  content: string
}

interface CharForm {
  id: string
  name: string
  description: string
  background: string
  speech_style: string
  default_emotion: string
  live2d_model_path: string
  model_3d_path: string
  portrait_path: string
  sprite_path: string
  sprite_paths: Record<string, string>
  openness: number
  conscientiousness: number
  extraversion: number
  agreeableness: number
  neuroticism: number
  relationships: Record<string, number>
  knowledge_entries: KnowledgeEntry[]
  knowledge_refs: string
  emotion_modifiers: Record<string, string>
  [key: string]: any
}

interface CharacterSummary {
  id: string
  name: string
  description: string
  emotion: string
  live2d_model_path: string | null
}

interface AssetDiagnostic {
  key: string
  label: string
  message: string
  state: 'ready' | 'warning'
}

const emotions = ['neutral', 'happy', 'sad', 'angry', 'surprised', 'love', 'embarrassed', 'thoughtful', 'excited', 'anxious']

const personalityTraits = [
  { key: 'openness', label: 'Openness', desc: 'Curiosity, creativity, and willingness to try new things.' },
  { key: 'conscientiousness', label: 'Conscientiousness', desc: 'Organization, discipline, and goal-oriented behavior.' },
  { key: 'extraversion', label: 'Extraversion', desc: 'Sociability, assertiveness, and energy from interactions.' },
  { key: 'agreeableness', label: 'Agreeableness', desc: 'Compassion, cooperation, and trust toward others.' },
  { key: 'neuroticism', label: 'Neuroticism', desc: 'Emotional sensitivity, anxiety, and mood variability.' },
]

const emotionConfigs = [
  { name: 'Happy', key: 'happy', icon: '\u263A', default_modifier: 'cheerful, upbeat, uses exclamations' },
  { name: 'Sad', key: 'sad', icon: '\u2639', default_modifier: 'quiet, hesitant, trailing off' },
  { name: 'Angry', key: 'angry', icon: '\u2620', default_modifier: 'sharp, direct, shorter sentences' },
  { name: 'Surprised', key: 'surprised', icon: '!', default_modifier: 'exclamatory, questions, disbelief' },
  { name: 'Love', key: 'love', icon: '\u2665', default_modifier: 'warm, tender, softer tone' },
  { name: 'Embarrassed', key: 'embarrassed', icon: '~', default_modifier: 'stammering, deflecting, nervous laughter' },
]

const tabs = [
  { key: 'basic', label: 'Basic Info' },
  { key: 'personality', label: 'Personality' },
  { key: 'emotions', label: 'Emotions' },
  { key: 'relationships', label: 'Relationships' },
  { key: 'knowledge', label: 'Knowledge' },
]

const characterList = ref<CharacterSummary[]>([])
const browserCharacters = ref<StoryCharacterInfo[]>([])
const selectedId = ref<string | null>(null)
const editing = ref(false)
const isNew = ref(false)
const saving = ref(false)
const activeTab = ref('basic')
const statusMsg = ref<string | null>(null)
const statusOk = ref(true)

const defaultForm = (): CharForm => ({
  id: '', name: '', description: '', background: '', speech_style: '',
  default_emotion: 'neutral', live2d_model_path: '', model_3d_path: '',
  portrait_path: '', sprite_path: '', sprite_paths: {},
  openness: 0.5, conscientiousness: 0.5, extraversion: 0.5, agreeableness: 0.5, neuroticism: 0.5,
  relationships: {}, knowledge_entries: [], knowledge_refs: '', emotion_modifiers: {},
})

const form = reactive<CharForm>(defaultForm())
const previewFailedRendererAssets = ref<Record<string, true>>({})

const otherCharacters = computed(() =>
  characterList.value.filter(c => c.id !== form.id)
)

const assetDiagnostics = computed<AssetDiagnostic[]>(() => {
  const diagnostics: AssetDiagnostic[] = []

  for (const spec of rendererAssetSpecs) {
    const path = form[spec.key]
    const message = rendererAssetValidationMessage(path, spec.extensions)
    if (message) {
      diagnostics.push({ key: spec.key, label: spec.label, message, state: 'warning' })
    } else if (path.trim()) {
      diagnostics.push({ key: spec.key, label: spec.label, message: 'Ready', state: 'ready' })
    }
  }

  for (const [emotion, path] of Object.entries(cleanSpritePaths(form.sprite_paths))) {
    const key = `sprite_paths.${emotion}`
    const message = rendererAssetValidationMessage(path, imageAssetExtensions)
    diagnostics.push({
      key,
      label: `${emotion} sprite`,
      message: message || 'Ready',
      state: message ? 'warning' : 'ready',
    })
  }

  return diagnostics
})

const assetIssueCount = computed(() =>
  assetDiagnostics.value.filter(diag => diag.state === 'warning').length
)

const rendererAssetSummary = computed(() => {
  if (assetIssueCount.value > 0) return `${assetIssueCount.value} warning${assetIssueCount.value === 1 ? '' : 's'}`
  const declaredCount = assetDiagnostics.value.length
  return declaredCount > 0 ? `${declaredCount} ready` : 'Fallback ready'
})

const assetIssueMap = computed(() => {
  const entries = assetDiagnostics.value
    .filter(diag => diag.state === 'warning')
    .map(diag => [diag.key, diag] as const)
  return Object.fromEntries(entries) as Record<string, AssetDiagnostic>
})

const previewExpression = computed(() => form.default_emotion || 'neutral')

const rendererPreviewAsset = computed(() =>
  selectCharacterRendererAsset(
    {
      live2d_model_path: form.live2d_model_path,
      model_3d_path: form.model_3d_path,
      portrait_path: form.portrait_path,
      sprite_path: form.sprite_path,
      sprite_paths: form.sprite_paths,
      emotion: form.default_emotion,
    },
    {
      expression: previewExpression.value,
      validatePaths: true,
      blockedPaths: Object.keys(previewFailedRendererAssets.value),
    },
  )
)

const previewLive2dPath = computed(() =>
  rendererPreviewAsset.value.mode === 'live2d' ? rendererPreviewAsset.value.resolvedUrl : null
)

const previewModel3dPath = computed(() =>
  rendererPreviewAsset.value.mode === 'model3d' ? rendererPreviewAsset.value.resolvedUrl : null
)

const previewSpritePath = computed(() =>
  rendererPreviewAsset.value.mode === 'sprite' ? rendererPreviewAsset.value.resolvedUrl : null
)

const rendererPreviewMode = computed(() => {
  if (rendererPreviewAsset.value.mode === 'live2d') return 'Live2D'
  if (rendererPreviewAsset.value.mode === 'model3d') return '3D'
  if (rendererPreviewAsset.value.mode === 'sprite') return 'Sprite'
  return 'Generated 3D'
})

function markPreviewRendererAssetFailed(payload: { path: string | null; message: string }) {
  const path = payload.path?.trim()
  if (!path) return
  previewFailedRendererAssets.value = { ...previewFailedRendererAssets.value, [path]: true }
}

watch(
  () => [
    form.live2d_model_path,
    form.model_3d_path,
    form.portrait_path,
    form.sprite_path,
    JSON.stringify(cleanSpritePaths(form.sprite_paths)),
  ],
  () => {
    previewFailedRendererAssets.value = {}
  },
)

const radarTraits = computed(() => [
  form.openness, form.agreeableness, form.extraversion, form.neuroticism, form.conscientiousness
])

const radarPoints = computed(() => {
  const cx = 130, cy = 130, maxR = 100
  return radarTraits.value.map((val, i) => {
    const angle = (Math.PI * 2 * i) / 5 - Math.PI / 2
    return { x: cx + Math.cos(angle) * maxR * val, y: cy + Math.sin(angle) * maxR * val }
  })
})

const radarPolygon = computed(() => radarPoints.value.map(p => p.x + ',' + p.y).join(' '))

const radarLabels = ['O', 'A', 'E', 'N', 'C']

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

function avatarColor(id: string): string {
  const hue = Array.from(id).reduce((s, c) => s + c.charCodeAt(0), 0) * 37 % 360
  return 'hsl(' + hue + ', 55%, 45%)'
}

function initials(name: string): string {
  return name.trim().slice(0, 2).toUpperCase() || '??'
}

function normalizeSpritePaths(value: unknown): Record<string, string> {
  return cleanRendererPathMap(value)
}

function cleanSpritePaths(paths: Record<string, string>): Record<string, string> {
  return cleanRendererPathMap(paths)
}

function spritePlaceholder(emotion: string): string {
  if (form.sprite_path.trim()) return form.sprite_path.trim()
  const base = form.id.trim() || 'character'
  return 'assets/sprites/' + base + '_' + emotion + '.png'
}

function fillDefaultSpritePaths() {
  const fallback = form.sprite_path.trim()
  if (!fallback) return
  for (const emotion of emotions) {
    if (!form.sprite_paths[emotion]?.trim()) {
      form.sprite_paths[emotion] = fallback
    }
  }
}

function assetFieldIssue(key: RendererAssetSpec['key']): AssetDiagnostic | undefined {
  return assetIssueMap.value[key]
}

function spritePathIssue(emotion: string): AssetDiagnostic | undefined {
  return assetIssueMap.value[`sprite_paths.${emotion}`]
}

function resetForm() {
  Object.assign(form, defaultForm())
}

function createNew() {
  resetForm()
  isNew.value = true
  editing.value = true
  activeTab.value = 'basic'
}

function cancelEdit() {
  editing.value = false
  isNew.value = false
  selectedId.value = null
}

async function loadList() {
  try {
    const characters = await loadStoryCharacters()
    browserCharacters.value = characters
    characterList.value = characters.map(character => ({
      id: character.id,
      name: character.name,
      description: character.description,
      emotion: character.emotion,
      live2d_model_path: character.live2d_model_path ?? null,
    }))
  } catch (e) {
    console.error('Failed to load characters:', e)
  }
}

async function selectChar(id: string) {
  try {
    const c = hasTauriRuntime()
      ? await invokeCommand<any>('get_character', { characterId: id })
      : browserCharacters.value.find(character => character.id === id)
    if (!c) return
    selectedId.value = id
    isNew.value = false
    Object.assign(form, {
      id: c.id, name: c.name || '', description: c.description || '',
      background: c.background || '', speech_style: c.personality?.speech_style || '',
      default_emotion: c.emotion || 'neutral',
      live2d_model_path: c.live2d_model_path || '', model_3d_path: c.model_3d_path || '',
      portrait_path: c.portrait_path || '', sprite_path: c.sprite_path || '',
      sprite_paths: normalizeSpritePaths(c.sprite_paths),
      openness: c.personality?.openness ?? 0.5,
      conscientiousness: c.personality?.conscientiousness ?? 0.5,
      extraversion: c.personality?.extraversion ?? 0.5,
      agreeableness: c.personality?.agreeableness ?? 0.5,
      neuroticism: c.personality?.neuroticism ?? 0.5,
      relationships: c.relationships || {},
      knowledge_entries: c.knowledge_entries || [],
      knowledge_refs: (c.knowledge_refs || c.knowledge || []).join(', '),
      emotion_modifiers: c.emotion_modifiers || {},
    })
    editing.value = true
    activeTab.value = 'basic'
  } catch (e) {
    statusMsg.value = 'Failed to load character: ' + String(e)
    statusOk.value = false
  }
}

async function save() {
  if (!form.id.trim() || !form.name.trim()) {
    statusMsg.value = 'ID and Name are required'
    statusOk.value = false
    return
  }
  saving.value = true
  try {
    await invokeCommand('create_character', {
      input: {
        id: form.id,
        name: form.name,
        description: form.description,
        background: form.background,
        personality: {
          openness: form.openness,
          conscientiousness: form.conscientiousness,
          extraversion: form.extraversion,
          agreeableness: form.agreeableness,
          neuroticism: form.neuroticism,
          speech_style: form.speech_style,
        },
        default_emotion: form.default_emotion,
        live2d_model_path: form.live2d_model_path || null,
        model_3d_path: form.model_3d_path || null,
        portrait_path: form.portrait_path || null,
        sprite_path: form.sprite_path || null,
        sprite_paths: cleanSpritePaths(form.sprite_paths),
        relationships: form.relationships,
        knowledge_entries: form.knowledge_entries,
        knowledge_refs: splitKnowledgeRefs(form.knowledge_refs),
        emotion_modifiers: form.emotion_modifiers,
      }
    })
    statusMsg.value = assetIssueCount.value > 0
      ? 'Character "' + form.name + '" saved with ' + assetIssueCount.value + ' asset warning(s)'
      : 'Character "' + form.name + '" saved'
    statusOk.value = assetIssueCount.value === 0
    editing.value = false
    isNew.value = false
    await loadList()
  } catch (e) {
    statusMsg.value = 'Save failed: ' + String(e)
    statusOk.value = false
  } finally {
    saving.value = false
  }
}

function exportChar() {
  const data = {
    id: form.id, name: form.name, description: form.description,
    background: form.background,
    personality: {
      openness: form.openness, conscientiousness: form.conscientiousness,
      extraversion: form.extraversion, agreeableness: form.agreeableness,
      neuroticism: form.neuroticism, speech_style: form.speech_style,
    },
    emotion: form.default_emotion,
    live2d_model_path: form.live2d_model_path || null,
    model_3d_path: form.model_3d_path || null,
    portrait_path: form.portrait_path || null,
    sprite_path: form.sprite_path || null,
    sprite_paths: cleanSpritePaths(form.sprite_paths),
    relationships: form.relationships,
    knowledge_entries: form.knowledge_entries,
    knowledge_refs: splitKnowledgeRefs(form.knowledge_refs),
    emotion_modifiers: form.emotion_modifiers,
  }
  const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = (form.id || 'character') + '.json'
  a.click()
  URL.revokeObjectURL(url)
  statusMsg.value = assetIssueCount.value > 0
    ? 'Exported with ' + assetIssueCount.value + ' asset warning(s)'
    : 'Character JSON exported'
  statusOk.value = assetIssueCount.value === 0
}

function addKnowledge() {
  form.knowledge_entries.push({ topic: '', content: '' })
}

function removeKnowledge(i: number) {
  form.knowledge_entries.splice(i, 1)
}

function splitKnowledgeRefs(value: string): string[] {
  return value.split(',')
    .map(ref => ref.trim())
    .filter(Boolean)
}

onMounted(async () => {
  await loadList()
  if (route.query.create === '1') {
    createNew()
    return
  }
  const requestedCharacter = typeof route.query.character === 'string' ? route.query.character : ''
  if (requestedCharacter) await selectChar(requestedCharacter)
})
</script>

<style scoped>
.editor-workbench {
  display: grid;
  grid-template-columns: 300px minmax(0, 1fr);
  height: 100vh;
  min-height: 0;
  background: var(--surface-0);
}

.char-rail {
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--border);
  background: var(--surface-1);
}

.rail-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 16px 16px;
  border-bottom: 1px solid var(--border);
}

.rail-header h1 {
  color: var(--text-primary);
  font-size: 20px;
  font-weight: 750;
  margin-top: 3px;
}

.eyebrow {
  display: block;
  color: var(--text-tertiary);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0;
  text-transform: uppercase;
}

.char-list {
  flex: 1;
  overflow-y: auto;
  padding: 10px;
}

.char-card {
  width: 100%;
  display: flex;
  gap: 12px;
  align-items: center;
  padding: 12px;
  margin-bottom: 6px;
  border: 1px solid transparent;
  border-radius: var(--radius);
  background: transparent;
  color: var(--text-primary);
  cursor: pointer;
  text-align: left;
  transition: background var(--transition-fast), border-color var(--transition-fast);
}

.char-card:hover, .char-card.selected {
  background: var(--surface-2);
  border-color: var(--border-light);
}

.char-card.selected {
  border-color: var(--brand);
}

.avatar {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 38px;
  height: 38px;
  border-radius: var(--radius-sm);
  color: white;
  font-weight: 800;
  font-size: 13px;
  flex-shrink: 0;
}

.char-info {
  min-width: 0;
  flex: 1;
  display: grid;
  gap: 2px;
}

.char-info strong {
  font-size: 13px;
  font-weight: 700;
}

.char-info small {
  overflow: hidden;
  color: var(--text-tertiary);
  font-size: 11px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.char-emotion {
  font-size: 11px;
  color: var(--text-tertiary);
  padding: 2px 8px;
  border: 1px solid var(--border);
  border-radius: 999px;
  flex-shrink: 0;
}

.empty-list {
  display: grid;
  place-items: center;
  gap: 8px;
  padding: 40px 20px;
  color: var(--text-tertiary);
  text-align: center;
}

.empty-mark {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 42px;
  height: 42px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-2);
  color: var(--brand-light);
  font-family: var(--font-mono);
  font-weight: 900;
}

.editor-main {
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.editor-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 14px 24px;
  border-bottom: 1px solid var(--border);
  background: var(--surface-1);
}

.toolbar-left h2 {
  color: var(--text-primary);
  font-size: 18px;
  font-weight: 750;
  margin-top: 2px;
}

.toolbar-right {
  display: flex;
  gap: 8px;
}

.editor-tabs {
  display: flex;
  gap: 2px;
  padding: 0 24px;
  border-bottom: 1px solid var(--border);
  background: var(--surface-1);
}

.tab-btn {
  padding: 10px 16px;
  border: none;
  border-bottom: 2px solid transparent;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  font-weight: 700;
  font-size: 13px;
  transition: color var(--transition-fast), border-color var(--transition-fast);
}

.tab-btn:hover {
  color: var(--text-primary);
}

.tab-btn.active {
  color: var(--brand-light);
  border-bottom-color: var(--brand);
}

.editor-content {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
}

.tab-panel {
  display: grid;
  gap: 28px;
  max-width: 880px;
}

.section-title {
  display: block;
  color: var(--text-primary);
  font-size: 15px;
  font-weight: 750;
  margin-bottom: 14px;
}

.section-desc {
  color: var(--text-tertiary);
  font-size: 12px;
  margin-bottom: 16px;
  line-height: 1.5;
}

.section-heading {
  display: flex;
  gap: 12px;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 14px;
}

.section-heading .section-title {
  margin-bottom: 0;
}

.preview-mode {
  display: inline-flex;
  align-items: center;
  min-height: 26px;
  padding: 4px 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
  color: var(--brand-light);
  font-size: 11px;
  font-weight: 800;
  text-transform: uppercase;
}

.renderer-preview-stage {
  position: relative;
  height: 360px;
  min-height: 360px;
  overflow: hidden;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background:
    linear-gradient(180deg, rgba(45,212,191,0.08), transparent 52%),
    var(--surface-0);
}

.sprite-preview {
  display: grid;
  place-items: end center;
  width: 100%;
  height: 100%;
  padding: 20px;
}

.sprite-preview img {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
  filter: drop-shadow(0 18px 28px rgba(0,0,0,0.34));
}

.form-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
}

.form-grid .full {
  grid-column: 1 / -1;
}

.form-field {
  display: grid;
  gap: 6px;
}

.form-field span {
  color: var(--text-secondary);
  font-size: 12px;
  font-weight: 700;
}

.form-field.invalid .input {
  border-color: var(--warning);
}

.field-warning {
  color: var(--warning);
  font-size: 11px;
  font-weight: 650;
  line-height: 1.35;
  overflow-wrap: anywhere;
}

.asset-diagnostics {
  display: grid;
  gap: 10px;
  margin-top: 14px;
  padding: 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-1);
}

.asset-diagnostics.warning {
  border-color: rgba(245,158,11,0.42);
}

.asset-diag-head,
.asset-diag-row {
  display: flex;
  gap: 12px;
  align-items: center;
  justify-content: space-between;
}

.asset-diag-head span {
  color: var(--text-secondary);
  font-size: 12px;
  font-weight: 800;
}

.asset-diag-head strong {
  color: var(--success);
  font-size: 12px;
}

.asset-diag-head strong.warning {
  color: var(--warning);
}

.asset-diag-list {
  display: grid;
  gap: 6px;
}

.asset-diag-row {
  min-height: 28px;
  padding: 6px 8px;
  border-radius: var(--radius-sm);
  background: var(--surface-2);
}

.asset-diag-row b {
  flex-shrink: 0;
  color: var(--text-secondary);
  font-size: 11px;
}

.asset-diag-row span {
  min-width: 0;
  color: var(--text-tertiary);
  font-size: 11px;
  overflow-wrap: anywhere;
  text-align: right;
}

.asset-diag-row.warning b,
.asset-diag-row.warning span {
  color: var(--warning);
}

.asset-diag-empty {
  color: var(--text-tertiary);
  font-size: 12px;
}

.trait-columns {
  display: grid;
  grid-template-columns: 1fr 280px;
  gap: 24px;
  align-items: start;
}

.trait-list {
  display: grid;
  gap: 14px;
}

.trait-item {
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 14px;
  background: var(--surface-1);
}

.trait-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.trait-header label {
  color: var(--text-primary);
  font-weight: 700;
  font-size: 13px;
}

.trait-val {
  color: var(--brand-light);
  font-family: var(--font-mono);
  font-size: 14px;
  font-weight: 800;
}

.trait-item input[type='range'] {
  width: 100%;
  accent-color: var(--brand);
}

.trait-desc {
  color: var(--text-tertiary);
  font-size: 11px;
  margin-top: 6px;
  line-height: 1.4;
}

.radar-chart {
  display: flex;
  justify-content: center;
  padding: 16px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-1);
}

.radar-svg {
  width: 260px;
  height: 260px;
}

.emotion-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: 14px;
}

.emotion-card {
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 14px;
  background: var(--surface-1);
}

.emotion-header {
  display: flex;
  gap: 10px;
  align-items: center;
  margin-bottom: 10px;
}

.emotion-icon {
  font-size: 18px;
  color: var(--brand-light);
}

.emotion-header strong {
  font-size: 13px;
}

.sprite-path-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: 12px;
}

.sprite-path-row {
  padding: 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-1);
}

.rel-list {
  display: grid;
  gap: 12px;
}

.rel-item {
  display: flex;
  gap: 12px;
  align-items: center;
  padding: 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-1);
}

.rel-avatar {
  width: 32px;
  height: 32px;
  border-radius: var(--radius-sm);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-weight: 800;
  font-size: 11px;
  flex-shrink: 0;
}

.rel-name {
  min-width: 100px;
  font-size: 13px;
  font-weight: 600;
}

.rel-item input[type='range'] {
  flex: 1;
  accent-color: var(--brand);
}

.rel-val {
  width: 36px;
  text-align: right;
  font-family: var(--font-mono);
  font-size: 13px;
  font-weight: 700;
  color: var(--brand-light);
}

.muted {
  color: var(--text-tertiary);
  font-size: 13px;
}

.knowledge-editor {
  display: grid;
  gap: 14px;
}

.knowledge-item {
  display: grid;
  grid-template-columns: 200px 1fr auto;
  gap: 12px;
  align-items: start;
  padding: 14px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-1);
}

.knowledge-item .full {
  grid-column: 2;
}

.btn-icon {
  width: 32px;
  height: 32px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
  color: var(--text-secondary);
  cursor: pointer;
  font-weight: 800;
}

.btn-icon.danger:hover {
  border-color: var(--danger);
  color: var(--danger);
}

.editor-empty {
  display: grid;
  place-items: center;
  padding: 40px;
}

.empty-state {
  text-align: center;
  max-width: 460px;
}

.empty-state h2 {
  color: var(--text-primary);
  font-size: 24px;
  margin-top: 16px;
}

.empty-state p {
  color: var(--text-tertiary);
  font-size: 13px;
  margin-top: 8px;
  line-height: 1.6;
}

.quick-stats {
  margin-top: 20px;
  color: var(--text-secondary);
  font-size: 13px;
}

.quick-stats strong {
  color: var(--brand-light);
  font-size: 18px;
}

.status-toast {
  position: fixed;
  bottom: 20px;
  left: 50%;
  transform: translateX(-50%);
  min-width: 300px;
  padding: 12px 18px;
  border: 1px solid rgba(45,212,191,0.36);
  border-radius: var(--radius);
  background: rgba(15,118,110,0.96);
  color: white;
  font-size: 13px;
  font-weight: 600;
  text-align: center;
  z-index: 100;
  box-shadow: var(--shadow-lg);
  cursor: pointer;
}

.status-toast.error {
  border-color: rgba(239,68,68,0.42);
  background: rgba(127,29,29,0.96);
}

.fade-enter-active, .fade-leave-active { transition: opacity 0.2s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }

@media (max-width: 860px) {
  .editor-workbench {
    grid-template-columns: 1fr;
  }
  .char-rail {
    max-height: 240px;
    border-right: none;
    border-bottom: 1px solid var(--border);
  }
  .form-grid {
    grid-template-columns: 1fr;
  }
  .trait-columns {
    grid-template-columns: 1fr;
  }
  .renderer-preview-stage {
    height: 300px;
    min-height: 300px;
  }
  .knowledge-item {
    grid-template-columns: 1fr;
  }
  .asset-diag-row {
    align-items: flex-start;
    flex-direction: column;
    gap: 4px;
  }
  .asset-diag-row span {
    text-align: left;
  }
}
</style>
