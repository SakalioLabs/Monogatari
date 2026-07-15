<template>
  <div class="character-workbench">
    <aside class="character-browser">
      <header class="browser-header">
        <div>
          <span class="eyebrow">{{ t('characters.editor.eyebrow', 'Cast authoring') }}</span>
          <h1>{{ t('characters.title', 'Characters') }}</h1>
          <p>{{ t('characters.editor.loaded-summary', '{visible} of {total}', { visible: filteredCharacterList.length, total: characterList.length }) }}</p>
        </div>
        <button class="icon-command primary" :title="t('characters.create', 'Create Character')" @click="requestCreate">
          <Plus :size="17" />
          <span class="sr-only">{{ t('characters.create', 'Create Character') }}</span>
        </button>
      </header>

      <label class="browser-search">
        <Search :size="15" />
        <input v-model="searchQuery" :placeholder="t('characters.search', 'Search characters...')" />
      </label>

      <div class="character-list">
        <button
          v-for="character in filteredCharacterList"
          :key="character.id"
          class="character-row"
          :class="{ selected: selectedId === character.id }"
          @click="requestSelect(character.id)"
        >
          <span class="avatar" :style="characterImage(character) ? undefined : { background: avatarColor(character.id) }">
            <img
              v-if="characterImage(character)"
              :src="characterImage(character) || ''"
              :alt="character.name"
              @error="markCharacterImageFailed(character.id)"
            />
            <template v-else>{{ initials(character.name) }}</template>
          </span>
          <span class="character-copy">
            <strong>{{ character.name }}</strong>
            <small>{{ character.description || t('characters.no-description', 'No description') }}</small>
          </span>
          <span class="emotion-chip">{{ emotionLabel(character.emotion || 'neutral') }}</span>
        </button>

        <div v-if="filteredCharacterList.length === 0" class="empty-list">
          <UserRound :size="26" />
          <strong>{{ t('characters.no-results', 'No characters found') }}</strong>
          <span>{{ searchQuery ? t('characters.editor.search-empty', 'Try another name or ID.') : t('characters.empty-copy', 'Create a character to begin building your project cast.') }}</span>
        </div>
      </div>

      <footer v-if="browserDraft" class="browser-draft-bar">
        <span><RotateCcw :size="13" />{{ t('characters.editor.browser-draft', 'Browser draft active') }}</span>
        <button @click="requestRestoreProject">{{ t('characters.editor.restore-project', 'Restore project') }}</button>
      </footer>
    </aside>

    <main v-if="editing" class="character-editor">
      <header class="editor-toolbar">
        <div class="editor-title">
          <span class="eyebrow">{{ isNew ? t('characters.editor.creating', 'Creating') : t('characters.editor.editing', 'Editing character') }}</span>
          <div><h2>{{ isNew ? t('characters.editor.new-title', 'New character') : form.name }}</h2><code>{{ form.id || t('characters.editor.unsaved-id', 'unsaved') }}</code></div>
          <span v-if="isDirty" class="dirty-chip">{{ t('characters.editor.dirty', 'Unsaved') }}</span>
        </div>
        <div class="toolbar-actions">
          <button class="btn btn-secondary btn-sm" :disabled="!form.id.trim()" @click="exportChar"><Download :size="14" />{{ t('characters.export', 'Export JSON') }}</button>
          <button class="btn btn-secondary btn-sm" @click="requestCancel"><X :size="14" />{{ t('common.cancel', 'Cancel') }}</button>
          <button class="btn btn-primary btn-sm" :disabled="saving || !canSave" @click="save">
            <LoaderCircle v-if="saving" class="spinner" :size="14" />
            <Save v-else :size="14" />
            {{ saving ? t('characters.editor.saving', 'Saving') : t('common.save', 'Save') }}
          </button>
        </div>
      </header>

      <nav class="editor-tabs" role="tablist" :aria-label="t('characters.editor.tabs-label', 'Character editor sections')">
        <button
          v-for="tab in tabs"
          :key="tab.key"
          role="tab"
          class="tab-button"
          :aria-selected="activeTab === tab.key"
          :class="{ active: activeTab === tab.key }"
          @click="activeTab = tab.key"
        ><component :is="tab.icon" :size="14" />{{ tab.label }}</button>
      </nav>

      <p v-if="validationMessage" class="validation-banner"><TriangleAlert :size="14" />{{ validationMessage }}</p>

      <div class="editor-scroll">
        <div v-if="activeTab === 'basic'" class="tab-panel basic-panel">
          <div class="basic-form">
            <section class="editor-section">
              <div class="section-heading-copy">
                <span class="section-title">{{ t('characters.editor.identity', 'Identity') }}</span>
                <p>{{ t('characters.editor.identity-copy', 'Define the name and stable context used throughout the project.') }}</p>
              </div>
              <div class="form-grid">
                <label class="form-field">
                  <span>{{ t('characters.editor.id', 'Character ID') }}</span>
                  <input v-model.trim="form.id" class="input mono" maxlength="128" :placeholder="t('characters.editor.id-placeholder', 'unique_id')" :disabled="!isNew" />
                </label>
                <label class="form-field">
                  <span>{{ t('characters.name', 'Name') }}</span>
                  <input v-model.trim="form.name" class="input" maxlength="256" :placeholder="t('characters.editor.name-placeholder', 'Character name')" />
                </label>
                <label class="form-field full">
                  <span>{{ t('characters.description', 'Description') }}</span>
                  <textarea v-model="form.description" class="input" rows="3" maxlength="2048" :placeholder="t('characters.editor.description-placeholder', 'Short description shown in character lists')"></textarea>
                </label>
                <label class="form-field full">
                  <span>{{ t('characters.background', 'Background') }}</span>
                  <textarea v-model="form.background" class="input" rows="7" maxlength="16384" :placeholder="t('characters.editor.background-placeholder', 'Character history, motivations, and important context')"></textarea>
                </label>
              </div>
            </section>

            <section class="editor-section">
              <div class="section-heading-copy">
                <span class="section-title">{{ t('characters.editor.voice', 'Voice and default state') }}</span>
                <p>{{ t('characters.editor.voice-copy', 'Set the baseline tone used by dialogue and model-generated responses.') }}</p>
              </div>
              <div class="form-grid">
                <label class="form-field">
                  <span>{{ t('characters.speech-style', 'Speech style') }}</span>
                  <input v-model="form.speech_style" class="input" maxlength="512" :placeholder="t('characters.editor.speech-placeholder', 'Cheerful, precise, softly spoken...')" />
                </label>
                <label class="form-field">
                  <span>{{ t('characters.editor.default-emotion', 'Default emotion') }}</span>
                  <select v-model="form.default_emotion" class="input">
                    <option v-for="emotion in emotionOptions" :key="emotion.value" :value="emotion.value">{{ emotion.label }}</option>
                  </select>
                </label>
              </div>
            </section>

            <section class="editor-section">
              <div class="section-heading-copy">
                <span class="section-title">{{ t('characters.editor.assets', 'Renderer assets') }}</span>
                <p>{{ t('characters.editor.assets-copy', 'Use project-relative paths; the preview selects Live2D, 3D, then sprite fallback.') }}</p>
              </div>
              <div class="form-grid">
                <label class="form-field" :class="{ invalid: assetFieldIssue('live2d_model_path') }">
                  <span>{{ t('characters.editor.live2d-path', 'Live2D model') }}</span>
                  <input v-model="form.live2d_model_path" class="input mono" :placeholder="t('characters.editor.live2d-placeholder', 'live2d/model.model3.json')" />
                  <small v-if="assetFieldIssue('live2d_model_path')" class="field-warning">{{ assetFieldIssue('live2d_model_path')?.message }}</small>
                </label>
                <label class="form-field" :class="{ invalid: assetFieldIssue('model_3d_path') }">
                  <span>{{ t('characters.editor.model3d-path', '3D model (GLB/GLTF)') }}</span>
                  <input v-model="form.model_3d_path" class="input mono" :placeholder="t('characters.editor.model3d-placeholder', 'models/character.glb')" />
                  <small v-if="assetFieldIssue('model_3d_path')" class="field-warning">{{ assetFieldIssue('model_3d_path')?.message }}</small>
                </label>
                <label class="form-field" :class="{ invalid: assetFieldIssue('portrait_path') }">
                  <span>{{ t('characters.editor.portrait-path', 'Portrait image') }}</span>
                  <input v-model="form.portrait_path" class="input mono" :placeholder="t('characters.editor.portrait-placeholder', 'assets/portraits/character.png')" />
                  <small v-if="assetFieldIssue('portrait_path')" class="field-warning">{{ assetFieldIssue('portrait_path')?.message }}</small>
                </label>
                <label class="form-field" :class="{ invalid: assetFieldIssue('sprite_path') }">
                  <span>{{ t('characters.editor.sprite-path', 'Fallback sprite') }}</span>
                  <input v-model="form.sprite_path" class="input mono" :placeholder="t('characters.editor.sprite-placeholder', 'assets/sprites/character.png')" />
                  <small v-if="assetFieldIssue('sprite_path')" class="field-warning">{{ assetFieldIssue('sprite_path')?.message }}</small>
                </label>
              </div>
            </section>
          </div>

          <aside class="preview-column">
            <section class="preview-section">
              <div class="preview-header">
                <div><span class="eyebrow">{{ t('characters.editor.preview', 'Renderer preview') }}</span><strong>{{ form.name || t('characters.editor.new-title', 'New character') }}</strong></div>
                <span class="preview-mode">{{ rendererPreviewMode }}</span>
              </div>
              <div class="renderer-preview-stage">
                <Live2DCanvas v-if="previewLive2dPath" :model-path="previewLive2dPath" :expression="previewExpression" motion="idle" @load-error="markPreviewRendererAssetFailed" />
                <CharacterModelView v-else-if="previewModel3dPath" :model-path="previewModel3dPath" :expression="previewExpression" motion="idle" @load-error="markPreviewRendererAssetFailed" />
                <div v-else-if="previewSpritePath" class="sprite-preview"><img :src="previewSpritePath" :alt="t('characters.editor.preview-alt', '{name} renderer preview', { name: form.name || t('characters.editor.new-title', 'New character') })" /></div>
                <CharacterModelView v-else :model-path="null" :expression="previewExpression" motion="idle" />
              </div>
            </section>

            <section class="asset-diagnostics" :class="{ warning: assetIssueCount > 0 }">
              <div class="asset-diag-head">
                <span>{{ t('characters.editor.asset-contract', 'Asset contract') }}</span>
                <strong :class="{ warning: assetIssueCount > 0 }">{{ rendererAssetSummary }}</strong>
              </div>
              <div v-if="assetDiagnostics.length > 0" class="asset-diag-list">
                <span v-for="diagnostic in assetDiagnostics" :key="diagnostic.key" class="asset-diag-row" :class="diagnostic.state">
                  <CheckCircle2 v-if="diagnostic.state === 'ready'" :size="13" />
                  <TriangleAlert v-else :size="13" />
                  <b>{{ diagnostic.label }}</b><span>{{ diagnostic.message }}</span>
                </span>
              </div>
              <div v-else class="asset-diag-empty"><Sparkles :size="14" />{{ t('characters.editor.generated-fallback', 'Generated 3D fallback ready') }}</div>
            </section>
          </aside>
        </div>

        <div v-else-if="activeTab === 'personality'" class="tab-panel">
          <section class="editor-section">
            <div class="section-heading-copy">
              <span class="section-title">{{ t('characters.editor.personality-title', 'Big Five personality') }}</span>
              <p>{{ t('characters.editor.personality-copy', 'These values shape model responses. Higher values amplify the trait in conversation.') }}</p>
            </div>
            <div class="trait-columns">
              <div class="trait-list">
                <div v-for="trait in personalityTraits" :key="trait.key" class="trait-item">
                  <div class="trait-header"><label>{{ trait.label }}</label><span class="trait-value">{{ form[trait.key].toFixed(2) }}</span></div>
                  <input v-model.number="form[trait.key]" type="range" min="0" max="1" step="0.05" />
                  <p>{{ trait.desc }}</p>
                </div>
              </div>
              <div class="radar-chart" :aria-label="t('characters.personality-chart', 'Personality chart')">
                <svg viewBox="0 0 260 260" class="radar-svg" role="img">
                  <polygon v-for="ring in 5" :key="ring" :points="radarRing(ring * 20)" fill="none" stroke="var(--border)" stroke-width="1" />
                  <polygon :points="radarPolygon" fill="var(--selection)" stroke="var(--brand)" stroke-width="2" />
                  <circle v-for="(point, index) in radarPoints" :key="index" :cx="point.x" :cy="point.y" r="4" fill="var(--brand)" />
                  <text v-for="(label, index) in radarLabels" :key="label" :x="radarLabelPos(index).x" :y="radarLabelPos(index).y" text-anchor="middle" fill="var(--text-secondary)" font-size="10" font-weight="700">{{ label }}</text>
                </svg>
              </div>
            </div>
          </section>
        </div>

        <div v-else-if="activeTab === 'emotions'" class="tab-panel">
          <section class="editor-section">
            <div class="section-heading-copy">
              <span class="section-title">{{ t('characters.editor.emotion-title', 'Emotion voice modifiers') }}</span>
              <p>{{ t('characters.editor.emotion-copy', 'Describe how each emotion changes the character’s phrasing and rhythm.') }}</p>
            </div>
            <div class="emotion-grid">
              <div v-for="emotion in emotionConfigs" :key="emotion.key" class="emotion-card">
                <div class="emotion-header"><span class="emotion-icon"><component :is="emotion.icon" :size="17" /></span><strong>{{ emotion.name }}</strong></div>
                <label class="form-field">
                  <span>{{ t('characters.editor.speech-modifier', 'Speech modifier') }}</span>
                  <input v-model="form.emotion_modifiers[emotion.key]" class="input" :placeholder="emotion.default_modifier" />
                </label>
              </div>
            </div>
          </section>

          <section class="editor-section">
            <div class="section-heading">
              <div class="section-heading-copy"><span class="section-title">{{ t('characters.editor.expression-sprites', 'Expression sprites') }}</span><p>{{ t('characters.editor.expression-copy', 'Override the fallback sprite for individual emotions.') }}</p></div>
              <button class="btn btn-secondary btn-sm" :disabled="!form.sprite_path.trim()" @click="fillDefaultSpritePaths"><Copy :size="14" />{{ t('characters.editor.copy-fallback', 'Copy fallback') }}</button>
            </div>
            <div class="sprite-path-grid">
              <label v-for="emotion in emotionOptions" :key="emotion.value" class="form-field sprite-path-row" :class="{ invalid: spritePathIssue(emotion.value) }">
                <span>{{ emotion.label }}</span>
                <input v-model="form.sprite_paths[emotion.value]" class="input mono" :placeholder="spritePlaceholder(emotion.value)" />
                <small v-if="spritePathIssue(emotion.value)" class="field-warning">{{ spritePathIssue(emotion.value)?.message }}</small>
              </label>
            </div>
          </section>
        </div>

        <div v-else-if="activeTab === 'relationships'" class="tab-panel">
          <section class="editor-section">
            <div class="section-heading-copy">
              <span class="section-title">{{ t('characters.editor.relationship-title', 'Default relationships') }}</span>
              <p>{{ t('characters.editor.relationship-copy', 'Set initial scores from -1.0 hostile to 1.0 close bond.') }}</p>
            </div>
            <div class="relationship-list">
              <div v-for="other in otherCharacters" :key="other.id" class="relationship-row">
                <span class="relationship-avatar" :style="{ background: avatarColor(other.id) }">{{ initials(other.name) }}</span>
                <span class="relationship-name"><strong>{{ other.name }}</strong><small>{{ other.id }}</small></span>
                <input v-model.number="form.relationships[other.id]" type="range" min="-1" max="1" step="0.1" />
                <span class="relationship-value">{{ (form.relationships[other.id] || 0).toFixed(1) }}</span>
              </div>
              <p v-if="otherCharacters.length === 0" class="muted">{{ t('characters.editor.no-relationships', 'Create another character to configure relationships.') }}</p>
            </div>
          </section>
        </div>

        <div v-else class="tab-panel">
          <section class="editor-section">
            <div class="section-heading-copy">
              <span class="section-title">{{ t('characters.editor.knowledge-title', 'Character knowledge') }}</span>
              <p>{{ t('characters.editor.knowledge-copy', 'Pin project knowledge and add private facts injected into conversations.') }}</p>
            </div>
            <label class="form-field full" :class="{ invalid: missingKnowledgeRefs.length > 0 }">
              <span>{{ t('characters.editor.pinned-refs', 'Pinned knowledge references') }}</span>
              <input v-model="form.knowledge_refs" class="input mono" :placeholder="t('characters.editor.pinned-placeholder', 'sakura_nature, sakura_art_knowledge')" />
              <small v-if="missingKnowledgeRefs.length > 0" class="field-warning">{{ t('characters.editor.missing-refs', 'Unknown knowledge: {ids}', { ids: missingKnowledgeRefs.join(', ') }) }}</small>
              <small v-else class="field-help">{{ t('characters.editor.pinned-help', 'Pinned entries are placed before search results in model context.') }}</small>
            </label>
            <div v-if="knowledgeIds.length > 0" class="knowledge-ref-options" :aria-label="t('characters.editor.available-refs', 'Available knowledge references')">
              <button v-for="id in knowledgeIds" :key="id" :class="{ selected: parsedKnowledgeRefs.includes(id) }" @click="toggleKnowledgeRef(id)"><BookOpen :size="11" />{{ id }}</button>
            </div>
          </section>

          <section class="editor-section">
            <div class="section-heading">
              <div class="section-heading-copy"><span class="section-title">{{ t('characters.editor.local-knowledge', 'Private knowledge entries') }}</span><p>{{ t('characters.editor.local-knowledge-copy', 'Keep character-specific facts alongside the character definition.') }}</p></div>
              <button class="btn btn-secondary btn-sm" @click="addKnowledge"><Plus :size="14" />{{ t('characters.editor.add-knowledge', 'Add entry') }}</button>
            </div>
            <div class="knowledge-editor">
              <div v-for="(entry, index) in form.knowledge_entries" :key="index" class="knowledge-item">
                <label class="form-field">
                  <span>{{ t('characters.editor.topic', 'Topic') }}</span>
                  <input v-model="entry.topic" class="input" :placeholder="t('characters.editor.topic-placeholder', 'My hometown')" />
                </label>
                <label class="form-field">
                  <span>{{ t('characters.editor.knowledge-content', 'Content') }}</span>
                  <textarea v-model="entry.content" class="input" rows="3" :placeholder="t('characters.editor.knowledge-placeholder', 'What this character knows')"></textarea>
                </label>
                <button class="icon-command danger" :title="t('characters.editor.remove-knowledge', 'Remove entry')" @click="removeKnowledge(index)"><Trash2 :size="15" /><span class="sr-only">{{ t('characters.editor.remove-knowledge', 'Remove entry') }}</span></button>
              </div>
              <div v-if="form.knowledge_entries.length === 0" class="inline-empty"><BookOpen :size="22" /><span>{{ t('characters.editor.no-local-knowledge', 'No private knowledge entries.') }}</span></div>
            </div>
          </section>
        </div>
      </div>
    </main>

    <main v-else class="editor-empty">
      <div class="empty-state">
        <span class="empty-icon"><UserRound :size="28" /></span>
        <h2>{{ t('characters.editor', 'Character Editor') }}</h2>
        <p>{{ t('characters.editor.empty-copy', 'Select a character to edit or create a new cast member.') }}</p>
        <span class="empty-count">{{ t('characters.loaded-count', '{count} characters loaded from project data.', { count: characterList.length }) }}</span>
        <button class="btn btn-primary btn-sm" @click="requestCreate"><Plus :size="14" />{{ t('characters.create', 'Create Character') }}</button>
      </div>
    </main>

    <Transition name="fade">
      <div v-if="pendingAction" class="modal-backdrop" @click.self="cancelPendingAction">
        <section class="confirm-dialog" role="dialog" aria-modal="true" :aria-label="confirmationTitle">
          <span class="confirm-icon"><TriangleAlert :size="18" /></span>
          <div><span class="eyebrow">{{ t('common.confirm', 'Confirm') }}</span><h2>{{ confirmationTitle }}</h2><p>{{ confirmationCopy }}</p></div>
          <div class="confirm-actions">
            <button class="btn btn-secondary btn-sm" :disabled="discarding" @click="cancelPendingAction"><X :size="14" />{{ t('characters.editor.keep-editing', 'Keep editing') }}</button>
            <button class="btn btn-danger btn-sm" :disabled="discarding" @click="confirmPendingAction">
              <LoaderCircle v-if="discarding" class="spinner" :size="14" />
              <RotateCcw v-else-if="pendingAction.kind === 'restore'" :size="14" />
              <Trash2 v-else :size="14" />
              {{ confirmationActionLabel }}
            </button>
          </div>
        </section>
      </div>
    </Transition>

    <Transition name="fade"><div v-if="statusMsg" class="status-toast" :class="{ error: !statusOk }" @click="statusMsg = null">{{ statusMsg }}</div></Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import {
  Angry,
  BookOpen,
  CheckCircle2,
  CircleAlert,
  Copy,
  Download,
  EyeOff,
  Frown,
  Heart,
  LoaderCircle,
  Plus,
  RotateCcw,
  Save,
  Search,
  SlidersHorizontal,
  Smile,
  Sparkles,
  Trash2,
  TriangleAlert,
  UserRound,
  UsersRound,
  X,
} from '@lucide/vue'
import { useRoute } from 'vue-router'
import Live2DCanvas from '../components/Live2DCanvas.vue'
import CharacterModelView from '../components/CharacterModelView.vue'
import { resolveAssetUrl } from '../lib/assets'
import {
  CHARACTER_EMOTIONS,
  buildStoryCharacter,
  characterFormFromStory,
  characterFormSnapshot,
  characterSpritePlaceholder,
  characterSummaryFromStory,
  createCharacterForm,
  fillMissingCharacterSpritePaths,
  filterCharacterSummaries,
  parseCharacterKnowledgeRefs,
  toggleCharacterKnowledgeRef,
  validateCharacterForm,
  type CharacterForm,
  type CharacterSummary,
  type CharacterTraitKey,
} from '../lib/characterAuthoring'
import { loadKnowledgeAuthoringCatalog } from '../lib/knowledgeContent'
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
import {
  loadBrowserCharacterDrafts,
  loadStoryCharacters,
  resetBrowserCharacterDrafts,
  saveBrowserCharacterDrafts,
  type StoryCharacterInfo,
} from '../lib/storyContent'

const { t } = useI18n()
const route = useRoute()

interface AssetDiagnostic {
  key: string
  label: string
  message: string
  state: 'ready' | 'warning'
}

type PendingAction =
  | { kind: 'select'; characterId: string }
  | { kind: 'create' }
  | { kind: 'cancel' }
  | { kind: 'restore' }

const emotions = CHARACTER_EMOTIONS

const emotionOptions = computed(() => [
  { value: 'neutral', label: t('characters.editor.emotion.neutral', 'Neutral') },
  { value: 'happy', label: t('characters.editor.emotion.happy', 'Happy') },
  { value: 'sad', label: t('characters.editor.emotion.sad', 'Sad') },
  { value: 'angry', label: t('characters.editor.emotion.angry', 'Angry') },
  { value: 'surprised', label: t('characters.editor.emotion.surprised', 'Surprised') },
  { value: 'love', label: t('characters.editor.emotion.love', 'Love') },
  { value: 'embarrassed', label: t('characters.editor.emotion.embarrassed', 'Embarrassed') },
  { value: 'thoughtful', label: t('characters.editor.emotion.thoughtful', 'Thoughtful') },
  { value: 'excited', label: t('characters.editor.emotion.excited', 'Excited') },
  { value: 'anxious', label: t('characters.editor.emotion.anxious', 'Anxious') },
])

function emotionLabel(value: string): string {
  return emotionOptions.value.find(option => option.value === value)?.label || value
}

const personalityTraits = computed<Array<{ key: CharacterTraitKey; label: string; desc: string }>>(() => [
  { key: 'openness', label: t('characters.trait.openness', 'Openness'), desc: t('characters.editor.trait.openness', 'Curiosity, creativity, and willingness to try new things.') },
  { key: 'conscientiousness', label: t('characters.trait.conscientiousness', 'Conscientiousness'), desc: t('characters.editor.trait.conscientiousness', 'Organization, discipline, and goal-oriented behavior.') },
  { key: 'extraversion', label: t('characters.trait.extraversion', 'Extraversion'), desc: t('characters.editor.trait.extraversion', 'Sociability, assertiveness, and energy from interactions.') },
  { key: 'agreeableness', label: t('characters.trait.agreeableness', 'Agreeableness'), desc: t('characters.editor.trait.agreeableness', 'Compassion, cooperation, and trust toward others.') },
  { key: 'neuroticism', label: t('characters.trait.neuroticism', 'Neuroticism'), desc: t('characters.editor.trait.neuroticism', 'Emotional sensitivity, anxiety, and mood variability.') },
])

const emotionConfigs = computed(() => [
  { name: t('characters.editor.emotion.happy', 'Happy'), key: 'happy', icon: Smile, default_modifier: t('characters.editor.modifier.happy', 'Cheerful, upbeat, uses exclamations') },
  { name: t('characters.editor.emotion.sad', 'Sad'), key: 'sad', icon: Frown, default_modifier: t('characters.editor.modifier.sad', 'Quiet, hesitant, trailing off') },
  { name: t('characters.editor.emotion.angry', 'Angry'), key: 'angry', icon: Angry, default_modifier: t('characters.editor.modifier.angry', 'Sharp, direct, shorter sentences') },
  { name: t('characters.editor.emotion.surprised', 'Surprised'), key: 'surprised', icon: CircleAlert, default_modifier: t('characters.editor.modifier.surprised', 'Exclamatory, questioning, disbelieving') },
  { name: t('characters.editor.emotion.love', 'Love'), key: 'love', icon: Heart, default_modifier: t('characters.editor.modifier.love', 'Warm, tender, softer tone') },
  { name: t('characters.editor.emotion.embarrassed', 'Embarrassed'), key: 'embarrassed', icon: EyeOff, default_modifier: t('characters.editor.modifier.embarrassed', 'Stammering, deflecting, nervous laughter') },
])

const tabs = computed(() => [
  { key: 'basic', label: t('characters.basic', 'Basic Info'), icon: UserRound },
  { key: 'personality', label: t('characters.personality', 'Personality'), icon: SlidersHorizontal },
  { key: 'emotions', label: t('characters.emotions', 'Emotions'), icon: Smile },
  { key: 'relationships', label: t('characters.relationships', 'Relationships'), icon: UsersRound },
  { key: 'knowledge', label: t('characters.knowledge', 'Knowledge'), icon: BookOpen },
])

const characterList = ref<CharacterSummary[]>([])
const browserCharacters = ref<StoryCharacterInfo[]>([])
const searchQuery = ref('')
const selectedId = ref<string | null>(null)
const editing = ref(false)
const isNew = ref(false)
const saving = ref(false)
const activeTab = ref('basic')
const statusMsg = ref<string | null>(null)
const statusOk = ref(true)
const browserDraft = ref(false)
const knowledgeIds = ref<string[]>([])
const knowledgeCatalogLoaded = ref(false)
const failedCharacterImages = ref<Record<string, true>>({})
const baselineSnapshot = ref('')
const pendingAction = ref<PendingAction | null>(null)
const discarding = ref(false)

const form = reactive<CharacterForm>(createCharacterForm())
const previewFailedRendererAssets = ref<Record<string, true>>({})

const filteredCharacterList = computed(() => filterCharacterSummaries(characterList.value, searchQuery.value))

const otherCharacters = computed(() =>
  characterList.value.filter(c => c.id !== form.id)
)

const parsedKnowledgeRefs = computed(() => parseCharacterKnowledgeRefs(form.knowledge_refs))
const validationIssue = computed(() => validateCharacterForm(form, {
  isNew: isNew.value,
  existingCharacterIds: characterList.value.map(({ id }) => id),
  knownKnowledgeIds: knowledgeCatalogLoaded.value ? knowledgeIds.value : null,
}))
const missingKnowledgeRefs = computed(() => validationIssue.value?.code === 'unknown_knowledge'
  ? validationIssue.value.knowledge_refs
  : [])

const validationMessage = computed(() => {
  if (validationIssue.value?.code === 'required') return t('characters.editor.validation.required', 'Character ID and name are required.')
  if (validationIssue.value?.code === 'invalid_id') return t('characters.editor.validation.id', 'ID can use only letters, numbers, underscores, or hyphens.')
  if (validationIssue.value?.code === 'duplicate_id') return t('characters.editor.validation.duplicate', 'This character ID already exists.')
  if (validationIssue.value?.code === 'unknown_knowledge') return t('characters.editor.validation.knowledge', 'Remove unknown knowledge references before saving.')
  if (validationIssue.value?.code === 'invalid_private_knowledge') return t('characters.editor.validation.local-knowledge', 'Each private knowledge entry needs a topic and content.')
  return ''
})
const canSave = computed(() => !validationMessage.value)
const isDirty = computed(() => editing.value
  && baselineSnapshot.value.length > 0
  && baselineSnapshot.value !== characterFormSnapshot(form))

const assetDiagnostics = computed<AssetDiagnostic[]>(() => {
  const diagnostics: AssetDiagnostic[] = []

  for (const spec of rendererAssetSpecs) {
    const path = form[spec.key]
    const message = localizedAssetValidationMessage(path, spec.extensions)
    if (message) {
      diagnostics.push({ key: spec.key, label: assetLabel(spec.key), message, state: 'warning' })
    } else if (path.trim()) {
      diagnostics.push({ key: spec.key, label: assetLabel(spec.key), message: t('characters.editor.asset-ready', 'Ready'), state: 'ready' })
    }
  }

  for (const [emotion, path] of Object.entries(cleanRendererPathMap(form.sprite_paths))) {
    const key = `sprite_paths.${emotion}`
    const message = localizedAssetValidationMessage(path, imageAssetExtensions)
    diagnostics.push({
      key,
      label: t('characters.editor.expression-label', '{emotion} sprite', { emotion }),
      message: message || t('characters.editor.asset-ready', 'Ready'),
      state: message ? 'warning' : 'ready',
    })
  }

  return diagnostics
})

const assetIssueCount = computed(() =>
  assetDiagnostics.value.filter(diag => diag.state === 'warning').length
)

const rendererAssetSummary = computed(() => {
  if (assetIssueCount.value > 0) return t('characters.editor.asset-warnings', '{count} warnings', { count: assetIssueCount.value })
  const declaredCount = assetDiagnostics.value.length
  return declaredCount > 0
    ? t('characters.editor.asset-ready-count', '{count} ready', { count: declaredCount })
    : t('characters.editor.fallback-ready', 'Fallback ready')
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
  if (rendererPreviewAsset.value.mode === 'sprite') return t('characters.editor.mode.sprite', 'Sprite')
  return t('characters.editor.mode.generated', 'Generated 3D')
})

const confirmationTitle = computed(() => pendingAction.value?.kind === 'restore'
  ? t('characters.editor.restore-title', 'Restore project characters?')
  : t('characters.editor.discard-title', 'Discard unsaved changes?'))
const confirmationCopy = computed(() => pendingAction.value?.kind === 'restore'
  ? t('characters.editor.restore-copy', 'Browser character drafts will be removed and project files loaded again.')
  : t('characters.editor.discard-copy', 'Your changes to this character have not been saved.'))
const confirmationActionLabel = computed(() => pendingAction.value?.kind === 'restore'
  ? t('characters.editor.restore-project', 'Restore project')
  : t('characters.editor.discard', 'Discard changes'))

function markPreviewRendererAssetFailed(payload: { path: string | null; message: string }) {
  const path = payload.path?.trim()
  if (!path) return
  previewFailedRendererAssets.value = { ...previewFailedRendererAssets.value, [path]: true }
}

function assetLabel(key: RendererAssetSpec['key']): string {
  const labels: Record<RendererAssetSpec['key'], string> = {
    live2d_model_path: 'Live2D',
    model_3d_path: '3D',
    portrait_path: t('characters.editor.asset.portrait', 'Portrait'),
    sprite_path: t('characters.editor.asset.sprite', 'Fallback sprite'),
  }
  return labels[key]
}

function localizedAssetValidationMessage(path: string, extensions: string[]): string | null {
  const message = rendererAssetValidationMessage(path, extensions)
  if (!message) return null
  if (message.startsWith('Expected ')) {
    return t('characters.editor.asset.expected', 'Expected {extensions}', { extensions: extensions.join(', ') })
  }
  const messages: Record<string, string> = {
    'Absolute paths are not portable': t('characters.editor.asset.absolute', 'Absolute paths are not portable.'),
    'Use a project-relative path': t('characters.editor.asset.relative', 'Use a project-relative path.'),
    'Parent traversal is not allowed': t('characters.editor.asset.traversal', 'Parent traversal is not allowed.'),
    'Path segments must be portable': t('characters.editor.asset.portable', 'Path segments must be portable.'),
  }
  return messages[message] || message
}

watch(
  () => [
    form.live2d_model_path,
    form.model_3d_path,
    form.portrait_path,
    form.sprite_path,
    JSON.stringify(cleanRendererPathMap(form.sprite_paths)),
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
  const lightness = 30 + (Array.from(id).reduce((sum, char) => sum + char.charCodeAt(0), 0) % 5) * 8
  return `hsl(0, 0%, ${lightness}%)`
}

function characterImage(character: CharacterSummary): string | null {
  if (failedCharacterImages.value[character.id]) return null
  return resolveAssetUrl(character.portrait_path || character.sprite_path)
}

function markCharacterImageFailed(characterId: string) {
  failedCharacterImages.value = { ...failedCharacterImages.value, [characterId]: true }
}

function initials(name: string): string {
  return name.trim().slice(0, 2).toUpperCase() || '??'
}

function spritePlaceholder(emotion: string): string {
  return characterSpritePlaceholder(form.id, emotion, form.sprite_path)
}

function fillDefaultSpritePaths() {
  form.sprite_paths = fillMissingCharacterSpritePaths(form.sprite_paths, form.sprite_path, emotions)
}

function assetFieldIssue(key: RendererAssetSpec['key']): AssetDiagnostic | undefined {
  return assetIssueMap.value[key]
}

function spritePathIssue(emotion: string): AssetDiagnostic | undefined {
  return assetIssueMap.value[`sprite_paths.${emotion}`]
}

function resetForm() {
  Object.assign(form, createCharacterForm())
  previewFailedRendererAssets.value = {}
}

function createNew() {
  resetForm()
  selectedId.value = null
  isNew.value = true
  editing.value = true
  activeTab.value = 'basic'
  setBaseline()
}

function closeEditor() {
  editing.value = false
  isNew.value = false
  selectedId.value = null
  baselineSnapshot.value = ''
}

async function loadList() {
  try {
    const characters = await loadStoryCharacters()
    browserCharacters.value = characters
    browserDraft.value = !hasTauriRuntime() && loadBrowserCharacterDrafts() !== null
    characterList.value = characters.map(characterSummaryFromStory)
  } catch (e) {
    notify('error', t('characters.editor.notice.load-failed', 'Characters could not be loaded: {error}', { error: String(e) }))
  }
}

async function loadKnowledgeIds() {
  try {
    const catalog = await loadKnowledgeAuthoringCatalog()
    knowledgeIds.value = catalog.entries.map(entry => entry.id).sort()
    knowledgeCatalogLoaded.value = true
  } catch {
    knowledgeIds.value = []
    knowledgeCatalogLoaded.value = false
  }
}

async function selectChar(id: string) {
  try {
    const character = hasTauriRuntime()
      ? await invokeCommand<StoryCharacterInfo>('get_character', { characterId: id })
      : browserCharacters.value.find(character => character.id === id)
    if (!character) return
    resetForm()
    selectedId.value = id
    isNew.value = false
    Object.assign(form, characterFormFromStory(character))
    editing.value = true
    activeTab.value = 'basic'
    setBaseline()
  } catch (e) {
    notify('error', t('characters.editor.notice.character-failed', 'Character could not be loaded: {error}', { error: String(e) }))
  }
}

async function save() {
  if (!canSave.value) {
    notify('error', validationMessage.value)
    return
  }
  saving.value = true
  try {
    const character = buildStoryCharacter(form)
    if (hasTauriRuntime()) {
      await invokeCommand('create_character', {
        input: { ...character, default_emotion: character.emotion },
      })
    } else {
      const characters = browserCharacters.value.filter(item => item.id !== character.id)
      characters.push(character)
      saveBrowserCharacterDrafts(characters)
      browserDraft.value = true
    }
    isNew.value = false
    selectedId.value = character.id
    await loadList()
    await selectChar(character.id)
    notify('success', assetIssueCount.value > 0
      ? t('characters.editor.notice.saved-warning', 'Character "{name}" saved with {count} asset warnings.', { name: character.name, count: assetIssueCount.value })
      : t('characters.editor.notice.saved', 'Character "{name}" saved.', { name: character.name }))
  } catch (e) {
    notify('error', t('characters.editor.notice.save-failed', 'Character could not be saved: {error}', { error: String(e) }))
  } finally {
    saving.value = false
  }
}

function exportChar() {
  const data = buildStoryCharacter(form)
  const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = (form.id || 'character') + '.json'
  a.click()
  URL.revokeObjectURL(url)
  notify('success', assetIssueCount.value > 0
    ? t('characters.editor.notice.export-warning', 'JSON exported with {count} asset warnings.', { count: assetIssueCount.value })
    : t('characters.editor.notice.exported', 'Character JSON exported.'))
}

function addKnowledge() {
  form.knowledge_entries.push({ topic: '', content: '' })
}

function removeKnowledge(i: number) {
  form.knowledge_entries.splice(i, 1)
}

function toggleKnowledgeRef(id: string) {
  form.knowledge_refs = toggleCharacterKnowledgeRef(form.knowledge_refs, id)
}

function setBaseline() {
  baselineSnapshot.value = characterFormSnapshot(form)
}

function requestSelect(characterId: string) {
  if (selectedId.value === characterId && editing.value) return
  if (isDirty.value) pendingAction.value = { kind: 'select', characterId }
  else void selectChar(characterId)
}

function requestCreate() {
  if (isDirty.value) pendingAction.value = { kind: 'create' }
  else createNew()
}

function requestCancel() {
  if (isDirty.value) pendingAction.value = { kind: 'cancel' }
  else closeEditor()
}

function requestRestoreProject() {
  pendingAction.value = { kind: 'restore' }
}

function cancelPendingAction() {
  if (!discarding.value) pendingAction.value = null
}

async function confirmPendingAction() {
  const action = pendingAction.value
  if (!action || discarding.value) return
  discarding.value = true
  try {
    pendingAction.value = null
    if (action.kind === 'select') await selectChar(action.characterId)
    if (action.kind === 'create') createNew()
    if (action.kind === 'cancel') closeEditor()
    if (action.kind === 'restore') {
      const previousId = selectedId.value
      resetBrowserCharacterDrafts()
      browserDraft.value = false
      await loadList()
      if (previousId && characterList.value.some(character => character.id === previousId)) await selectChar(previousId)
      else if (characterList.value.length > 0) await selectChar(characterList.value[0].id)
      else createNew()
      notify('success', t('characters.editor.notice.restored', 'Project characters restored.'))
    }
  } catch (error) {
    notify('error', t('characters.editor.notice.restore-failed', 'Project characters could not be restored: {error}', { error: String(error) }))
  } finally {
    discarding.value = false
  }
}

function notify(type: 'success' | 'error', message: string) {
  statusOk.value = type === 'success'
  statusMsg.value = message
}

onMounted(async () => {
  await Promise.all([loadList(), loadKnowledgeIds()])
  if (route.query.create === '1') {
    createNew()
    return
  }
  const requestedCharacter = typeof route.query.character === 'string' ? route.query.character : ''
  if (requestedCharacter && characterList.value.some(character => character.id === requestedCharacter)) {
    await selectChar(requestedCharacter)
    return
  }
  if (characterList.value.length > 0) await selectChar(characterList.value[0].id)
  else createNew()
})
</script>

<style scoped>
.character-workbench { display: grid; grid-template-columns: 276px minmax(0, 1fr); height: calc(100svh - 56px); min-height: 0; overflow: hidden; background: var(--surface-0); }
.character-browser { display: flex; min-width: 0; min-height: 0; flex-direction: column; border-right: 1px solid var(--border); background: var(--surface-1); }
.browser-header { display: flex; min-height: 86px; align-items: flex-start; justify-content: space-between; gap: 12px; padding: 17px 14px 12px; }
.browser-header h1 { margin: 3px 0 0; color: var(--text-primary); font-size: 20px; line-height: 1.15; }
.browser-header p { margin: 4px 0 0; color: var(--text-tertiary); font-size: 10px; }
.eyebrow { display: block; color: var(--text-tertiary); font-size: 10px; font-weight: 800; text-transform: uppercase; }
.icon-command { display: inline-grid; width: 34px; height: 34px; flex: 0 0 34px; place-items: center; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--surface-2); color: var(--text-secondary); cursor: pointer; }
.icon-command:hover { border-color: var(--border-strong); color: var(--text-primary); }
.icon-command.primary { border-color: var(--brand); background: var(--brand); color: var(--surface-0); }
.icon-command.danger:hover { border-color: var(--danger); color: var(--danger); }
.browser-search { display: flex; height: 34px; align-items: center; gap: 7px; margin: 0 10px 8px; padding: 0 9px; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--surface-0); color: var(--text-tertiary); }
.browser-search:focus-within { border-color: var(--brand); color: var(--brand-light); }
.browser-search input { min-width: 0; flex: 1; border: 0; outline: 0; background: transparent; color: var(--text-primary); font: inherit; font-size: 11px; }
.character-list { flex: 1; min-height: 0; overflow-y: auto; padding: 2px 8px 10px; }
.character-row { display: grid; width: 100%; min-height: 62px; grid-template-columns: 38px minmax(0, 1fr) auto; align-items: center; gap: 9px; margin-bottom: 4px; padding: 8px; border: 1px solid transparent; border-radius: var(--radius); background: transparent; color: inherit; cursor: pointer; font: inherit; text-align: left; }
.character-row:hover { background: var(--surface-2); }
.character-row.selected { border-color: color-mix(in srgb, var(--brand) 65%, var(--border)); background: var(--selection); }
.avatar { display: inline-flex; width: 38px; height: 38px; flex: 0 0 38px; align-items: center; justify-content: center; overflow: hidden; border-radius: var(--radius-sm); color: white; font-size: 11px; font-weight: 850; }
.avatar img { width: 100%; height: 100%; object-fit: cover; }
.character-copy { display: grid; min-width: 0; gap: 3px; }
.character-copy strong { overflow: hidden; color: var(--text-primary); font-size: 12px; text-overflow: ellipsis; white-space: nowrap; }
.character-copy small { overflow: hidden; color: var(--text-tertiary); font-size: 9px; text-overflow: ellipsis; white-space: nowrap; }
.emotion-chip { max-width: 68px; overflow: hidden; padding: 3px 6px; border: 1px solid var(--border); border-radius: 999px; color: var(--text-tertiary); font-size: 8px; text-overflow: ellipsis; white-space: nowrap; }
.empty-list { display: grid; min-height: 220px; place-items: center; align-content: center; gap: 7px; padding: 20px; color: var(--text-tertiary); text-align: center; }
.empty-list strong { color: var(--text-secondary); font-size: 12px; }
.empty-list span { font-size: 10px; line-height: 1.5; }
.browser-draft-bar { display: flex; min-height: 44px; align-items: center; justify-content: space-between; gap: 8px; padding: 7px 10px; border-top: 1px solid var(--border); background: var(--surface-2); }
.browser-draft-bar span { display: flex; min-width: 0; align-items: center; gap: 5px; color: var(--warning); font-size: 9px; }
.browser-draft-bar button { flex: 0 0 auto; border: 0; background: transparent; color: var(--brand-light); cursor: pointer; font: inherit; font-size: 9px; font-weight: 750; }
.character-editor { display: flex; min-width: 0; min-height: 0; flex-direction: column; overflow: hidden; }
.editor-toolbar { display: flex; min-height: 66px; align-items: center; justify-content: space-between; gap: 16px; padding: 10px 18px; border-bottom: 1px solid var(--border); background: var(--surface-1); }
.editor-title { display: flex; min-width: 0; align-items: center; gap: 10px; }
.editor-title > div { display: flex; min-width: 0; align-items: baseline; gap: 8px; }
.editor-title h2 { overflow: hidden; margin: 0; color: var(--text-primary); font-size: 17px; text-overflow: ellipsis; white-space: nowrap; }
.editor-title code { overflow: hidden; max-width: 180px; color: var(--text-tertiary); font-size: 9px; text-overflow: ellipsis; white-space: nowrap; }
.editor-title > .eyebrow { display: none; }
.dirty-chip { flex: 0 0 auto; padding: 3px 7px; border: 1px solid color-mix(in srgb, var(--warning) 45%, var(--border)); border-radius: 999px; color: var(--warning); font-size: 8px; font-weight: 800; text-transform: uppercase; }
.toolbar-actions { display: flex; flex: 0 0 auto; gap: 7px; }
.toolbar-actions .btn { display: inline-flex; align-items: center; justify-content: center; gap: 6px; }
.editor-tabs { display: flex; min-height: 42px; flex: 0 0 auto; gap: 2px; overflow-x: auto; padding: 0 18px; border-bottom: 1px solid var(--border); background: var(--surface-1); scrollbar-width: none; }
.editor-tabs::-webkit-scrollbar { display: none; }
.tab-button { display: inline-flex; min-width: max-content; align-items: center; gap: 6px; padding: 10px 12px 8px; border: 0; border-bottom: 2px solid transparent; background: transparent; color: var(--text-secondary); cursor: pointer; font: inherit; font-size: 10px; font-weight: 750; }
.tab-button:hover { color: var(--text-primary); }
.tab-button.active { border-bottom-color: var(--brand); color: var(--brand-light); }
.validation-banner { display: flex; min-height: 34px; align-items: center; gap: 7px; margin: 0; padding: 7px 18px; border-bottom: 1px solid color-mix(in srgb, var(--warning) 35%, var(--border)); background: color-mix(in srgb, var(--warning) 8%, transparent); color: var(--warning); font-size: 10px; }
.editor-scroll { flex: 1; min-height: 0; overflow-y: auto; padding: 22px 24px 44px; }
.tab-panel { display: grid; width: min(1120px, 100%); gap: 26px; margin: 0 auto; }
.basic-panel { grid-template-columns: minmax(0, 1fr) 350px; align-items: start; }
.basic-form { display: grid; gap: 28px; min-width: 0; }
.editor-section { min-width: 0; padding-bottom: 26px; border-bottom: 1px solid var(--border); }
.editor-section:last-child { border-bottom: 0; }
.section-heading { display: flex; align-items: flex-start; justify-content: space-between; gap: 16px; margin-bottom: 14px; }
.section-heading-copy { margin-bottom: 14px; }
.section-heading .section-heading-copy { margin-bottom: 0; }
.section-title { display: block; color: var(--text-primary); font-size: 14px; font-weight: 800; }
.section-heading-copy p { max-width: 680px; margin: 5px 0 0; color: var(--text-tertiary); font-size: 10px; line-height: 1.55; }
.form-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 13px; }
.form-grid .full { grid-column: 1 / -1; }
.form-field { display: grid; min-width: 0; gap: 6px; }
.form-field > span { color: var(--text-secondary); font-size: 10px; font-weight: 750; }
.form-field.invalid .input { border-color: var(--warning); }
.field-warning, .field-help { color: var(--warning); font-size: 9px; line-height: 1.4; overflow-wrap: anywhere; }
.field-help { color: var(--text-tertiary); }
.mono { font-family: var(--font-mono); }
.preview-column { position: sticky; top: 0; display: grid; gap: 13px; min-width: 0; }
.preview-section { overflow: hidden; border: 1px solid var(--border); border-radius: var(--radius); background: var(--surface-1); }
.preview-header { display: flex; min-height: 54px; align-items: center; justify-content: space-between; gap: 10px; padding: 10px 12px; border-bottom: 1px solid var(--border); }
.preview-header strong { display: block; max-width: 200px; margin-top: 3px; overflow: hidden; color: var(--text-primary); font-size: 12px; text-overflow: ellipsis; white-space: nowrap; }
.preview-mode { flex: 0 0 auto; padding: 4px 7px; border: 1px solid var(--border); border-radius: var(--radius-sm); color: var(--brand-light); font-size: 8px; font-weight: 850; text-transform: uppercase; }
.renderer-preview-stage { position: relative; width: 100%; height: 390px; overflow: hidden; background: var(--surface-0); }
.sprite-preview { display: grid; width: 100%; height: 100%; place-items: end center; padding: 18px; }
.sprite-preview img { max-width: 100%; max-height: 100%; object-fit: contain; filter: drop-shadow(0 16px 24px rgba(0, 0, 0, 0.32)); }
.asset-diagnostics { display: grid; gap: 8px; padding: 11px 12px; border: 1px solid var(--border); border-radius: var(--radius); background: var(--surface-1); }
.asset-diagnostics.warning { border-color: color-mix(in srgb, var(--warning) 45%, var(--border)); }
.asset-diag-head { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
.asset-diag-head span, .asset-diag-head strong { font-size: 9px; font-weight: 800; }
.asset-diag-head span { color: var(--text-secondary); }
.asset-diag-head strong { color: var(--success); }
.asset-diag-head strong.warning { color: var(--warning); }
.asset-diag-list { display: grid; gap: 5px; }
.asset-diag-row { display: grid; min-height: 27px; grid-template-columns: 14px auto minmax(0, 1fr); align-items: center; gap: 6px; padding: 5px 7px; border-radius: var(--radius-sm); background: var(--surface-2); color: var(--success); }
.asset-diag-row b { color: var(--text-secondary); font-size: 8px; }
.asset-diag-row span { overflow: hidden; color: var(--text-tertiary); font-size: 8px; text-align: right; text-overflow: ellipsis; white-space: nowrap; }
.asset-diag-row.warning, .asset-diag-row.warning b, .asset-diag-row.warning span { color: var(--warning); }
.asset-diag-empty { display: flex; align-items: center; gap: 6px; color: var(--text-tertiary); font-size: 9px; }
.trait-columns { display: grid; grid-template-columns: minmax(0, 1fr) 300px; align-items: start; gap: 22px; }
.trait-list { display: grid; gap: 9px; }
.trait-item { padding: 12px; border: 1px solid var(--border); border-radius: var(--radius); background: var(--surface-1); }
.trait-header { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
.trait-header label { color: var(--text-primary); font-size: 11px; font-weight: 750; }
.trait-value { color: var(--brand-light); font-family: var(--font-mono); font-size: 11px; font-weight: 800; }
.trait-item input[type='range'] { width: 100%; margin-top: 7px; accent-color: var(--brand); }
.trait-item p { margin: 5px 0 0; color: var(--text-tertiary); font-size: 9px; line-height: 1.45; }
.radar-chart { display: flex; justify-content: center; padding: 16px; border: 1px solid var(--border); border-radius: var(--radius); background: var(--surface-1); }
.radar-svg { width: 260px; height: 260px; }
.emotion-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 10px; }
.emotion-card { padding: 12px; border: 1px solid var(--border); border-radius: var(--radius); background: var(--surface-1); }
.emotion-header { display: flex; align-items: center; gap: 8px; margin-bottom: 9px; }
.emotion-icon { display: grid; width: 29px; height: 29px; place-items: center; border-radius: var(--radius-sm); background: var(--surface-2); color: var(--brand-light); }
.emotion-header strong { color: var(--text-primary); font-size: 11px; }
.sprite-path-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 9px; }
.sprite-path-row { padding: 10px; border: 1px solid var(--border); border-radius: var(--radius); background: var(--surface-1); }
.relationship-list { display: grid; gap: 7px; }
.relationship-row { display: grid; min-height: 54px; grid-template-columns: 32px 150px minmax(100px, 1fr) 38px; align-items: center; gap: 10px; padding: 8px 10px; border: 1px solid var(--border); border-radius: var(--radius); background: var(--surface-1); }
.relationship-avatar { display: inline-flex; width: 32px; height: 32px; align-items: center; justify-content: center; border-radius: var(--radius-sm); color: white; font-size: 9px; font-weight: 800; }
.relationship-name { display: grid; min-width: 0; }
.relationship-name strong { overflow: hidden; color: var(--text-primary); font-size: 10px; text-overflow: ellipsis; white-space: nowrap; }
.relationship-name small { color: var(--text-tertiary); font-family: var(--font-mono); font-size: 8px; }
.relationship-row input { width: 100%; accent-color: var(--brand); }
.relationship-value { color: var(--brand-light); font-family: var(--font-mono); font-size: 10px; font-weight: 800; text-align: right; }
.muted { margin: 0; color: var(--text-tertiary); font-size: 10px; }
.knowledge-ref-options { display: flex; max-height: 150px; flex-wrap: wrap; gap: 5px; overflow-y: auto; margin-top: 11px; padding: 9px; border: 1px solid var(--border); border-radius: var(--radius); background: var(--surface-1); }
.knowledge-ref-options button { display: inline-flex; align-items: center; gap: 4px; padding: 4px 7px; border: 1px solid var(--border); border-radius: 999px; background: transparent; color: var(--text-tertiary); cursor: pointer; font: inherit; font-family: var(--font-mono); font-size: 8px; }
.knowledge-ref-options button:hover, .knowledge-ref-options button.selected { border-color: var(--brand); background: var(--selection); color: var(--brand-light); }
.knowledge-editor { display: grid; gap: 9px; }
.knowledge-item { display: grid; grid-template-columns: 180px minmax(0, 1fr) 34px; align-items: start; gap: 10px; padding: 11px; border: 1px solid var(--border); border-radius: var(--radius); background: var(--surface-1); }
.inline-empty { display: flex; min-height: 100px; align-items: center; justify-content: center; gap: 8px; border: 1px dashed var(--border); border-radius: var(--radius); color: var(--text-tertiary); font-size: 10px; }
.editor-empty { display: grid; min-height: 0; place-items: center; padding: 36px; }
.empty-state { display: grid; max-width: 430px; justify-items: center; gap: 9px; text-align: center; }
.empty-icon { display: grid; width: 52px; height: 52px; place-items: center; border: 1px solid var(--border); border-radius: var(--radius); background: var(--surface-1); color: var(--brand-light); }
.empty-state h2 { margin: 4px 0 0; color: var(--text-primary); font-size: 21px; }
.empty-state p { margin: 0; color: var(--text-tertiary); font-size: 11px; line-height: 1.6; }
.empty-count { color: var(--text-secondary); font-size: 9px; }
.empty-state .btn { display: inline-flex; align-items: center; gap: 6px; margin-top: 4px; }
.modal-backdrop { position: fixed; z-index: 100; inset: 0; display: grid; place-items: center; padding: 20px; background: rgba(4, 8, 15, 0.68); backdrop-filter: blur(4px); }
.confirm-dialog { display: grid; width: min(430px, 100%); grid-template-columns: 38px minmax(0, 1fr); gap: 12px; padding: 18px; border: 1px solid var(--border); border-radius: var(--radius); background: var(--surface-1); box-shadow: var(--shadow-lg); }
.confirm-icon { display: grid; width: 38px; height: 38px; place-items: center; border-radius: var(--radius-sm); background: color-mix(in srgb, var(--warning) 13%, transparent); color: var(--warning); }
.confirm-dialog h2 { margin: 5px 0 0; color: var(--text-primary); font-size: 14px; }
.confirm-dialog p { margin: 6px 0 0; color: var(--text-secondary); font-size: 10px; line-height: 1.5; }
.confirm-actions { grid-column: 1 / -1; display: flex; justify-content: flex-end; gap: 7px; margin-top: 6px; padding-top: 13px; border-top: 1px solid var(--border); }
.confirm-actions .btn { display: inline-flex; align-items: center; gap: 6px; }
.status-toast { position: fixed; z-index: 110; right: 20px; bottom: 20px; max-width: min(460px, calc(100vw - 32px)); padding: 10px 12px; border: 1px solid color-mix(in srgb, var(--success) 45%, var(--border)); border-radius: var(--radius); background: color-mix(in srgb, var(--success) 78%, var(--surface-1)); color: white; box-shadow: var(--shadow-lg); cursor: pointer; font-size: 10px; }
.status-toast.error { border-color: color-mix(in srgb, var(--danger) 45%, var(--border)); background: color-mix(in srgb, var(--danger) 72%, var(--surface-1)); }
.spinner { animation: spin 0.8s linear infinite; }
.sr-only { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; }
.fade-enter-active, .fade-leave-active { transition: opacity 0.18s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
@keyframes spin { to { transform: rotate(360deg); } }

@media (max-width: 1100px) {
  .basic-panel { grid-template-columns: 1fr; }
  .preview-column { position: static; grid-row: 1; grid-template-columns: minmax(0, 1fr) minmax(240px, 0.7fr); }
  .renderer-preview-stage { height: 320px; }
}

@media (max-width: 860px) {
  .character-workbench { display: block; height: auto; min-height: calc(100svh - 114px); overflow: visible; }
  .character-browser { max-height: none; border-right: 0; border-bottom: 1px solid var(--border); }
  .browser-header { min-height: 72px; padding: 13px 14px 8px; }
  .browser-search { margin-inline: 14px; }
  .character-list { display: flex; flex: none; gap: 6px; overflow-x: auto; overflow-y: hidden; padding: 2px 14px 11px; scrollbar-width: none; }
  .character-list::-webkit-scrollbar { display: none; }
  .character-row { flex: 0 0 230px; margin: 0; }
  .empty-list { width: 100%; min-height: 100px; }
  .browser-draft-bar { min-height: 38px; padding-inline: 14px; }
  .character-editor { overflow: visible; }
  .editor-toolbar { align-items: flex-start; flex-direction: column; padding: 11px 14px; }
  .toolbar-actions { width: 100%; }
  .toolbar-actions .btn { flex: 1; }
  .editor-tabs { padding-inline: 10px; }
  .validation-banner { padding-inline: 14px; }
  .editor-scroll { overflow: visible; padding: 18px 16px 40px; }
  .preview-column { grid-template-columns: 1fr; }
  .trait-columns { grid-template-columns: 1fr; }
  .radar-chart { order: -1; }
}

@media (max-width: 620px) {
  .browser-header p { display: none; }
  .editor-title > div { display: grid; gap: 2px; }
  .editor-title code { max-width: 130px; }
  .toolbar-actions { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); }
  .toolbar-actions .btn { min-width: 0; padding-inline: 7px; font-size: 9px; }
  .tab-button { padding-inline: 10px; font-size: 9px; }
  .form-grid, .emotion-grid, .sprite-path-grid { grid-template-columns: 1fr; }
  .form-grid .full { grid-column: auto; }
  .renderer-preview-stage { height: 300px; }
  .section-heading { align-items: stretch; flex-direction: column; }
  .section-heading > .btn { align-self: flex-start; }
  .relationship-row { grid-template-columns: 32px minmax(0, 1fr) 38px; }
  .relationship-row input { grid-column: 2 / -1; }
  .knowledge-item { grid-template-columns: 1fr 34px; }
  .knowledge-item .form-field:first-child { grid-column: 1; }
  .knowledge-item .form-field:nth-child(2) { grid-column: 1 / -1; }
  .knowledge-item .icon-command { grid-column: 2; grid-row: 1; }
  .confirm-actions { display: grid; grid-template-columns: 1fr 1fr; }
  .confirm-actions .btn { justify-content: center; }
  .status-toast { right: 16px; bottom: 76px; }
}
</style>
