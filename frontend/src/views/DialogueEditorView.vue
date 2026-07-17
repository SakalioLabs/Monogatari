<template>
  <div class="dialogue-editor">
    <header class="editor-header">
      <div class="header-copy">
        <span class="eyebrow">{{ t('dialogue.editor-eyebrow', 'Narrative Design') }}</span>
        <h1>{{ t('dialogue.graph-title', 'Dialogue Graph') }}</h1>
        <p>{{ t('dialogue.catalog-summary', '{dialogues} dialogues · {nodes} nodes · {choices} choices', {
          dialogues: snapshot?.dialogue_count || 0,
          nodes: snapshot?.node_count || 0,
          choices: snapshot?.choice_count || 0,
        }) }}</p>
      </div>
      <div class="header-actions">
        <button class="btn btn-secondary btn-sm" :disabled="busy" @click="createDialogue"><Plus :size="14" />{{ t('authoring.new', 'New') }}</button>
        <button class="btn btn-secondary btn-sm" :disabled="!draft || busy" @click="duplicateDialogue"><Copy :size="14" />{{ t('authoring.duplicate', 'Duplicate') }}</button>
        <button class="btn btn-secondary btn-sm" :disabled="busy" @click="reloadCatalog"><RotateCcw :size="14" />{{ t('authoring.reload', 'Reload') }}</button>
        <button class="btn btn-secondary btn-sm" :disabled="!canPreview || busy" @click="previewDialogue"><Play :size="14" />{{ t('authoring.story-mode', 'Playtest') }}</button>
        <button class="btn btn-primary btn-sm" :disabled="!canSave || busy" @click="saveDialogue">
          <Save :size="14" />{{ busy ? t('authoring.working', 'Working') : t('common.save', 'Save') }}
        </button>
      </div>
    </header>

    <div class="catalog-strip">
      <span><strong>{{ filteredDialogues.length }}</strong> {{ t('authoring.visible', 'visible') }}</span>
      <span><strong>{{ snapshot?.llm_node_count || 0 }}</strong> {{ t('dialogue.llm-nodes', 'LLM nodes') }}</span>
      <span><strong>{{ gatedCount }}</strong> {{ t('authoring.gated', 'gated') }}</span>
      <span><strong>{{ snapshot?.catalog_fingerprint.slice(0, 12) || t('authoring.unavailable', 'unavailable') }}</strong> {{ t('authoring.catalog', 'catalog') }}</span>
      <span v-if="dirty" class="dirty-indicator">{{ t('authoring.unsaved-changes', 'Unsaved changes') }}</span>
    </div>

    <main class="editor-workspace">
      <aside class="dialogue-list" :aria-label="t('dialogue.catalog-aria', 'Dialogue catalog')">
        <div class="list-toolbar">
          <label class="search-field">
            <span class="sr-only">{{ t('dialogue.search', 'Search dialogues') }}</span>
            <input v-model.trim="search" class="input" type="search" :placeholder="t('dialogue.search', 'Search dialogues')" />
          </label>
          <span>{{ filteredDialogues.length }}</span>
        </div>
        <div class="dialogue-items">
          <button
            v-for="dialogue in filteredDialogues"
            :key="dialogue.id"
            class="dialogue-item"
            :class="{ active: selectedDialogueId === dialogue.id }"
            @click="selectDialogue(dialogue)"
          >
            <span class="dialogue-mark">DL</span>
            <span class="dialogue-item-copy">
              <strong>{{ dialogue.title }}</strong>
              <small>{{ dialogue.id }}</small>
              <span>
                {{ t('dialogue.nodes-count', '{count} nodes', { count: Object.keys(dialogue.nodes).length }) }} ·
                {{ dialogue.access.gated ? t('authoring.gated-label', 'Gated') : t('authoring.open', 'Open') }}
              </span>
            </span>
          </button>
          <div v-if="filteredDialogues.length === 0" class="empty-list">{{ t('dialogue.no-dialogues', 'No dialogues') }}</div>
        </div>
      </aside>

      <section v-if="draft" class="graph-panel">
        <div class="graph-header">
          <div class="graph-title">
            <span class="eyebrow">{{ sourcePath }}</span>
            <h2>{{ draft.title || t('dialogue.untitled', 'Untitled dialogue') }}</h2>
            <p>{{ draft.description || t('authoring.no-description', 'No description') }}</p>
          </div>
          <div class="graph-actions">
            <span :class="validationIssues.length ? 'graph-state invalid' : 'graph-state valid'">
              {{ validationIssues.length ? t('authoring.issues-count', '{count} issues', { count: validationIssues.length }) : t('dialogue.reachable-count', '{count} reachable', { count: nodeOrder.length }) }}
            </span>
            <button class="btn btn-secondary btn-sm" @click="addNode"><Plus :size="14" />{{ t('dialogue.add-node', 'Add Node') }}</button>
          </div>
        </div>

        <div v-if="validationIssues.length" class="validation-banner error" role="alert">
          <strong>{{ t('authoring.blocking-issues', '{count} blocking issues', { count: validationIssues.length }) }}</strong>
          <span>{{ validationIssues[0] }}</span>
        </div>
        <div v-else-if="warnings.length" class="validation-banner warning">
          <strong>{{ t('authoring.warnings-count', '{count} warnings', { count: warnings.length }) }}</strong>
          <span>{{ warnings[0] }}</span>
        </div>
        <div v-else class="validation-banner valid">
          <strong>{{ t('dialogue.graph-valid', 'Graph valid') }}</strong>
          <span>{{ t('dialogue.graph-valid-copy', 'All nodes and transition targets are reachable.') }}</span>
        </div>

        <div class="graph-scroll">
          <div class="route-rail" aria-hidden="true"></div>
          <button
            v-for="(nodeId, index) in nodeOrder"
            :key="nodeId"
            class="node-card"
            :class="{
              selected: selectedNodeId === nodeId,
              start: draft.start_node_id === nodeId,
              terminal: flowMode(draft.nodes[nodeId]) === 'end',
            }"
            @click="selectNode(nodeId)"
          >
            <span class="node-index">{{ index + 1 }}</span>
            <span class="node-content">
              <span class="node-heading">
                <b>{{ nodeId }}</b>
                <em v-if="draft.start_node_id === nodeId">{{ t('dialogue.start', 'Start') }}</em>
                <em v-if="draft.nodes[nodeId].use_llm" class="llm">LLM</em>
                <em v-if="flowMode(draft.nodes[nodeId]) === 'end'" class="end">{{ t('dialogue.end', 'End') }}</em>
                <small>{{ draft.nodes[nodeId].speaker_id || t('dialogue.narrator', 'Narrator') }}</small>
              </span>
              <span class="node-text">{{ truncate(draft.nodes[nodeId].text, 150) || t('dialogue.empty-node', 'Empty node') }}</span>
              <span class="node-flow">
                <template v-if="draft.nodes[nodeId].next_node_id">
                  {{ t('dialogue.next-label', 'Next') }} · {{ draft.nodes[nodeId].next_node_id }}
                </template>
                <template v-else-if="draft.nodes[nodeId].choices.length">
                  {{ t('dialogue.choices-count', '{count} choices', { count: draft.nodes[nodeId].choices.length }) }} ·
                  {{ draft.nodes[nodeId].choices.map((choice) => choice.next_node_id || t('authoring.missing', 'Missing')).join(', ') }}
                </template>
                <template v-else>{{ t('dialogue.no-transition', 'No outgoing transition') }}</template>
              </span>
            </span>
          </button>
        </div>

        <footer class="graph-footer">
          <div>
            <strong>{{ selectedEntry?.access.unlock_event_ids.length || 0 }}</strong>
            <span>{{ t('authoring.unlock-events', 'unlock events') }}</span>
          </div>
          <div>
            <strong>{{ terminalCount }}</strong>
            <span>{{ t('dialogue.terminal-nodes', 'terminal nodes') }}</span>
          </div>
          <div>
            <strong>{{ selectedEntry?.content_fingerprint.slice(0, 10) || t('authoring.draft', 'draft') }}</strong>
            <span>{{ t('dialogue.content', 'content') }}</span>
          </div>
        </footer>
      </section>

      <section v-else class="empty-editor">
        <span class="empty-mark">DL</span>
        <h2>{{ t('dialogue.no-selection', 'No dialogue selected') }}</h2>
      </section>

      <aside v-if="draft" class="property-panel" :aria-label="t('dialogue.properties-aria', 'Dialogue properties')">
        <div class="property-tabs" role="tablist" :aria-label="t('dialogue.property-scope', 'Property scope')">
          <button :class="{ active: propertyTab === 'node' }" @click="propertyTab = 'node'">{{ t('dialogue.node', 'Node') }}</button>
          <button :class="{ active: propertyTab === 'script' }" @click="propertyTab = 'script'">{{ t('dialogue.script-tab', 'Script') }}</button>
        </div>

        <div v-if="propertyTab === 'script'" class="property-scroll">
          <section class="property-section">
            <span class="eyebrow">{{ t('authoring.identity', 'Identity') }}</span>
            <label class="form-field">
              <span>{{ t('dialogue.id', 'Dialogue ID') }}</span>
              <input v-model.trim="draft.id" class="input mono" :disabled="selectedDialogueId !== null" maxlength="128" />
            </label>
            <label class="form-field">
              <span>{{ t('ending.title-label', 'Title') }}</span>
              <input v-model="draft.title" class="input" maxlength="256" />
            </label>
            <label class="form-field">
              <span>{{ t('common.description', 'Description') }}</span>
              <textarea v-model="draft.description" class="input" rows="4" maxlength="2048"></textarea>
            </label>
          </section>

          <section class="property-section">
            <span class="eyebrow">{{ t('dialogue.graph', 'Graph') }}</span>
            <label class="form-field">
              <span>{{ t('ending.start-node', 'Start node') }}</span>
              <select v-model="draft.start_node_id" class="input">
                <option v-for="nodeId in Object.keys(draft.nodes)" :key="nodeId" :value="nodeId">{{ nodeId }}</option>
              </select>
            </label>
            <label class="form-field">
              <span>{{ t('dialogue.variables-json', 'Variables JSON') }}</span>
              <textarea v-model="variablesText" class="input mono variables-input" rows="9" spellcheck="false"></textarea>
              <small>{{ parsedVariables ? t('dialogue.variables-count', '{count} variables', { count: Object.keys(parsedVariables).length }) : t('dialogue.invalid-json', 'Invalid JSON object') }}</small>
            </label>
          </section>

          <section class="property-section access-section">
            <span class="eyebrow">{{ t('authoring.runtime-access', 'Runtime Access') }}</span>
            <div class="metric-row"><span>{{ t('common.status', 'Status') }}</span><strong>{{ accessStatus }}</strong></div>
            <div class="metric-row"><span>{{ t('dialogue.events', 'Events') }}</span><strong>{{ selectedEntry?.access.unlock_event_ids.length || 0 }}</strong></div>
            <p v-if="selectedEntry?.access.unlock_event_ids.length">{{ selectedEntry.access.unlock_event_ids.join(', ') }}</p>
          </section>
        </div>

        <div v-else-if="selectedNode" class="property-scroll">
          <section class="property-section identity-section">
            <div class="section-heading">
              <span class="eyebrow">{{ t('dialogue.node-identity', 'Node Identity') }}</span>
              <button class="text-button" :disabled="draft.start_node_id === selectedNodeId" @click="setStartNode">{{ t('dialogue.set-start', 'Set Start') }}</button>
            </div>
            <div class="rename-row">
              <input v-model.trim="nodeIdInput" class="input mono" maxlength="128" />
              <button class="btn btn-secondary btn-sm" :disabled="nodeIdInput === selectedNodeId" @click="renameNode">{{ t('dialogue.rename', 'Rename') }}</button>
            </div>
            <div class="field-grid">
              <label class="form-field">
                <span>{{ t('dialogue.speaker', 'Speaker') }}</span>
                <select v-model="selectedNode.speaker_id" class="input">
                  <option :value="null">{{ t('dialogue.narrator', 'Narrator') }}</option>
                  <option v-for="character in characters" :key="character.id" :value="character.id">
                    {{ character.name }} · {{ character.id }}
                  </option>
                </select>
              </label>
              <label class="form-field">
                <span>{{ t('dialogue.emotion', 'Emotion') }}</span>
                <input v-model="selectedNode.emotion" class="input" maxlength="64" placeholder="neutral" />
              </label>
              <label class="form-field">
                <span>{{ t('dialogue.scene', 'Scene') }}</span>
                <select v-model="selectedNode.scene_id" class="input">
                  <option :value="null">{{ t('dialogue.keep-scene', 'Keep current scene') }}</option>
                  <option v-for="scene in scenes" :key="scene.id" :value="scene.id">
                    {{ scene.name }} · {{ scene.id }}
                  </option>
                </select>
              </label>
            </div>
            <label class="form-field">
              <span>{{ t('dialogue.dialogue-text', 'Dialogue text') }}</span>
              <textarea v-model="selectedNode.text" class="input node-textarea" rows="6" maxlength="16384"></textarea>
              <small>{{ selectedNode.text.length }} / 16384</small>
            </label>
          </section>

          <section class="property-section">
            <span class="eyebrow">{{ t('dialogue.flow', 'Flow') }}</span>
            <div class="segmented-control" role="group" :aria-label="t('dialogue.flow-mode', 'Node flow mode')">
              <button :class="{ active: flowMode(selectedNode) === 'linear' }" @click="setFlowMode('linear')">{{ t('dialogue.linear', 'Linear') }}</button>
              <button :class="{ active: flowMode(selectedNode) === 'choices' }" @click="setFlowMode('choices')">{{ t('dialogue.choices', 'Choices') }}</button>
              <button :class="{ active: flowMode(selectedNode) === 'end' }" @click="setFlowMode('end')">{{ t('dialogue.end', 'End') }}</button>
            </div>
            <label v-if="flowMode(selectedNode) === 'linear'" class="form-field">
              <span>{{ t('dialogue.next', 'Next node') }}</span>
              <select v-model="selectedNode.next_node_id" class="input">
                <option :value="null">{{ t('dialogue.select-target', 'Select target') }}</option>
                <option v-for="nodeId in targetNodeIds" :key="nodeId" :value="nodeId">{{ nodeId }}</option>
              </select>
            </label>
            <template v-else-if="flowMode(selectedNode) === 'end'">
              <label class="check-field">
                <input v-model="selectedNode.is_ending" type="checkbox" />
                <span>{{ t('dialogue.mark-ending', 'Mark as authored ending') }}</span>
              </label>
              <label v-if="selectedNode.is_ending" class="form-field">
                <span>{{ t('dialogue.ending-type', 'Ending type') }}</span>
                <input v-model="selectedNode.ending_type" class="input" maxlength="64" placeholder="good" />
              </label>
            </template>
          </section>

          <section v-if="flowMode(selectedNode) === 'choices'" class="property-section choices-section">
            <div class="section-heading">
              <span class="eyebrow">{{ t('dialogue.choices-heading', 'Choices · {count}', { count: selectedNode.choices.length }) }}</span>
              <button class="text-button" :disabled="selectedNode.choices.length >= 32" @click="addChoice">{{ t('dialogue.add-choice', 'Add Choice') }}</button>
            </div>
            <article v-for="(choice, choiceIndex) in selectedNode.choices" :key="choiceIndex" class="choice-editor">
              <div class="choice-heading">
                <strong>{{ t('dialogue.choice-number', 'Choice {count}', { count: choiceIndex + 1 }) }}</strong>
                <button class="text-button danger" @click="removeChoice(choiceIndex)">{{ t('dialogue.remove', 'Remove') }}</button>
              </div>
              <label class="form-field">
                <span>{{ t('dialogue.text', 'Text') }}</span>
                <textarea v-model="choice.text" class="input" rows="3" maxlength="2048"></textarea>
              </label>
              <label class="form-field">
                <span>{{ t('dialogue.target-node', 'Target node') }}</span>
                <select v-model="choice.next_node_id" class="input">
                  <option value="">{{ t('dialogue.select-target', 'Select target') }}</option>
                  <option v-for="nodeId in targetNodeIds" :key="nodeId" :value="nodeId">{{ nodeId }}</option>
                </select>
              </label>
              <label class="form-field">
                <span>{{ t('dialogue.condition', 'Condition') }}</span>
                <input v-model="choice.condition" class="input mono" maxlength="2000" placeholder="hasFlag(&quot;route_open&quot;)" />
              </label>
              <div class="relationship-editor">
                <span class="field-label">{{ t('dialogue.relationship-changes', 'Relationship changes') }}</span>
                <div
                  v-for="([characterId, delta]) in relationshipEntries(choice)"
                  :key="characterId"
                  class="relationship-row"
                >
                  <select :value="characterId" class="input" @change="renameRelationship(choice, characterId, $event)">
                    <option v-for="character in characters" :key="character.id" :value="character.id">{{ character.name }}</option>
                  </select>
                  <input
                    :value="delta"
                    class="input delta-input"
                    type="number"
                    min="-1"
                    max="1"
                    step="0.05"
                    @input="setRelationshipDelta(choice, characterId, $event)"
                  />
                  <button class="remove-symbol" :title="t('dialogue.remove-relationship', 'Remove relationship change')" @click="removeRelationship(choice, characterId)"><Trash2 :size="13" /></button>
                </div>
                <button class="btn btn-secondary btn-sm" :disabled="availableRelationshipCharacters(choice).length === 0" @click="addRelationship(choice)">
                  {{ t('dialogue.add-relationship', 'Add Relationship') }}
                </button>
              </div>
            </article>
          </section>

          <section class="property-section">
            <span class="eyebrow">{{ t('dialogue.logic', 'Logic') }}</span>
            <label class="form-field">
              <span>{{ t('dialogue.node-condition', 'Node condition') }}</span>
              <input v-model="selectedNode.condition" class="input mono" maxlength="2000" placeholder="hasFlag(&quot;chapter_open&quot;)" />
            </label>
            <label class="form-field">
              <span>{{ t('dialogue.entry-script', 'Entry script') }}</span>
              <textarea v-model="selectedNode.script" class="input mono" rows="3" maxlength="20000" placeholder="setFlag('visited', true)"></textarea>
            </label>
          </section>

          <section class="property-section">
            <span class="eyebrow">{{ t('dialogue.llm-generation', 'LLM Generation') }}</span>
            <label class="check-field">
              <input v-model="selectedNode.use_llm" type="checkbox" />
              <span>{{ t('dialogue.generate-runtime', 'Generate node text at runtime') }}</span>
            </label>
            <template v-if="selectedNode.use_llm">
              <label class="form-field">
                <span>{{ t('dialogue.prompt', 'Prompt') }}</span>
                <textarea v-model="selectedNode.llm_prompt" class="input" rows="4" maxlength="20000"></textarea>
              </label>
              <label class="form-field">
                <span>{{ t('dialogue.system-prompt', 'System prompt override') }}</span>
                <textarea v-model="selectedNode.llm_system_prompt" class="input" rows="4" maxlength="20000"></textarea>
              </label>
            </template>
          </section>

          <section class="property-section danger-section">
            <button class="btn btn-danger btn-sm" :disabled="Object.keys(draft.nodes).length <= 1" @click="deleteNode"><Trash2 :size="14" />{{ t('dialogue.delete-node', 'Delete Node') }}</button>
          </section>
        </div>

        <div v-else class="empty-properties">{{ t('dialogue.select-node', 'Select a node') }}</div>

        <footer class="property-footer">
          <button class="btn btn-danger btn-sm" :disabled="!selectedEntry || busy" @click="removeDialogue"><Trash2 :size="14" />{{ t('dialogue.delete-dialogue', 'Delete Dialogue') }}</button>
          <button class="btn btn-primary" :disabled="!canSave || busy" @click="saveDialogue"><Save :size="15" />{{ t('dialogue.save-dialogue', 'Save Dialogue') }}</button>
        </footer>
      </aside>
    </main>

    <Transition name="fade">
      <button v-if="notice" class="notice" :class="notice.type" @click="notice = null">
        <strong>{{ notice.title }}</strong>
        <span>{{ notice.message }}</span>
      </button>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { onBeforeRouteLeave, useRouter } from 'vue-router'
import { Copy, Play, Plus, RotateCcw, Save, Trash2 } from '@lucide/vue'
import {
  deleteDialogueDefinition,
  loadDialogueAuthoringCatalog,
  normalizeDialogueDefinition,
  saveDialogueDefinition,
  validateDialogueDefinition,
  type DialogueAuthoringCatalogSnapshot,
  type DialogueAuthoringEntry,
  type DialogueChoiceDefinition,
  type DialogueDefinition,
} from '../lib/dialogueAuthoring'
import {
  addDialogueRelationship as addRelationshipToChoice,
  appendDialogueChoice,
  appendDialogueNode,
  availableDialogueRelationshipCharacters,
  cloneDialogueDefinition,
  createDialogueDraft,
  deleteDialogueNode as deleteNodeFromGraph,
  dialogueDefinitionFromEntry,
  dialogueDraftSnapshot,
  dialogueFlowMode as flowMode,
  dialogueImplicitTerminalIds,
  dialogueNodeOrder as orderDialogueNodes,
  dialogueRelationshipEntries as relationshipEntries,
  dialogueTargetNodeIds,
  dialogueTerminalCount,
  duplicateDialogueDraft,
  filterDialogueEntries,
  hasDialogueIdCollision,
  mergeDialogueCharacters,
  parseDialogueVariables,
  removeDialogueChoice,
  removeDialogueRelationship as removeRelationshipFromChoice,
  renameDialogueNode,
  renameDialogueRelationship as renameRelationshipInChoice,
  setDialogueNodeFlowMode,
  setDialogueRelationshipDelta,
  type DialogueCharacterIdentity,
  type DialogueFlowMode,
} from '../lib/dialogueGraphEditing'
import { hasTauriRuntime, invokeCommand } from '../lib/tauri'
import { useI18n } from '../lib/i18n'
import { loadStoryScenes, type StorySceneInfo } from '../lib/storyContent'

type CharacterInfo = DialogueCharacterIdentity

const router = useRouter()
const { t } = useI18n()
const snapshot = ref<DialogueAuthoringCatalogSnapshot | null>(null)
const draft = ref<DialogueDefinition | null>(null)
const selectedDialogueId = ref<string | null>(null)
const selectedNodeId = ref<string | null>(null)
const nodeIdInput = ref('')
const variablesText = ref('{}')
const baseline = ref('')
const search = ref('')
const propertyTab = ref<'node' | 'script'>('node')
const characters = ref<CharacterInfo[]>([])
const scenes = ref<StorySceneInfo[]>([])
const busy = ref(false)
const notice = ref<{ type: 'success' | 'error'; title: string; message: string } | null>(null)

const filteredDialogues = computed(() => filterDialogueEntries(snapshot.value?.dialogues || [], search.value))
const selectedEntry = computed(() => (snapshot.value?.dialogues || [])
  .find((dialogue) => dialogue.id === selectedDialogueId.value) || null)
const selectedNode = computed(() => selectedNodeId.value && draft.value
  ? draft.value.nodes[selectedNodeId.value] || null
  : null)
const serializedDraft = computed(() => dialogueDraftSnapshot(draft.value, variablesText.value))
const dirty = computed(() => serializedDraft.value !== baseline.value)
const parsedVariables = computed<Record<string, unknown> | null>(() => parseDialogueVariables(variablesText.value))
const validationIssues = computed(() => {
  if (!draft.value) return [t('dialogue.error.no-selection', 'No dialogue selected.')]
  if (!parsedVariables.value) return [t('dialogue.error.variables-json', 'Variables must be a valid JSON object.')]
  const candidate = { ...draft.value, variables: parsedVariables.value }
  const issues = validateDialogueDefinition(
    candidate,
    characters.value.map((character) => character.id),
    scenes.value.map((scene) => scene.id),
  )
  if (!selectedDialogueId.value && hasDialogueIdCollision(
    snapshot.value?.dialogues.map((dialogue) => dialogue.id) || [],
    candidate.id,
  )) {
    issues.push(t('dialogue.error.already-exists', 'Dialogue "{id}" already exists.', { id: candidate.id.trim() }))
  }
  return issues
})
const warnings = computed(() => {
  if (!draft.value) return []
  const implicitTerminals = dialogueImplicitTerminalIds(draft.value)
  return implicitTerminals.length > 0
    ? [t('dialogue.warning.implicit-terminals', 'Terminal nodes without an ending marker: {nodes}.', { nodes: implicitTerminals.join(', ') })]
    : []
})
const nodeOrder = computed(() => draft.value ? orderDialogueNodes(draft.value) : [])
const targetNodeIds = computed(() => draft.value ? dialogueTargetNodeIds(draft.value) : [])
const terminalCount = computed(() => draft.value ? dialogueTerminalCount(draft.value) : 0)
const gatedCount = computed(() => (snapshot.value?.dialogues || []).filter((dialogue) => dialogue.access.gated).length)
const sourcePath = computed(() => selectedEntry.value?.source_path || `dialogue/${draft.value?.id || 'new'}.json`)
const accessStatus = computed(() => {
  const access = selectedEntry.value?.access
  if (!access) return t('authoring.draft', 'Draft')
  if (!access.gated) return t('authoring.open', 'Open')
  return access.unlocked ? t('authoring.unlocked', 'Unlocked') : t('authoring.locked', 'Locked')
})
const canSave = computed(() => Boolean(draft.value && snapshot.value && dirty.value && validationIssues.value.length === 0))
const canPreview = computed(() => Boolean(selectedDialogueId.value && !dirty.value && validationIssues.value.length === 0))

function truncate(value: string, length: number): string {
  return value.length > length ? `${value.slice(0, length)}...` : value
}

function setDraft(definition: DialogueDefinition, dialogueId: string | null, isSaved = true) {
  const normalized = normalizeDialogueDefinition(cloneDialogueDefinition(definition))
  draft.value = normalized
  selectedDialogueId.value = dialogueId
  selectedNodeId.value = normalized.start_node_id || Object.keys(normalized.nodes)[0] || null
  nodeIdInput.value = selectedNodeId.value || ''
  variablesText.value = JSON.stringify(normalized.variables, null, 2)
  baseline.value = isSaved ? dialogueDraftSnapshot(draft.value, variablesText.value) : ''
  propertyTab.value = 'node'
}

function confirmDiscard(): boolean {
  return !dirty.value || window.confirm(t('dialogue.confirm.discard', 'Discard unsaved dialogue changes?'))
}

function selectDialogue(entry: DialogueAuthoringEntry) {
  if (entry.id === selectedDialogueId.value) return
  if (!confirmDiscard()) return
  setDraft(dialogueDefinitionFromEntry(entry), entry.id)
}

function selectNode(nodeId: string) {
  selectedNodeId.value = nodeId
  nodeIdInput.value = nodeId
  propertyTab.value = 'node'
}

function createDialogue() {
  if (!confirmDiscard()) return
  const speaker = characters.value[0]?.id || null
  setDraft(createDialogueDraft(
    snapshot.value?.dialogues.map((dialogue) => dialogue.id) || [],
    t('dialogue.new-dialogue', 'New Dialogue'),
    t('dialogue.new-line', 'New dialogue line.'),
    speaker,
  ), null, false)
}

function duplicateDialogue() {
  if (!draft.value || !confirmDiscard()) return
  const copy = duplicateDialogueDraft(
    draft.value,
    snapshot.value?.dialogues.map((dialogue) => dialogue.id) || [],
    t('authoring.copy-name', '{name} Copy', { name: draft.value.title }),
  )
  setDraft(copy, null, false)
}

function addNode() {
  if (!draft.value) return
  const result = appendDialogueNode(draft.value, selectedNode.value?.speaker_id || characters.value[0]?.id || null)
  draft.value = result.dialogue
  selectNode(result.node_id)
}

function renameNode() {
  if (!draft.value || !selectedNodeId.value) return
  const result = renameDialogueNode(draft.value, selectedNodeId.value, nodeIdInput.value)
  if (result.error === 'invalid_id') {
    showNotice('error', t('dialogue.notice.rename-rejected', 'Rename rejected'), t('dialogue.error.node-id', 'Node ID must be a portable 1-128 character ID.'))
    return
  }
  if (result.error === 'node_exists') {
    showNotice('error', t('dialogue.notice.rename-rejected', 'Rename rejected'), t('dialogue.error.node-exists', 'Node "{id}" already exists.', { id: nodeIdInput.value.trim() }))
    return
  }
  if (!result.changed) return
  draft.value = result.dialogue
  selectedNodeId.value = result.node_id
  nodeIdInput.value = result.node_id
}

function deleteNode() {
  if (!draft.value || !selectedNodeId.value) return
  const result = deleteNodeFromGraph(draft.value, selectedNodeId.value)
  if (result.error === 'node_referenced') {
    showNotice('error', t('dialogue.notice.node-referenced', 'Node is referenced'), t('dialogue.error.remove-transitions', 'Remove transitions from: {nodes}.', { nodes: result.references.join(', ') }))
    return
  }
  if (result.error === 'start_node') {
    showNotice('error', t('dialogue.notice.start-protected', 'Start node protected'), t('dialogue.error.choose-start', 'Choose another start node before deleting this node.'))
    return
  }
  if (!result.changed || !result.selected_node_id) return
  draft.value = result.dialogue
  selectNode(result.selected_node_id)
}

function setStartNode() {
  if (draft.value && selectedNodeId.value) draft.value.start_node_id = selectedNodeId.value
}

function setFlowMode(mode: DialogueFlowMode) {
  if (!selectedNode.value || !selectedNodeId.value) return
  Object.assign(selectedNode.value, setDialogueNodeFlowMode(
    selectedNode.value,
    selectedNodeId.value,
    mode,
    targetNodeIds.value,
    t('dialogue.new-choice', 'New choice'),
  ))
}

function addChoice() {
  if (!selectedNode.value || !selectedNodeId.value) return
  Object.assign(selectedNode.value, appendDialogueChoice(
    selectedNode.value,
    selectedNodeId.value,
    targetNodeIds.value,
    t('dialogue.new-choice', 'New choice'),
  ))
}

function removeChoice(index: number) {
  if (selectedNode.value) Object.assign(selectedNode.value, removeDialogueChoice(selectedNode.value, index))
}

function availableRelationshipCharacters(choice: DialogueChoiceDefinition): CharacterInfo[] {
  return availableDialogueRelationshipCharacters(choice, characters.value)
}

function addRelationship(choice: DialogueChoiceDefinition) {
  const character = availableRelationshipCharacters(choice)[0]
  if (character) Object.assign(choice, addRelationshipToChoice(choice, character.id))
}

function removeRelationship(choice: DialogueChoiceDefinition, characterId: string) {
  Object.assign(choice, removeRelationshipFromChoice(choice, characterId))
}

function renameRelationship(choice: DialogueChoiceDefinition, before: string, event: Event) {
  const after = (event.target as HTMLSelectElement).value
  Object.assign(choice, renameRelationshipInChoice(choice, before, after))
}

function setRelationshipDelta(choice: DialogueChoiceDefinition, characterId: string, event: Event) {
  Object.assign(choice, setDialogueRelationshipDelta(choice, characterId, (event.target as HTMLInputElement).value))
}

async function loadCatalog(preferredId?: string | null) {
  busy.value = true
  try {
    const [nextSnapshot, projectCharacters, projectScenes] = await Promise.all([
      loadDialogueAuthoringCatalog(),
      invokeCommand<CharacterInfo[]>('get_characters', undefined, []),
      loadStoryScenes(),
    ])
    snapshot.value = nextSnapshot
    characters.value = mergeDialogueCharacters(nextSnapshot, projectCharacters)
    scenes.value = projectScenes
    const target = nextSnapshot.dialogues.find((dialogue) => dialogue.id === preferredId) || nextSnapshot.dialogues[0]
    if (target) setDraft(dialogueDefinitionFromEntry(target), target.id)
    else {
      draft.value = null
      selectedDialogueId.value = null
      selectedNodeId.value = null
      baseline.value = ''
    }
  } catch (error) {
    showNotice('error', t('authoring.catalog-unavailable', 'Catalog unavailable'), String(error))
  } finally {
    busy.value = false
  }
}

async function reloadCatalog() {
  if (!confirmDiscard()) return
  await loadCatalog(selectedDialogueId.value)
  showNotice('success', t('authoring.catalog-reloaded', 'Catalog reloaded'), t('dialogue.notice.reloaded', 'Dialogue graphs and character references are current.'))
}

async function saveDialogue() {
  if (!draft.value || !snapshot.value || !parsedVariables.value || !canSave.value) return
  busy.value = true
  try {
    const wasExisting = selectedDialogueId.value !== null
    const dialogue = normalizeDialogueDefinition({ ...cloneDialogueDefinition(draft.value), variables: parsedVariables.value })
    const next = await saveDialogueDefinition(
      dialogue,
      selectedDialogueId.value,
      snapshot.value.catalog_fingerprint,
      characters.value.map((character) => character.id),
      scenes.value.map((scene) => scene.id),
    )
    snapshot.value = next
    const saved = next.dialogues.find((entry) => entry.id === dialogue.id)
    if (saved) setDraft(dialogueDefinitionFromEntry(saved), saved.id)
    showNotice('success', wasExisting ? t('dialogue.notice.saved-title', 'Dialogue saved') : t('dialogue.notice.created-title', 'Dialogue created'), t('dialogue.notice.saved-message', '{title} passed graph and project validation.', { title: dialogue.title }))
  } catch (error) {
    showNotice('error', t('authoring.save-rejected', 'Save rejected'), String(error))
  } finally {
    busy.value = false
  }
}

async function removeDialogue() {
  if (!selectedDialogueId.value || !snapshot.value) return
  const dialogueId = selectedDialogueId.value
  if (!window.confirm(t('dialogue.confirm.delete', 'Delete dialogue "{id}"?', { id: dialogueId }))) return
  busy.value = true
  try {
    const next = await deleteDialogueDefinition(dialogueId, snapshot.value.catalog_fingerprint)
    snapshot.value = next
    const target = next.dialogues[0]
    if (target) setDraft(dialogueDefinitionFromEntry(target), target.id)
    else {
      draft.value = null
      selectedDialogueId.value = null
      selectedNodeId.value = null
      baseline.value = ''
    }
    showNotice('success', t('dialogue.notice.deleted-title', 'Dialogue deleted'), t('dialogue.notice.deleted-message', '{id} was removed and the runtime catalog was reloaded.', { id: dialogueId }))
  } catch (error) {
    showNotice('error', t('authoring.delete-rejected', 'Delete rejected'), String(error))
  } finally {
    busy.value = false
  }
}

async function previewDialogue() {
  if (!selectedDialogueId.value || !canPreview.value) return
  busy.value = true
  try {
    if (hasTauriRuntime()) {
      await invokeCommand('preview_dialogue', { dialogueId: selectedDialogueId.value })
      await router.push('/game')
    } else {
      await router.push({ path: '/game', query: { previewDialogue: selectedDialogueId.value, authoring: '1' } })
    }
  } catch (error) {
    showNotice('error', t('authoring.preview-unavailable', 'Preview unavailable'), String(error))
  } finally {
    busy.value = false
  }
}

function showNotice(type: 'success' | 'error', title: string, message: string) {
  notice.value = { type, title, message }
}

function handleBeforeUnload(event: BeforeUnloadEvent) {
  if (!dirty.value) return
  event.preventDefault()
  event.returnValue = ''
}

function handleKeydown(event: KeyboardEvent) {
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 's') {
    event.preventDefault()
    void saveDialogue()
  }
}

onBeforeRouteLeave(() => confirmDiscard())

onMounted(async () => {
  window.addEventListener('beforeunload', handleBeforeUnload)
  window.addEventListener('keydown', handleKeydown)
  await loadCatalog()
})

onUnmounted(() => {
  window.removeEventListener('beforeunload', handleBeforeUnload)
  window.removeEventListener('keydown', handleKeydown)
})
</script>

<style scoped>
.dialogue-editor {
  height: 100vh;
  height: 100svh;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--surface-0);
  color: var(--text-primary);
}
.editor-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 20px;
  padding: 21px 26px 15px;
  border-bottom: 1px solid var(--border);
  background: var(--surface-1);
}
.header-copy { min-width: 0; }
.header-copy h1 { margin-top: 3px; font-size: 25px; line-height: 1.15; }
.header-copy p { margin-top: 5px; color: var(--text-tertiary); font-size: 12px; }
.header-actions { display: flex; flex-wrap: wrap; justify-content: flex-end; gap: 8px; }
.eyebrow { display: block; color: var(--text-tertiary); font-size: 10px; font-weight: 800; letter-spacing: 0; text-transform: uppercase; }
.catalog-strip {
  min-height: 40px;
  display: flex;
  align-items: center;
  gap: 20px;
  padding: 7px 26px;
  border-bottom: 1px solid var(--border);
  color: var(--text-tertiary);
  font-size: 11px;
}
.catalog-strip strong { color: var(--text-secondary); font-family: var(--font-mono); }
.dirty-indicator { margin-left: auto; color: var(--warning); font-weight: 800; }
.editor-workspace { flex: 1; min-height: 0; display: grid; grid-template-columns: 250px minmax(380px, 1fr) 390px; }

.dialogue-list { min-height: 0; display: flex; flex-direction: column; border-right: 1px solid var(--border); background: var(--surface-1); }
.list-toolbar { min-height: 58px; display: flex; align-items: center; gap: 9px; padding: 10px; border-bottom: 1px solid var(--border); color: var(--text-tertiary); font-size: 11px; }
.search-field { flex: 1; min-width: 0; }
.dialogue-items { min-height: 0; overflow-y: auto; padding: 7px; }
.dialogue-item {
  width: 100%;
  min-height: 72px;
  display: grid;
  grid-template-columns: 32px minmax(0, 1fr);
  gap: 9px;
  align-items: start;
  padding: 9px;
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-primary);
  text-align: left;
  cursor: pointer;
}
.dialogue-item:hover { background: var(--surface-2); border-color: var(--border); }
.dialogue-item.active { background: var(--surface-3); border-color: var(--brand); }
.dialogue-mark { width: 32px; height: 32px; display: grid; place-items: center; border: 1px solid var(--border); border-radius: 4px; color: var(--narrative); font-family: var(--font-mono); font-size: 10px; font-weight: 900; }
.dialogue-item-copy { min-width: 0; display: block; }
.dialogue-item-copy strong,
.dialogue-item-copy small,
.dialogue-item-copy > span { display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.dialogue-item-copy strong { font-size: 12px; }
.dialogue-item-copy small { margin-top: 1px; color: var(--text-tertiary); font-family: var(--font-mono); font-size: 9px; }
.dialogue-item-copy > span { margin-top: 5px; color: var(--text-secondary); font-size: 9px; }
.empty-list { padding: 28px 12px; color: var(--text-tertiary); text-align: center; }

.graph-panel { min-width: 0; min-height: 0; display: flex; flex-direction: column; }
.graph-header { min-height: 92px; display: flex; align-items: center; justify-content: space-between; gap: 18px; padding: 15px 20px; border-bottom: 1px solid var(--border); background: var(--surface-1); }
.graph-title { min-width: 0; }
.graph-title .eyebrow { overflow: hidden; font-family: var(--font-mono); text-overflow: ellipsis; white-space: nowrap; text-transform: none; }
.graph-title h2 { margin-top: 3px; overflow: hidden; font-size: 18px; line-height: 1.2; text-overflow: ellipsis; white-space: nowrap; }
.graph-title p { margin-top: 4px; overflow: hidden; color: var(--text-tertiary); font-size: 10px; text-overflow: ellipsis; white-space: nowrap; }
.graph-actions { display: flex; align-items: center; gap: 9px; flex-shrink: 0; }
.graph-state { padding: 3px 8px; border-radius: 4px; font-size: 9px; font-weight: 800; }
.graph-state.valid { background: rgba(34, 197, 94, 0.12); color: #86efac; }
.graph-state.invalid { background: rgba(239, 68, 68, 0.12); color: #fca5a5; }
.validation-banner { min-height: 44px; display: grid; grid-template-columns: auto minmax(0, 1fr); align-items: center; gap: 12px; padding: 8px 18px; border-bottom: 1px solid var(--border); font-size: 10px; }
.validation-banner span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.validation-banner.error { background: rgba(239, 68, 68, 0.08); color: #fca5a5; }
.validation-banner.warning { background: rgba(245, 158, 11, 0.08); color: #fcd34d; }
.validation-banner.valid { background: rgba(34, 197, 94, 0.07); color: #86efac; }
.graph-scroll { position: relative; flex: 1; min-height: 0; overflow-y: auto; padding: 18px 22px 28px 44px; }
.route-rail { position: absolute; top: 20px; bottom: 26px; left: 28px; width: 1px; background: var(--border-light); }
.node-card {
  position: relative;
  width: 100%;
  min-height: 104px;
  display: grid;
  grid-template-columns: 34px minmax(0, 1fr);
  gap: 12px;
  margin-bottom: 12px;
  padding: 13px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-1);
  color: var(--text-primary);
  text-align: left;
  cursor: pointer;
}
.node-card:hover { border-color: var(--border-light); background: var(--surface-2); }
.node-card.selected { border-color: var(--brand); box-shadow: var(--shadow-brand); }
.node-card.start { border-left: 3px solid var(--success); }
.node-card.terminal { border-right: 3px solid var(--narrative); }
.node-index { width: 30px; height: 30px; display: grid; place-items: center; border: 1px solid var(--border); border-radius: 50%; background: var(--surface-2); color: var(--text-tertiary); font-family: var(--font-mono); font-size: 10px; font-weight: 800; }
.node-content { min-width: 0; display: block; }
.node-heading { display: flex; align-items: center; gap: 6px; min-width: 0; }
.node-heading b { overflow: hidden; font-family: var(--font-mono); font-size: 11px; text-overflow: ellipsis; white-space: nowrap; }
.node-heading em { padding: 2px 5px; border-radius: 3px; background: rgba(34, 197, 94, 0.12); color: #86efac; font-size: 8px; font-style: normal; font-weight: 900; text-transform: uppercase; }
.node-heading em.llm { background: rgba(96, 165, 250, 0.14); color: #93c5fd; }
.node-heading em.end { background: rgba(192, 132, 252, 0.14); color: #d8b4fe; }
.node-heading small { margin-left: auto; color: var(--brand-light); font-size: 9px; }
.node-text { display: -webkit-box; margin-top: 8px; overflow: hidden; color: var(--text-secondary); font-size: 11px; line-height: 1.5; -webkit-box-orient: vertical; -webkit-line-clamp: 2; }
.node-flow { display: block; margin-top: 8px; overflow: hidden; color: var(--text-tertiary); font-family: var(--font-mono); font-size: 9px; text-overflow: ellipsis; white-space: nowrap; }
.graph-footer { min-height: 58px; display: grid; grid-template-columns: repeat(3, 1fr); border-top: 1px solid var(--border); background: var(--surface-1); }
.graph-footer div { display: grid; place-content: center; border-right: 1px solid var(--border); text-align: center; }
.graph-footer strong { color: var(--text-secondary); font-family: var(--font-mono); font-size: 10px; }
.graph-footer span { color: var(--text-tertiary); font-size: 8px; text-transform: uppercase; }

.property-panel { min-width: 0; min-height: 0; display: flex; flex-direction: column; border-left: 1px solid var(--border); background: var(--surface-1); }
.property-tabs { min-height: 48px; display: grid; grid-template-columns: 1fr 1fr; padding: 7px; border-bottom: 1px solid var(--border); }
.property-tabs button,
.segmented-control button { border: 1px solid transparent; border-radius: 4px; background: transparent; color: var(--text-tertiary); font-size: 11px; font-weight: 800; cursor: pointer; }
.property-tabs button.active,
.segmented-control button.active { border-color: var(--border-light); background: var(--surface-3); color: var(--text-primary); }
.property-scroll { flex: 1; min-height: 0; overflow-y: auto; }
.property-section { display: grid; gap: 12px; padding: 17px 16px 19px; border-bottom: 1px solid var(--border); }
.section-heading { display: flex; align-items: center; justify-content: space-between; gap: 12px; }
.form-field { min-width: 0; display: grid; gap: 5px; }
.form-field > span,
.field-label { color: var(--text-secondary); font-size: 10px; font-weight: 800; }
.form-field small { color: var(--text-tertiary); font-size: 9px; }
.field-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }
.mono { font-family: var(--font-mono); }
.rename-row { display: grid; grid-template-columns: minmax(0, 1fr) auto; gap: 8px; }
.text-button { padding: 2px 0; border: none; background: transparent; color: var(--brand-light); font-size: 10px; font-weight: 800; cursor: pointer; }
.text-button:disabled { opacity: 0.42; cursor: not-allowed; }
.text-button.danger { color: #fca5a5; }
.node-textarea { min-height: 126px; }
.variables-input { min-height: 180px; resize: vertical; }
.segmented-control { height: 36px; display: grid; grid-template-columns: repeat(3, 1fr); gap: 4px; padding: 3px; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--surface-2); }
.check-field { display: flex; align-items: center; gap: 9px; color: var(--text-secondary); font-size: 11px; }
.check-field input { width: 16px; height: 16px; accent-color: var(--brand); }
.choices-section { gap: 10px; }
.choice-editor { display: grid; gap: 10px; padding: 12px; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--surface-2); }
.choice-heading { display: flex; align-items: center; justify-content: space-between; }
.choice-heading strong { font-size: 10px; }
.relationship-editor { display: grid; gap: 7px; padding-top: 2px; }
.relationship-row { display: grid; grid-template-columns: minmax(0, 1fr) 78px 26px; gap: 6px; }
.delta-input { text-align: right; }
.remove-symbol { width: 26px; height: 34px; border: 1px solid var(--border); border-radius: 4px; background: var(--surface-3); color: var(--text-tertiary); cursor: pointer; }
.remove-symbol:hover { border-color: var(--danger); color: #fca5a5; }
.access-section p { overflow-wrap: anywhere; color: var(--warning); font-family: var(--font-mono); font-size: 9px; }
.metric-row { display: flex; align-items: center; justify-content: space-between; color: var(--text-tertiary); font-size: 10px; }
.metric-row strong { color: var(--text-secondary); }
.danger-section { justify-items: start; }
.empty-properties { flex: 1; display: grid; place-content: center; color: var(--text-tertiary); font-size: 11px; }
.property-footer { min-height: 62px; display: flex; justify-content: flex-end; gap: 8px; padding: 11px 14px; border-top: 1px solid var(--border); background: var(--surface-2); }
.empty-editor { display: grid; place-content: center; gap: 10px; color: var(--text-tertiary); text-align: center; }
.empty-editor h2 { color: var(--text-secondary); font-size: 17px; }
.empty-mark { display: inline-grid; min-width: 44px; height: 44px; place-items: center; border: 1px solid var(--border); border-radius: var(--radius); background: var(--surface-2); color: var(--narrative); font-family: var(--font-mono); font-weight: 900; }
.sr-only { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; }
.input:disabled { opacity: 0.68; cursor: not-allowed; }

.notice { position: fixed; right: 22px; bottom: 22px; z-index: 80; width: min(370px, calc(100vw - 44px)); display: grid; gap: 3px; padding: 13px 15px; border: 1px solid var(--border-light); border-radius: var(--radius); background: var(--surface-3); color: var(--text-primary); box-shadow: var(--shadow-lg); text-align: left; cursor: pointer; }
.notice strong { font-size: 12px; }
.notice span { color: var(--text-secondary); font-size: 10px; }
.notice.success { border-color: rgba(34, 197, 94, 0.55); }
.notice.error { border-color: rgba(239, 68, 68, 0.65); }

@media (max-width: 1240px) {
  .editor-workspace { grid-template-columns: 225px minmax(390px, 1fr); grid-template-rows: minmax(0, 1fr) minmax(330px, 44vh); }
  .property-panel { grid-column: 1 / -1; display: grid; grid-template-columns: 1fr; border-top: 1px solid var(--border); border-left: none; }
  .property-tabs { width: 260px; }
  .property-scroll { border-top: 1px solid var(--border); }
  .property-footer { position: absolute; right: 12px; align-self: start; border: none; background: transparent; }
}

@media (max-width: 760px) {
  .dialogue-editor { height: auto; min-height: 100svh; overflow: visible; }
  .editor-header { flex-direction: column; padding: 18px 16px 14px; }
  .header-actions { width: 100%; justify-content: flex-start; }
  .catalog-strip { flex-wrap: wrap; gap: 8px 15px; padding: 7px 16px; }
  .dirty-indicator { width: 100%; margin-left: 0; }
  .editor-workspace { display: block; }
  .dialogue-list { height: 230px; border-right: none; border-bottom: 1px solid var(--border); }
  .graph-panel { min-height: 620px; }
  .graph-header { align-items: flex-start; }
  .graph-actions { align-items: flex-end; flex-direction: column; }
  .graph-scroll { min-height: 440px; padding-left: 32px; }
  .route-rail { left: 20px; }
  .property-panel { min-height: 680px; border-top: 1px solid var(--border); }
  .property-tabs { width: 100%; }
  .property-footer { position: static; }
}

@media (max-width: 480px) {
  .header-actions .btn { flex: 1 1 auto; justify-content: center; }
  .catalog-strip span:nth-child(4) { display: none; }
  .graph-header { flex-direction: column; }
  .graph-actions { width: 100%; align-items: center; flex-direction: row; justify-content: space-between; }
  .validation-banner { grid-template-columns: 1fr; gap: 2px; padding: 8px 14px; }
  .validation-banner span { white-space: normal; }
  .node-card { grid-template-columns: 28px minmax(0, 1fr); padding: 11px; }
  .node-index { width: 26px; height: 26px; }
  .node-heading small { display: none; }
  .field-grid { grid-template-columns: 1fr; }
  .relationship-row { grid-template-columns: minmax(0, 1fr) 72px 26px; }
  .notice { bottom: calc(74px + env(safe-area-inset-bottom, 0px)); }
}
</style>
