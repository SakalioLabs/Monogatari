<template>
  <div class="event-editor">
    <header class="event-toolbar">
      <div class="title-block">
        <span class="eyebrow">{{ t('event.editor-eyebrow', 'Event Authoring') }}</span>
        <h1>{{ t('event.title', 'Story Events') }}</h1>
        <span class="fingerprint" :title="catalogFingerprint">{{ shortFingerprint }}</span>
      </div>
      <div class="toolbar-actions">
        <button class="icon-btn" :title="t('event.new-event', 'New event')" :aria-label="t('event.new-event', 'New event')" @click="addEvent"><Plus :size="16" /></button>
        <button class="btn btn-secondary btn-sm" :disabled="!selectedEvent" @click="duplicateEvent"><Copy :size="14" />{{ t('authoring.duplicate', 'Duplicate') }}</button>
        <button class="btn btn-secondary btn-sm" :disabled="!dirty" @click="reloadCatalog"><RotateCcw :size="14" />{{ t('authoring.reload', 'Reload') }}</button>
        <button class="btn btn-secondary btn-sm" @click="runValidation"><ShieldCheck :size="14" />{{ t('dialogue.validate', 'Validate') }}</button>
        <button class="btn btn-primary btn-sm" :disabled="saving || !dirty || issues.length > 0" @click="saveCatalog">
          <Save :size="14" />{{ saving ? t('event.saving', 'Saving') : t('event.save-catalog', 'Save catalog') }}
        </button>
      </div>
    </header>

    <div v-if="statusMessage" class="status-strip" :class="statusKind">{{ statusMessage }}</div>

    <main class="event-workbench">
      <aside class="event-list-panel">
        <div class="list-controls">
          <input v-model.trim="search" class="input" type="search" :placeholder="t('event.search', 'Search events')" :aria-label="t('event.search', 'Search events')" />
          <select v-model="typeFilter" class="input" :aria-label="t('event.type-filter', 'Filter event type')">
            <option value="">{{ t('event.all-types', 'All types') }}</option>
            <option v-for="type in eventTypes" :key="type" :value="type">{{ type }}</option>
          </select>
        </div>
        <div class="list-summary">
          <span>{{ filteredEvents.length }} / {{ editableDocument.events.length }}</span>
          <span>{{ t('authoring.errors-count', '{count} errors', { count: issues.length }) }}</span>
        </div>
        <div class="event-list">
          <button
            v-for="item in filteredEvents"
            :key="`${item.event.event_id}-${item.index}`"
            class="event-row"
            :class="{ active: item.index === selectedIndex }"
            @click="selectedIndex = item.index"
          >
            <span class="event-row-main">
              <strong>{{ item.event.event_id || t('event.untitled-id', 'untitled_event') }}</strong>
              <small>{{ item.event.description || t('authoring.no-description', 'No description') }}</small>
            </span>
            <span class="event-row-meta">
              <span>{{ item.event.event_type || t('event.type', 'type') }}</span>
              <b>{{ item.event.actions?.length || 0 }}</b>
            </span>
          </button>
          <div v-if="filteredEvents.length === 0" class="empty-list">{{ t('event.no-matches', 'No matching events') }}</div>
        </div>
      </aside>

      <section v-if="selectedEvent" class="event-inspector">
        <div class="inspector-heading">
          <div>
            <span class="eyebrow">{{ t('event.definition', 'Event definition') }}</span>
            <h2>{{ selectedEvent.event_id || t('event.untitled', 'Untitled event') }}</h2>
          </div>
          <button class="icon-btn danger" :title="t('event.delete-event', 'Delete event')" :aria-label="t('event.delete-event', 'Delete event')" @click="deleteEvent"><Trash2 :size="15" /></button>
        </div>

        <section class="form-section identity-section">
          <label class="field">
            <span>{{ t('event.id', 'Event ID') }}</span>
            <input v-model.trim="selectedEvent.event_id" class="input mono" autocomplete="off" />
          </label>
          <label class="field">
            <span>{{ t('event.event-type', 'Event type') }}</span>
            <input v-model.trim="selectedEvent.event_type" class="input mono" list="event-type-options" autocomplete="off" />
            <datalist id="event-type-options">
              <option v-for="type in eventTypes" :key="type" :value="type" />
            </datalist>
          </label>
          <label class="field wide">
            <span>{{ t('common.description', 'Description') }}</span>
            <textarea v-model="selectedEvent.description" class="input" rows="3" maxlength="2048"></textarea>
          </label>
        </section>

        <section class="form-section">
          <div class="section-heading">
            <div>
              <span class="eyebrow">{{ t('event.trigger', 'Trigger') }}</span>
              <h3>{{ t('event.trigger-gates', 'Score and relationship gates') }}</h3>
            </div>
            <label class="toggle-line">
              <span>{{ t('event.repeatable', 'Repeatable') }}</span>
              <input v-model="selectedEvent.repeatable" type="checkbox" />
            </label>
          </div>
          <div class="gate-grid">
            <div class="gate-control">
              <label class="toggle-line">
                <span>{{ t('event.relationship', 'Relationship') }}</span>
                <input type="checkbox" :checked="hasRelationshipGate" @change="toggleRelationshipGate" />
              </label>
              <input
                v-if="hasRelationshipGate"
                v-model.number="selectedEvent.rule!.min_relationship"
                class="input"
                type="number"
                min="-1"
                max="1"
                step="0.05"
              />
              <span v-else class="gate-off">{{ t('event.not-required', 'Not required') }}</span>
            </div>
            <div class="gate-control score-control">
              <label class="toggle-line">
                <span>{{ t('event.conversation-score', 'Conversation score') }}</span>
                <input type="checkbox" :checked="hasScoreGate" @change="toggleScoreGate" />
              </label>
              <template v-if="hasScoreGate">
                <select v-model="selectedEvent.rule!.score_metric" class="input">
                  <option value="friendliness">{{ t('quality.friendliness', 'Friendliness') }}</option>
                  <option value="engagement">{{ t('quality.engagement', 'Engagement') }}</option>
                  <option value="creativity">{{ t('quality.creativity', 'Creativity') }}</option>
                  <option value="overall">{{ t('quality.overall', 'Overall') }}</option>
                </select>
                <input v-model.number="selectedEvent.rule!.min_score" class="input" type="number" min="0" max="1" step="0.05" />
              </template>
              <span v-else class="gate-off">{{ t('event.not-required', 'Not required') }}</span>
            </div>
            <div class="gate-control">
              <label class="toggle-line">
                <span>{{ t('event.evaluation-count', 'Evaluation count') }}</span>
                <input type="checkbox" :checked="hasEvaluationGate" @change="toggleEvaluationGate" />
              </label>
              <input
                v-if="hasEvaluationGate"
                v-model.number="selectedEvent.rule!.min_evaluation_count"
                class="input"
                type="number"
                min="0"
                max="1000000"
                step="1"
              />
              <span v-else class="gate-off">{{ t('event.not-required', 'Not required') }}</span>
            </div>
          </div>
        </section>

        <section class="form-section scope-section">
          <div class="section-heading">
            <div>
              <span class="eyebrow">{{ t('event.scope', 'Scope') }}</span>
              <h3>{{ t('characters.title', 'Characters') }}</h3>
            </div>
            <span class="scope-mode">{{ selectedEvent.character_ids?.length ? t('event.selected-count', '{count} selected', { count: selectedEvent.character_ids.length }) : t('event.global', 'Global') }}</span>
          </div>
          <div class="character-scopes">
            <label v-for="character in characters" :key="character.id" class="scope-option">
              <input
                type="checkbox"
                :checked="selectedEvent.character_ids?.includes(character.id)"
                @change="toggleCharacter(character.id)"
              />
              <span>{{ character.name }}</span>
              <small>{{ character.id }}</small>
            </label>
            <span v-if="characters.length === 0" class="gate-off">{{ t('event.no-characters', 'No project characters loaded') }}</span>
          </div>
        </section>

        <section class="form-section actions-section">
          <div class="section-heading">
            <div>
              <span class="eyebrow">{{ t('event.effects', 'Effects') }}</span>
              <h3>{{ t('event.actions', 'Actions') }}</h3>
            </div>
            <div class="add-action">
              <select v-model="newActionType" class="input" :aria-label="t('event.new-action-type', 'New action type')">
                <option value="unlock_scene">{{ t('event.unlock-scene', 'Unlock scene') }}</option>
                <option value="unlock_dialogue">{{ t('event.unlock-dialogue', 'Unlock dialogue') }}</option>
                <option value="unlock_ending">{{ t('event.unlock-ending', 'Unlock ending') }}</option>
                <option value="set_flag">{{ t('event.set-flag', 'Set flag') }}</option>
              </select>
              <button class="icon-btn" :title="t('event.add-action', 'Add action')" :aria-label="t('event.add-action', 'Add action')" @click="addAction"><Plus :size="16" /></button>
            </div>
          </div>
          <div class="action-list">
            <div v-for="(action, index) in selectedEvent.actions" :key="index" class="action-row">
              <span class="action-index">{{ index + 1 }}</span>
              <select class="input" :value="action.type" @change="changeActionType(index, ($event.target as HTMLSelectElement).value)">
                <option value="unlock_scene">{{ t('event.unlock-scene', 'Unlock scene') }}</option>
                <option value="unlock_dialogue">{{ t('event.unlock-dialogue', 'Unlock dialogue') }}</option>
                <option value="unlock_ending">{{ t('event.unlock-ending', 'Unlock ending') }}</option>
                <option value="set_flag">{{ t('event.set-flag', 'Set flag') }}</option>
              </select>
              <template v-if="action.type === 'unlock_scene'">
                <input v-model.trim="action.scene_id" class="input mono" list="scene-options" placeholder="scene_id" />
              </template>
              <template v-else-if="action.type === 'unlock_dialogue'">
                <input v-model.trim="action.dialogue_id" class="input mono" list="dialogue-options" placeholder="dialogue_id" />
              </template>
              <template v-else-if="action.type === 'unlock_ending'">
                <input v-model.trim="action.ending_id" class="input mono" list="ending-options" placeholder="ending_id" />
              </template>
              <template v-else>
                <input v-model.trim="action.flag" class="input mono" placeholder="story.flag" />
                <label class="boolean-value">
                  <input v-model="action.value" type="checkbox" />
                  <span>{{ action.value ? t('event.true', 'True') : t('event.false', 'False') }}</span>
                </label>
              </template>
              <button class="icon-btn danger" :title="t('event.remove-action', 'Remove action')" :aria-label="t('event.remove-action', 'Remove action')" @click="removeAction(index)"><Trash2 :size="15" /></button>
            </div>
            <div v-if="!selectedEvent.actions?.length" class="empty-actions">{{ t('event.no-effects', 'No effects configured') }}</div>
          </div>
          <datalist id="scene-options">
            <option v-for="scene in scenes" :key="scene.id" :value="scene.id">{{ scene.name }}</option>
          </datalist>
          <datalist id="dialogue-options">
            <option v-for="dialogue in dialogues" :key="dialogue.id" :value="dialogue.id">{{ dialogue.title }}</option>
          </datalist>
          <datalist id="ending-options">
            <option v-for="ending in endings" :key="ending.id" :value="ending.id">{{ ending.title }}</option>
          </datalist>
        </section>

        <section class="form-section metadata-section">
          <div class="section-heading">
            <div>
              <span class="eyebrow">{{ t('event.payload', 'Payload') }}</span>
              <h3>{{ t('event.metadata-json', 'Metadata JSON') }}</h3>
            </div>
          </div>
          <textarea
            class="input mono metadata-input"
            rows="5"
            :value="metadataText"
            spellcheck="false"
            @input="metadataText = ($event.target as HTMLTextAreaElement).value"
            @blur="applyMetadata"
          ></textarea>
        </section>
      </section>

      <section v-else class="empty-inspector">
        <span class="empty-mark">EV</span>
        <h2>{{ t('event.no-selection', 'No event selected') }}</h2>
        <button class="btn btn-primary" @click="addEvent"><Plus :size="15" />{{ t('event.create-event', 'Create event') }}</button>
      </section>

      <aside class="validation-panel">
        <div class="validation-heading">
          <span class="eyebrow">{{ t('event.catalog-health', 'Catalog health') }}</span>
          <strong :class="issues.length ? 'has-errors' : 'is-valid'">{{ issues.length ? t('authoring.errors-count', '{count} errors', { count: issues.length }) : t('event.valid', 'Valid') }}</strong>
        </div>
        <div class="validation-list">
          <div v-for="issue in issues" :key="issue" class="validation-item error">{{ issue }}</div>
          <div v-for="warning in warnings" :key="warning" class="validation-item warning">{{ warning }}</div>
          <div v-if="issues.length === 0 && warnings.length === 0" class="validation-empty">{{ t('event.catalog-ready', 'Catalog is ready to save.') }}</div>
        </div>
        <div class="catalog-metrics">
          <span><b>{{ editableDocument.events.length }}</b> {{ t('event.events', 'events') }}</span>
          <span><b>{{ totalActions }}</b> {{ t('event.actions-lower', 'actions') }}</span>
          <span><b>{{ lockedTargetCount }}</b> {{ t('event.gated-targets', 'gated targets') }}</span>
        </div>
      </aside>
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { onBeforeRouteLeave } from 'vue-router'
import { Copy, Plus, RotateCcw, Save, ShieldCheck, Trash2 } from '@lucide/vue'
import { useI18n } from '../lib/i18n'
import { invokeCommand } from '../lib/tauri'
import { loadStoryDialogues, loadStoryEndings, loadStoryScenes } from '../lib/storyContent'
import {
  STORY_EVENT_CATALOG_SCHEMA_V1,
  reloadStoryEventCatalog,
  saveStoryEventCatalog,
  storyEventCatalogDocument,
  type StoryEventAction,
  type StoryEventDocument,
  type StoryEventDraft,
} from '../lib/storyEvents'

interface CharacterSummary { id: string; name: string }
interface SceneSummary { id: string; name: string }
interface DialogueSummary { id: string; title: string; node_count: number }
interface EndingSummary { id: string; title: string }

const { t } = useI18n()
const editableDocument = ref<StoryEventDocument>({ schema: STORY_EVENT_CATALOG_SCHEMA_V1, events: [] })
const catalogFingerprint = ref('')
const baseline = ref('')
const selectedIndex = ref(-1)
const search = ref('')
const typeFilter = ref('')
const characters = ref<CharacterSummary[]>([])
const scenes = ref<SceneSummary[]>([])
const dialogues = ref<DialogueSummary[]>([])
const endings = ref<EndingSummary[]>([])
const newActionType = ref<StoryEventAction['type']>('unlock_scene')
const metadataText = ref('{}')
const metadataError = ref<string | null>(null)
const statusMessage = ref<string | null>(null)
const statusKind = ref<'success' | 'error' | 'info'>('info')
const saving = ref(false)

const selectedEvent = computed(() => editableDocument.value.events[selectedIndex.value] || null)
const dirty = computed(() => JSON.stringify(editableDocument.value) !== baseline.value)
const shortFingerprint = computed(() => catalogFingerprint.value ? catalogFingerprint.value.slice(0, 12) : t('event.browser-draft', 'browser draft'))
const eventTypes = computed(() => [...new Set([
  'relationship_milestone',
  'special_dialogue',
  'cumulative_achievement',
  ...editableDocument.value.events.map((event) => event.event_type).filter(Boolean),
])].sort())
const filteredEvents = computed(() => editableDocument.value.events
  .map((event, index) => ({ event, index }))
  .filter(({ event }) => !typeFilter.value || event.event_type === typeFilter.value)
  .filter(({ event }) => {
    const needle = search.value.toLowerCase()
    return !needle || `${event.event_id} ${event.event_type} ${event.description}`.toLowerCase().includes(needle)
  }))
const hasRelationshipGate = computed(() => selectedEvent.value?.rule?.min_relationship !== undefined)
const hasScoreGate = computed(() => selectedEvent.value?.rule?.score_metric !== undefined || selectedEvent.value?.rule?.min_score !== undefined)
const hasEvaluationGate = computed(() => selectedEvent.value?.rule?.min_evaluation_count !== undefined)
const totalActions = computed(() => editableDocument.value.events.reduce((sum, event) => sum + (event.actions?.length || 0), 0))
const lockedTargetCount = computed(() => new Set(editableDocument.value.events.flatMap((event) => (event.actions || [])
  .filter((action) => action.type !== 'set_flag')
  .map((action) => action.type === 'unlock_scene' ? `scene:${action.scene_id}`
    : action.type === 'unlock_dialogue' ? `dialogue:${action.dialogue_id}` : `ending:${action.ending_id}`))).size)

const issues = computed(() => validateDocument())
const warnings = computed(() => documentWarnings())

watch(selectedIndex, () => {
  metadataError.value = null
  metadataText.value = JSON.stringify(selectedEvent.value?.data || {}, null, 2)
})

function normalizeDraft(event: StoryEventDraft): StoryEventDraft {
  return {
    ...event,
    data: event.data && typeof event.data === 'object' ? event.data : {},
    actions: [...(event.actions || [])],
    character_ids: [...(event.character_ids || [])],
    repeatable: Boolean(event.repeatable),
    rule: { ...(event.rule || {}) },
  }
}

async function loadCatalog(force = false) {
  if (dirty.value && !force && !window.confirm(t('event.confirm.discard', 'Discard unsaved Story Event changes?'))) return
  statusMessage.value = null
  try {
    characters.value = await invokeCommand<CharacterSummary[]>('get_characters', undefined, [
      { id: 'sakura', name: 'Sakura' }, { id: 'luna', name: 'Luna' },
    ])
    const [snapshot, sceneCatalog, dialogueCatalog, endingCatalog] = await Promise.all([
      reloadStoryEventCatalog(),
      loadStoryScenes(),
      loadStoryDialogues(),
      loadStoryEndings(),
    ])
    const document = storyEventCatalogDocument(snapshot)
    document.events = document.events.map(normalizeDraft)
    editableDocument.value = document
    scenes.value = sceneCatalog
    dialogues.value = dialogueCatalog
    endings.value = endingCatalog
    catalogFingerprint.value = snapshot.catalog_fingerprint
    baseline.value = JSON.stringify(document)
    selectedIndex.value = document.events.length ? 0 : -1
    statusMessage.value = t('event.status.loaded', 'Loaded {count} events', { count: document.events.length })
    statusKind.value = 'success'
  } catch (error) {
    statusMessage.value = String(error)
    statusKind.value = 'error'
  }
}

async function reloadCatalog() {
  await loadCatalog(false)
}

function uniqueEventId(base = 'new_event'): string {
  const existing = new Set(editableDocument.value.events.map((event) => event.event_id))
  if (!existing.has(base)) return base
  let index = 2
  while (existing.has(`${base}_${index}`)) index += 1
  return `${base}_${index}`
}

function addEvent() {
  const event: StoryEventDraft = normalizeDraft({
    event_id: uniqueEventId(),
    event_type: 'special_dialogue',
    description: t('event.new-description', 'Describe the player-facing story milestone.'),
    actions: [],
    rule: { min_relationship: 0.5 },
  })
  editableDocument.value.events.push(event)
  selectedIndex.value = editableDocument.value.events.length - 1
}

function duplicateEvent() {
  if (!selectedEvent.value) return
  const copy = structuredClone(selectedEvent.value)
  copy.event_id = uniqueEventId(`${selectedEvent.value.event_id}_copy`)
  editableDocument.value.events.splice(selectedIndex.value + 1, 0, copy)
  selectedIndex.value += 1
}

function deleteEvent() {
  if (!selectedEvent.value || !window.confirm(t('event.confirm.delete', 'Delete event {id}?', { id: selectedEvent.value.event_id }))) return
  editableDocument.value.events.splice(selectedIndex.value, 1)
  selectedIndex.value = Math.min(selectedIndex.value, editableDocument.value.events.length - 1)
}

function ensureRule() {
  if (selectedEvent.value && !selectedEvent.value.rule) selectedEvent.value.rule = {}
}

function toggleRelationshipGate(event: Event) {
  ensureRule()
  if (!selectedEvent.value?.rule) return
  if ((event.target as HTMLInputElement).checked) selectedEvent.value.rule.min_relationship = 0.5
  else delete selectedEvent.value.rule.min_relationship
}

function toggleScoreGate(event: Event) {
  ensureRule()
  if (!selectedEvent.value?.rule) return
  if ((event.target as HTMLInputElement).checked) {
    selectedEvent.value.rule.score_metric = 'overall'
    selectedEvent.value.rule.min_score = 0.7
  } else {
    delete selectedEvent.value.rule.score_metric
    delete selectedEvent.value.rule.min_score
  }
}

function toggleEvaluationGate(event: Event) {
  ensureRule()
  if (!selectedEvent.value?.rule) return
  if ((event.target as HTMLInputElement).checked) selectedEvent.value.rule.min_evaluation_count = 1
  else delete selectedEvent.value.rule.min_evaluation_count
}

function toggleCharacter(characterId: string) {
  if (!selectedEvent.value) return
  const scopes = selectedEvent.value.character_ids || (selectedEvent.value.character_ids = [])
  const index = scopes.indexOf(characterId)
  if (index >= 0) scopes.splice(index, 1)
  else scopes.push(characterId)
  scopes.sort()
}

function makeAction(type: string): StoryEventAction {
  if (type === 'unlock_dialogue') return { type, dialogue_id: dialogues.value[0]?.id || '' }
  if (type === 'unlock_ending') return { type, ending_id: endings.value[0]?.id || 'new_ending' }
  if (type === 'set_flag') return { type, flag: 'story.event_complete', value: true }
  return { type: 'unlock_scene', scene_id: scenes.value[0]?.id || '' }
}

function addAction() {
  if (!selectedEvent.value) return
  if (!selectedEvent.value.actions) selectedEvent.value.actions = []
  selectedEvent.value.actions.push(makeAction(newActionType.value))
}

function changeActionType(index: number, type: string) {
  if (!selectedEvent.value?.actions) return
  selectedEvent.value.actions[index] = makeAction(type)
}

function removeAction(index: number) {
  selectedEvent.value?.actions?.splice(index, 1)
}

function applyMetadata() {
  if (!selectedEvent.value) return
  try {
    const value = JSON.parse(metadataText.value)
    if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error(t('event.error.metadata-object', 'Metadata must be a JSON object'))
    selectedEvent.value.data = value
    metadataError.value = null
  } catch (error) {
    metadataError.value = String(error)
  }
}

function validateDocument(): string[] {
  const result: string[] = []
  const ids = new Set<string>()
  const portable = /^[A-Za-z0-9_.-]+$/
  const sceneIds = new Set(scenes.value.map((scene) => scene.id))
  const dialogueIds = new Set(dialogues.value.map((dialogue) => dialogue.id))
  const endingIds = new Set(endings.value.map((ending) => ending.id))
  const characterIds = new Set(characters.value.map((character) => character.id))
  if (metadataError.value) result.push(t('event.error.metadata', 'Metadata: {message}', { message: metadataError.value }))
  if (editableDocument.value.events.length > 512) result.push(t('event.error.catalog-limit', 'Catalog exceeds 512 events'))
  for (const event of editableDocument.value.events) {
    const label = event.event_id || t('event.untitled-id', 'untitled_event')
    if (!portable.test(event.event_id) || event.event_id.length > 128) result.push(t('event.error.invalid-id', '{label}: invalid event ID', { label }))
    if (ids.has(event.event_id)) result.push(t('event.error.duplicate-id', '{label}: duplicate event ID', { label }))
    ids.add(event.event_id)
    if (!portable.test(event.event_type) || event.event_type.length > 128) result.push(t('event.error.invalid-type', '{label}: invalid event type', { label }))
    if (!event.description?.trim() || event.description.length > 2048) result.push(t('event.error.description', '{label}: description is required and limited to 2048 characters', { label }))
    const scopes = event.character_ids || []
    if (new Set(scopes).size !== scopes.length) result.push(t('event.error.duplicate-scope', '{label}: duplicate character scope', { label }))
    for (const characterId of scopes) if (!characterIds.has(characterId)) result.push(t('event.error.unknown-character', '{label}: unknown character {id}', { label, id: characterId }))
    const rule = event.rule || {}
    if (rule.min_relationship !== undefined && (!Number.isFinite(rule.min_relationship) || rule.min_relationship < -1 || rule.min_relationship > 1)) result.push(t('event.error.relationship-threshold', '{label}: relationship threshold must be between -1 and 1', { label }))
    if ((rule.score_metric === undefined) !== (rule.min_score === undefined)) result.push(t('event.error.score-pair', '{label}: score metric and threshold must be configured together', { label }))
    if (rule.min_score !== undefined && (!Number.isFinite(rule.min_score) || rule.min_score < 0 || rule.min_score > 1)) result.push(t('event.error.score-threshold', '{label}: score threshold must be between 0 and 1', { label }))
    if (rule.min_evaluation_count !== undefined && (!Number.isInteger(rule.min_evaluation_count) || rule.min_evaluation_count < 0 || rule.min_evaluation_count > 1_000_000)) result.push(t('event.error.evaluation-count', '{label}: evaluation count is invalid', { label }))
    const actionKeys = new Set<string>()
    for (const action of event.actions || []) {
      const key = JSON.stringify(action)
      if (actionKeys.has(key)) result.push(t('event.error.duplicate-action', '{label}: duplicate action {type}', { label, type: action.type }))
      actionKeys.add(key)
      if (action.type === 'unlock_scene' && !sceneIds.has(action.scene_id)) result.push(t('event.error.unknown-scene', '{label}: unknown scene {id}', { label, id: action.scene_id }))
      if (action.type === 'unlock_dialogue' && !dialogueIds.has(action.dialogue_id)) result.push(t('event.error.unknown-dialogue', '{label}: unknown dialogue {id}', { label, id: action.dialogue_id }))
      if (action.type === 'unlock_ending' && !endingIds.has(action.ending_id)) result.push(t('event.error.unknown-ending', '{label}: unknown ending {id}', { label, id: action.ending_id }))
      if (action.type === 'set_flag' && !portable.test(action.flag)) result.push(t('event.error.invalid-flag', '{label}: invalid flag name', { label }))
    }
  }
  return result
}

function documentWarnings(): string[] {
  const result: string[] = []
  for (const event of editableDocument.value.events) {
    const rule = event.rule || {}
    if (!(event.actions?.length)) result.push(t('event.warning.no-effects', '{id}: no effects configured', { id: event.event_id }))
    if (rule.min_relationship === undefined && rule.min_score === undefined && rule.min_evaluation_count === undefined) {
      result.push(t('event.warning.no-trigger', '{id}: no trigger gate; eligible immediately', { id: event.event_id }))
    }
  }
  return result
}

function runValidation() {
  applyMetadata()
  statusMessage.value = issues.value.length ? issues.value[0] : t('event.status.validation-passed', 'Validation passed with {count} warnings', { count: warnings.value.length })
  statusKind.value = issues.value.length ? 'error' : 'success'
}

async function saveCatalog() {
  applyMetadata()
  if (issues.value.length > 0) {
    runValidation()
    return
  }
  saving.value = true
  try {
    const snapshot = await saveStoryEventCatalog(structuredClone(editableDocument.value), catalogFingerprint.value)
    const document = storyEventCatalogDocument(snapshot)
    document.events = document.events.map(normalizeDraft)
    editableDocument.value = document
    catalogFingerprint.value = snapshot.catalog_fingerprint
    baseline.value = JSON.stringify(document)
    selectedIndex.value = Math.min(selectedIndex.value, document.events.length - 1)
    statusMessage.value = t('event.status.saved', 'Saved {count} events', { count: snapshot.event_count })
    statusKind.value = 'success'
  } catch (error) {
    statusMessage.value = String(error)
    statusKind.value = 'error'
  } finally {
    saving.value = false
  }
}

function beforeUnload(event: BeforeUnloadEvent) {
  if (!dirty.value) return
  event.preventDefault()
  event.returnValue = ''
}

onBeforeRouteLeave(() => !dirty.value || window.confirm(t('event.confirm.leave', 'Leave without saving Story Event changes?')))
onMounted(() => {
  window.addEventListener('beforeunload', beforeUnload)
  loadCatalog(true)
})
onBeforeUnmount(() => window.removeEventListener('beforeunload', beforeUnload))
</script>

<style scoped>
.event-editor { min-height: 100vh; display: flex; flex-direction: column; background: var(--surface-0); color: var(--text-primary); }
.event-toolbar { min-height: 76px; display: flex; align-items: center; justify-content: space-between; gap: 20px; padding: 14px 20px; border-bottom: 1px solid var(--border); background: var(--surface-1); }
.title-block { min-width: 0; display: flex; align-items: baseline; gap: 10px; }
.title-block h1 { font-size: 22px; white-space: nowrap; }
.eyebrow { color: var(--text-tertiary); font-size: 10px; font-weight: 800; letter-spacing: 0; text-transform: uppercase; }
.fingerprint { overflow: hidden; max-width: 130px; color: var(--text-tertiary); font-family: var(--font-mono); font-size: 11px; text-overflow: ellipsis; white-space: nowrap; }
.toolbar-actions { display: flex; align-items: center; justify-content: flex-end; gap: 8px; flex-wrap: wrap; }
.icon-btn { width: 34px; height: 34px; display: inline-grid; flex: 0 0 34px; place-items: center; padding: 0; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--surface-2); color: var(--text-primary); cursor: pointer; font: inherit; font-size: 18px; font-weight: 800; }
.icon-btn:hover { border-color: var(--brand); color: var(--brand-light); }
.icon-btn:disabled { opacity: .45; cursor: not-allowed; }
.icon-btn.danger:hover { border-color: var(--error); color: var(--error); }
.status-strip { padding: 8px 20px; border-bottom: 1px solid var(--border); font-size: 12px; }
.status-strip.success { color: var(--success); background: rgba(45, 212, 191, .07); }
.status-strip.error { color: var(--error); background: rgba(248, 113, 113, .08); }
.status-strip.info { color: var(--text-secondary); background: var(--surface-1); }
.event-workbench { flex: 1; min-height: 0; display: grid; grid-template-columns: 260px minmax(460px, 1fr) 250px; }
.event-list-panel, .validation-panel { min-width: 0; background: var(--surface-1); }
.event-list-panel { display: flex; flex-direction: column; border-right: 1px solid var(--border); }
.validation-panel { border-left: 1px solid var(--border); }
.list-controls { display: grid; gap: 8px; padding: 14px; border-bottom: 1px solid var(--border); }
.input { width: 100%; min-width: 0; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--surface-2); color: var(--text-primary); padding: 8px 10px; font: inherit; font-size: 12px; }
.input:focus { outline: 2px solid rgba(45, 212, 191, .22); border-color: var(--brand); }
.mono { font-family: var(--font-mono); }
.list-summary { display: flex; justify-content: space-between; padding: 8px 14px; color: var(--text-tertiary); font-size: 10px; font-weight: 700; }
.event-list { flex: 1; min-height: 0; overflow: auto; }
.event-row { width: 100%; display: grid; grid-template-columns: minmax(0, 1fr) auto; gap: 10px; padding: 12px 14px; border: 0; border-bottom: 1px solid var(--border); background: transparent; color: inherit; cursor: pointer; text-align: left; }
.event-row:hover { background: var(--surface-2); }
.event-row.active { box-shadow: inset 3px 0 var(--brand); background: var(--surface-2); }
.event-row-main { min-width: 0; display: grid; gap: 3px; }
.event-row-main strong, .event-row-main small { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.event-row-main strong { font-family: var(--font-mono); font-size: 12px; }
.event-row-main small { color: var(--text-tertiary); font-size: 10px; }
.event-row-meta { display: grid; justify-items: end; gap: 4px; color: var(--text-tertiary); font-size: 9px; }
.event-row-meta b { min-width: 18px; padding: 1px 5px; border-radius: 8px; background: var(--surface-3); color: var(--text-secondary); text-align: center; }
.empty-list, .empty-actions, .validation-empty { padding: 24px 14px; color: var(--text-tertiary); font-size: 12px; text-align: center; }
.event-inspector { min-width: 0; overflow: auto; padding: 20px 24px 40px; }
.inspector-heading, .section-heading { display: flex; align-items: center; justify-content: space-between; gap: 16px; }
.inspector-heading { margin-bottom: 18px; }
.inspector-heading h2 { margin-top: 3px; font-size: 20px; overflow-wrap: anywhere; }
.form-section { padding: 18px 0; border-top: 1px solid var(--border); }
.identity-section { display: grid; grid-template-columns: minmax(180px, 1fr) minmax(180px, 1fr); gap: 14px; border-top: 0; padding-top: 0; }
.field { min-width: 0; display: grid; gap: 6px; color: var(--text-secondary); font-size: 11px; font-weight: 700; }
.field.wide { grid-column: 1 / -1; }
textarea.input { resize: vertical; line-height: 1.5; }
.section-heading { margin-bottom: 14px; }
.section-heading h3 { margin-top: 2px; font-size: 14px; }
.toggle-line { display: flex; align-items: center; justify-content: space-between; gap: 10px; color: var(--text-secondary); font-size: 11px; font-weight: 700; }
.toggle-line input, .scope-option input, .boolean-value input { accent-color: var(--brand); }
.gate-grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 12px; }
.gate-control { min-height: 104px; display: grid; align-content: start; gap: 9px; padding: 12px; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--surface-1); }
.score-control { grid-template-columns: minmax(0, 1fr) 90px; }
.score-control .toggle-line, .score-control .gate-off { grid-column: 1 / -1; }
.gate-off { color: var(--text-tertiary); font-size: 11px; }
.scope-mode { color: var(--brand-light); font-size: 11px; font-weight: 700; }
.character-scopes { display: grid; grid-template-columns: repeat(auto-fill, minmax(150px, 1fr)); gap: 8px; }
.scope-option { min-width: 0; display: grid; grid-template-columns: auto minmax(0, 1fr); column-gap: 8px; align-items: center; padding: 9px; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--surface-1); cursor: pointer; }
.scope-option small { grid-column: 2; overflow: hidden; color: var(--text-tertiary); font-family: var(--font-mono); font-size: 9px; text-overflow: ellipsis; }
.add-action { display: flex; gap: 7px; }
.add-action .input { width: 160px; }
.action-list { display: grid; gap: 8px; }
.action-row { display: grid; grid-template-columns: 24px 150px minmax(150px, 1fr) auto; gap: 8px; align-items: center; }
.action-index { color: var(--text-tertiary); font-family: var(--font-mono); font-size: 10px; text-align: center; }
.boolean-value { min-width: 86px; display: flex; align-items: center; gap: 7px; color: var(--text-secondary); font-size: 11px; }
.metadata-input { min-height: 110px; }
.empty-inspector { display: grid; align-content: center; justify-items: center; gap: 12px; color: var(--text-tertiary); }
.empty-mark { display: grid; place-items: center; width: 56px; height: 56px; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--surface-2); color: var(--brand-light); font-weight: 900; }
.validation-heading { display: grid; gap: 6px; padding: 16px; border-bottom: 1px solid var(--border); }
.validation-heading strong { font-size: 15px; }
.has-errors { color: var(--error); }
.is-valid { color: var(--success); }
.validation-list { max-height: calc(100vh - 250px); overflow: auto; }
.validation-item { padding: 10px 14px; border-bottom: 1px solid var(--border); font-size: 10px; line-height: 1.45; overflow-wrap: anywhere; }
.validation-item.error { color: var(--error); }
.validation-item.warning { color: var(--warning); }
.catalog-metrics { display: grid; grid-template-columns: repeat(3, 1fr); gap: 1px; border-top: 1px solid var(--border); background: var(--border); }
.catalog-metrics span { display: grid; gap: 2px; padding: 10px 6px; background: var(--surface-1); color: var(--text-tertiary); font-size: 9px; text-align: center; }
.catalog-metrics b { color: var(--text-primary); font-size: 14px; }

@media (max-width: 1120px) {
  .event-workbench { grid-template-columns: 230px minmax(0, 1fr); }
  .validation-panel { grid-column: 1 / -1; border-top: 1px solid var(--border); border-left: 0; }
  .validation-list { max-height: 160px; }
}

@media (max-width: 760px) {
  .event-toolbar { align-items: flex-start; flex-direction: column; }
  .toolbar-actions { justify-content: flex-start; }
  .event-workbench { display: block; }
  .event-list-panel { max-height: 280px; border-right: 0; border-bottom: 1px solid var(--border); }
  .event-inspector { padding: 18px 14px 32px; }
  .identity-section, .gate-grid { grid-template-columns: 1fr; }
  .action-row { grid-template-columns: 24px minmax(0, 1fr) 34px; }
  .action-row > .input:nth-of-type(2), .action-row > .boolean-value { grid-column: 2; }
  .action-row > .icon-btn { grid-column: 3; grid-row: 1; }
}
</style>
