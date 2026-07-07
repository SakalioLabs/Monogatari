<template>
  <div class="dialogue-editor">
    <header class="toolbar">
      <div class="toolbar-left">
        <span class="eyebrow">Authoring</span>
        <h1>{{ t("dialogue.title", "Dialogue Editor") }}</h1>
        <span class="dialogue-name">{{ currentDialogue?.title || 'Untitled' }}</span>
      </div>
      <div class="toolbar-right">
        <button class="btn btn-secondary btn-sm" @click="newDialogue">New</button>
        <button class="btn btn-secondary btn-sm" @click="importDialogue">{{ t("dialogue.import", "Import") }}</button>
        <button class="btn btn-secondary btn-sm" @click="validate">{{ t("dialogue.validate", "Validate") }}</button>
        <button class="btn btn-primary btn-sm" @click="exportDialogue">{{ t("dialogue.export", "Export") }}</button>
      </div>
    </header>

    <main class="editor-body">
      <!-- Dialogue List Sidebar -->
      <aside class="dialogue-list">
        <div class="panel-title">
          <span class="eyebrow">Dialogues</span>
          <strong>{{ dialogues.length }}</strong>
        </div>
        <button
          v-for="dlg in dialogues"
          :key="dlg.id"
          class="dlg-item"
          :class="{ active: currentDialogue?.id === dlg.id }"
          @click="selectDialogue(dlg)"
        >
          <strong>{{ dlg.title || dlg.id }}</strong>
          <small>{{ Object.keys(dlg.nodes).length }} nodes</small>
        </button>
        <div v-if="dialogues.length === 0" class="empty-list">
          <span>No dialogues loaded</span>
        </div>
      </aside>

      <!-- Node Tree Canvas -->
      <section class="tree-canvas">
        <div v-if="!currentDialogue" class="empty-canvas">
          <span class="empty-mark">DE</span>
          <h2>Dialogue Editor</h2>
          <p>Select a dialogue from the list or create a new one. Build branching conversation trees with visual node editing.</p>
        </div>
        <div v-else class="node-tree">
          <div class="tree-header">
            <span class="eyebrow">Node Tree</span>
            <button class="btn btn-secondary btn-sm" @click="addNode">+ Add Node</button>
          </div>
          <div class="tree-scroll">
            <div
              v-for="(node, nodeId) in currentDialogue.nodes"
              :key="nodeId"
              class="tree-node"
              :class="{
                selected: selectedNodeId === nodeId,
                'is-start': nodeId === currentDialogue.start_node_id,
                'is-end': !node.choices || node.choices.length === 0
              }"
              @click="selectNode(nodeId as string)"
            >
              <div class="node-header">
                <span class="node-badge" :class="nodeId === currentDialogue.start_node_id ? 'start' : (!node.choices?.length ? 'end' : 'middle')">
                  {{ nodeId === currentDialogue.start_node_id ? 'START' : (!node.choices?.length ? 'END' : 'NODE') }}
                </span>
                <strong>{{ nodeId }}</strong>
                <span class="speaker-tag" v-if="node.speaker_id">{{ node.speaker_id }}</span>
              </div>
              <p class="node-text">{{ truncate(node.text, 80) }}</p>
              <div v-if="node.choices?.length" class="node-choices">
                <span v-for="(choice, ci) in node.choices" :key="ci" class="choice-chip">
                  {{ truncate(choice.text, 30) }} &rarr; {{ choice.next_node_id }}
                </span>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- Properties Panel -->
      <aside class="properties-panel" v-if="selectedNodeId && currentDialogue">
        <div class="panel-title">
          <span class="eyebrow">Node Properties</span>
          <strong>{{ selectedNodeId }}</strong>
        </div>

        <div class="prop-section">
          <span class="prop-label">Node ID</span>
          <input class="input" v-model="selectedNodeId" disabled />
        </div>

        <div class="prop-section">
          <span class="prop-label">Speaker</span>
          <select class="input" v-model="currentDialogue.nodes[selectedNodeId].speaker_id">
            <option value="">None (Narrator)</option>
            <option v-for="char in characters" :key="char.id" :value="char.id">{{ char.name }}</option>
          </select>
        </div>

        <div class="prop-section">
          <span class="prop-label">Dialogue Text</span>
          <textarea class="input" rows="4" v-model="currentDialogue.nodes[selectedNodeId].text" placeholder="Character speaks this text..."></textarea>
        </div>

        <div class="prop-section">
          <div class="prop-header">
            <span class="prop-label">Choices ({{ currentDialogue.nodes[selectedNodeId].choices?.length || 0 }})</span>
            <button class="btn btn-secondary btn-sm" @click="addChoice">+ Add</button>
          </div>
          <div v-for="(choice, ci) in currentDialogue.nodes[selectedNodeId].choices" :key="ci" class="choice-editor">
            <div class="choice-row">
              <input class="input" v-model="choice.text" placeholder="Choice text" />
              <input class="input" v-model="choice.next_node_id" placeholder="Next node ID" />
            </div>
            <div class="choice-meta">
              <label class="check-row">
                <span>Relationship change</span>
                <input type="number" class="input input-sm" v-model.number="choice.relationship_change" step="0.05" min="-1" max="1" />
              </label>
              <button class="btn-icon danger" @click="removeChoice(ci)" title="Remove choice">x</button>
            </div>
          </div>
        </div>

        <div class="prop-section">
          <span class="prop-label">Set as Start Node</span>
          <button class="btn btn-secondary btn-sm" @click="setStartNode(selectedNodeId)">Set Start</button>
        </div>

        <button class="btn btn-danger btn-sm" @click="deleteNode">Delete Node</button>
      </aside>
    </main>

    <Transition name="fade">
      <div v-if="statusMsg" class="status-toast" :class="{ error: !statusOk }" @click="statusMsg = null">
        {{ statusMsg }}
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invokeCommand } from '../lib/tauri'
import { useI18n } from '../lib/i18n'

const { t } = useI18n()

interface DialogueChoice {
  text: string
  next_node_id: string
  relationship_changes?: Record<string, number>
  relationship_change?: number
}

interface DialogueNode {
  speaker_id: string
  text: string
  emotion?: string
  choices: DialogueChoice[]
}

interface Dialogue {
  id: string
  title: string
  start_node_id: string
  nodes: Record<string, DialogueNode>
}

interface CharacterInfo {
  id: string
  name: string
}

const dialogues = ref<Dialogue[]>([])
const currentDialogue = ref<Dialogue | null>(null)
const selectedNodeId = ref<string | null>(null)
const characters = ref<CharacterInfo[]>([])
const statusMsg = ref<string | null>(null)
const statusOk = ref(true)

function truncate(text: string, len: number): string {
  if (!text) return ''
  return text.length > len ? text.slice(0, len) + '...' : text
}

async function loadDialogues() {
  try {
    await invokeCommand('load_dialogues', { directory: 'dialogue' }, undefined)
  } catch {}
}

async function loadCharacters() {
  try {
    characters.value = await invokeCommand<CharacterInfo[]>('get_characters', undefined, [])
  } catch {}
}

function selectDialogue(dlg: Dialogue) {
  currentDialogue.value = dlg
  selectedNodeId.value = dlg.start_node_id || Object.keys(dlg.nodes)[0] || null
}

function selectNode(nodeId: string) {
  selectedNodeId.value = nodeId
}

function newDialogue() {
  const id = 'dlg_' + Date.now()
  const dlg: Dialogue = {
    id,
    title: 'New Dialogue',
    start_node_id: 'start',
    nodes: {
      start: { speaker_id: '', text: 'Hello there!', choices: [] }
    }
  }
  dialogues.value.push(dlg)
  selectDialogue(dlg)
}

function addNode() {
  if (!currentDialogue.value) return
  const id = 'node_' + Date.now()
  currentDialogue.value.nodes[id] = { speaker_id: '', text: '', choices: [] }
  selectedNodeId.value = id
}

function deleteNode() {
  if (!currentDialogue.value || !selectedNodeId.value) return
  if (selectedNodeId.value === currentDialogue.value.start_node_id) {
    statusMsg.value = 'Cannot delete the start node'
    statusOk.value = false
    return
  }
  delete currentDialogue.value.nodes[selectedNodeId.value]
  selectedNodeId.value = Object.keys(currentDialogue.value.nodes)[0] || null
}

function addChoice() {
  if (!currentDialogue.value || !selectedNodeId.value) return
  const node = currentDialogue.value.nodes[selectedNodeId.value]
  if (!node.choices) node.choices = []
  node.choices.push({ text: '', next_node_id: '' })
}

function removeChoice(index: number) {
  if (!currentDialogue.value || !selectedNodeId.value) return
  currentDialogue.value.nodes[selectedNodeId.value].choices.splice(index, 1)
}

function setStartNode(nodeId: string) {
  if (!currentDialogue.value) return
  currentDialogue.value.start_node_id = nodeId
  statusMsg.value = 'Start node set to ' + nodeId
  statusOk.value = true
}

function validate() {
  if (!currentDialogue.value) return
  const dlg = currentDialogue.value
  const issues: string[] = []
  const nodeIds = new Set(Object.keys(dlg.nodes))

  if (!dlg.start_node_id || !nodeIds.has(dlg.start_node_id)) {
    issues.push('Start node ID does not match any existing node')
  }

  for (const [nodeId, node] of Object.entries(dlg.nodes)) {
    if (!node.text?.trim()) {
      issues.push('Node "' + nodeId + '" has empty text')
    }
    for (const choice of node.choices || []) {
      if (!choice.next_node_id) {
        issues.push('Choice in "' + nodeId + '" has no target node')
      } else if (!nodeIds.has(choice.next_node_id)) {
        issues.push('Choice in "' + nodeId + '" targets missing node "' + choice.next_node_id + '"')
      }
    }
  }

  if (issues.length === 0) {
    statusMsg.value = 'Validation passed! All nodes and connections are valid.'
    statusOk.value = true
  } else {
    statusMsg.value = issues.length + ' issues: ' + issues[0]
    statusOk.value = false
  }
}

function exportDialogue() {
  if (!currentDialogue.value) return
  const json = JSON.stringify(currentDialogue.value, null, 2)
  const blob = new Blob([json], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = (currentDialogue.value.title || 'dialogue') + '.json'
  a.click()
  URL.revokeObjectURL(url)
}

function importDialogue() {
  const input = document.createElement('input')
  input.type = 'file'
  input.accept = '.json'
  input.onchange = async (e: Event) => {
    const file = (e.target as HTMLInputElement).files?.[0]
    if (!file) return
    try {
      const text = await file.text()
      const dlg = JSON.parse(text) as Dialogue
      if (!dlg.id) dlg.id = 'imported_' + Date.now()
      if (!dlg.nodes) dlg.nodes = {}
      dialogues.value.push(dlg)
      selectDialogue(dlg)
      statusMsg.value = 'Imported: ' + dlg.title
      statusOk.value = true
    } catch (e) {
      statusMsg.value = 'Import failed: invalid JSON'
      statusOk.value = false
    }
  }
  input.click()
}

onMounted(async () => {
  await loadCharacters()
  await loadDialogues()
  // For browser preview, add sample dialogue
  if (dialogues.value.length === 0) {
    dialogues.value = [{
      id: 'sample',
      title: 'Sample Dialogue',
      start_node_id: 'start',
      nodes: {
        start: { speaker_id: characters.value[0]?.id || '', text: 'Welcome to the dialogue editor! This is a sample node.', choices: [
          { text: 'Hello!', next_node_id: 'response', relationship_changes: {} },
          { text: 'Tell me more.', next_node_id: 'info', relationship_changes: {} }
        ]},
        response: { speaker_id: characters.value[0]?.id || '', text: 'Nice to meet you!', choices: [] },
        info: { speaker_id: characters.value[0]?.id || '', text: 'This editor supports branching dialogues with multiple choices and outcomes.', choices: [] }
      }
    }]
  }
})
</script>

<style scoped>
.dialogue-editor {
  height: 100vh;
  min-height: 0;
  display: flex;
  flex-direction: column;
  background: var(--surface-0);
}

.toolbar {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  align-items: center;
  padding: 14px 18px;
  border-bottom: 1px solid var(--border);
  background: var(--surface-1);
}

.toolbar-left {
  min-width: 0;
  display: flex;
  align-items: baseline;
  gap: 12px;
}

.toolbar-left h1 { color: var(--text-primary); font-size: 18px; }
.dialogue-name { color: var(--text-tertiary); font-size: 13px; }

.toolbar-right { display: flex; gap: 8px; flex-shrink: 0; }

.eyebrow {
  display: block;
  color: var(--text-tertiary);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0;
  text-transform: uppercase;
}

.editor-body {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: 240px minmax(0, 1fr) 320px;
}

.dialogue-list, .properties-panel {
  min-height: 0;
  overflow-y: auto;
  background: var(--surface-1);
}

.dialogue-list {
  border-right: 1px solid var(--border);
  padding: 14px;
}

.properties-panel {
  border-left: 1px solid var(--border);
  padding: 14px;
}

.panel-title {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: center;
  margin-bottom: 14px;
}

.panel-title strong { color: var(--brand-light); font-size: 12px; }

.dlg-item {
  width: 100%;
  display: grid;
  gap: 3px;
  padding: 10px;
  margin-bottom: 6px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-2);
  color: var(--text-primary);
  cursor: pointer;
  text-align: left;
}

.dlg-item:hover, .dlg-item.active {
  border-color: var(--brand);
  background: var(--surface-3);
}

.dlg-item strong { font-size: 13px; }
.dlg-item small { color: var(--text-tertiary); font-size: 11px; }

.empty-list { color: var(--text-tertiary); font-size: 12px; padding: 20px; text-align: center; }

.tree-canvas { min-width: 0; display: flex; flex-direction: column; }

.empty-canvas {
  flex: 1;
  display: grid;
  place-items: center;
  gap: 12px;
  padding: 40px;
  text-align: center;
}

.empty-canvas h2 { color: var(--text-primary); font-size: 22px; }
.empty-canvas p { color: var(--text-tertiary); font-size: 13px; max-width: 460px; }

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

.node-tree { flex: 1; display: flex; flex-direction: column; }

.tree-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 18px;
  border-bottom: 1px solid var(--border);
}

.tree-scroll {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  display: grid;
  gap: 12px;
}

.tree-node {
  padding: 14px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-1);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.tree-node:hover { border-color: var(--brand); }
.tree-node.selected { border-color: var(--brand); background: var(--surface-2); box-shadow: var(--shadow-brand); }
.tree-node.is-start { border-left: 3px solid var(--success); }
.tree-node.is-end { border-left: 3px solid var(--danger); }

.node-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.node-badge {
  padding: 2px 8px;
  border-radius: var(--radius-sm);
  font-size: 9px;
  font-weight: 900;
  font-family: var(--font-mono);
}

.node-badge.start { background: rgba(34,197,94,0.2); color: var(--success); }
.node-badge.end { background: rgba(239,68,68,0.2); color: var(--danger); }
.node-badge.middle { background: var(--surface-3); color: var(--text-secondary); }

.node-header strong { font-size: 13px; color: var(--text-primary); }

.speaker-tag {
  margin-left: auto;
  padding: 2px 8px;
  border: 1px solid var(--border);
  border-radius: 999px;
  font-size: 10px;
  font-weight: 700;
  color: var(--brand-light);
}

.node-text {
  color: var(--text-secondary);
  font-size: 12px;
  line-height: 1.5;
  margin-bottom: 8px;
}

.node-choices { display: flex; gap: 6px; flex-wrap: wrap; }

.choice-chip {
  padding: 3px 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
  color: var(--text-secondary);
  font-size: 11px;
}

.prop-section { margin-bottom: 16px; }

.prop-label {
  display: block;
  color: var(--text-secondary);
  font-size: 12px;
  font-weight: 700;
  margin-bottom: 6px;
}

.prop-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 6px;
}

.choice-editor {
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  padding: 10px;
  margin-bottom: 8px;
  background: var(--surface-2);
}

.choice-row { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; margin-bottom: 8px; }

.choice-meta { display: flex; justify-content: space-between; align-items: center; }

.check-row { display: flex; gap: 8px; align-items: center; font-size: 12px; color: var(--text-secondary); }

.input-sm { width: 80px; }

.btn-icon {
  width: 28px;
  height: 28px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
  color: var(--text-secondary);
  cursor: pointer;
  font-weight: 800;
}

.btn-icon.danger:hover { border-color: var(--danger); color: var(--danger); }

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

@media (max-width: 1120px) {
  .editor-body { grid-template-columns: 200px minmax(0, 1fr); }
  .properties-panel { display: none; }
}

@media (max-width: 760px) {
  .editor-body { grid-template-columns: 1fr; }
  .dialogue-list { max-height: 200px; border-right: none; border-bottom: 1px solid var(--border); }
}
</style>
