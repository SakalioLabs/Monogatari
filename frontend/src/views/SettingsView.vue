<template>
  <div class="settings-page">
    <header class="page-header">
      <div>
        <span class="eyebrow">Project Control</span>
        <h1>Settings</h1>
        <p>{{ projectState?.project_path || projectPath }}</p>
      </div>
      <div class="header-actions">
        <button class="btn btn-secondary btn-sm" @click="refreshAll">Refresh</button>
        <button class="btn btn-primary btn-sm" :disabled="savingProject" @click="saveProject">
          {{ savingProject ? 'Saving' : 'Save Project' }}
        </button>
      </div>
    </header>

    <section class="status-strip">
      <div class="status-cell" :class="{ ok: projectState?.settings_exists }">
        <span>Settings</span>
        <strong>{{ projectState?.settings_exists ? 'Saved' : 'Default' }}</strong>
      </div>
      <div class="status-cell" :class="{ ok: projectState?.valid, danger: projectState && !projectState.valid }">
        <span>Project</span>
        <strong>{{ projectState?.valid ? 'Ready' : 'Review' }}</strong>
      </div>
      <div class="status-cell" :class="{ ok: engineStatus?.initialized }">
        <span>Runtime</span>
        <strong>{{ engineStatus?.initialized ? 'Online' : 'Idle' }}</strong>
      </div>
      <div class="status-cell" :class="{ warning: issueCount > 0 }">
        <span>Issues</span>
        <strong>{{ issueCount }}</strong>
      </div>
    </section>

    <main class="settings-layout">
      <section class="settings-stack">
        <div class="panel">
          <div class="panel-head">
            <div>

              <span class="eyebrow">Project</span>
              <strong>Workspace</strong>
            </div>
            <button class="btn btn-secondary btn-sm" :disabled="initializing" @click="initEngine">
              {{ initializing ? 'Initializing' : 'Initialize' }}
            </button>
          </div>
          <div class="form-grid two">
            <label class="form-field wide">
              <span>Project Data Path</span>
              <input v-model="projectPath" class="input" placeholder="./data" @change="loadProjectConfig" />
            </label>
            <label class="form-field">
              <span>Title</span>
              <input v-model="projectTitle" class="input" />
            </label>
            <label class="form-field">
              <span>Target FPS</span>
              <input v-model.number="targetFps" type="number" min="15" max="240" class="input" />
            </label>
          </div>
        </div>

        <div class="panel">
          <div class="panel-head">
            <div>
              <span class="eyebrow">Paths</span>
              <strong>Project Directories</strong>
            </div>
          </div>
          <div class="path-grid">
            <label v-for="path in editablePaths" :key="path.key" class="path-row" :class="{ missing: !path.exists && path.required }">
              <span>
                <b>{{ path.label }}</b>
                <small>{{ path.exists ? `${path.item_count} items` : 'missing' }}</small>
              </span>
              <input v-model="pathEdits[path.key]" class="input" />
            </label>
          </div>
        </div>

        <div class="panel">
          <div class="panel-head">
            <div>
              <span class="eyebrow">AI Backend</span>
              <strong>{{ providerLabel }}</strong>
            </div>
            <button class="btn btn-primary btn-sm" :disabled="savingAI" @click="saveAI">
              {{ savingAI ? 'Connecting' : 'Connect' }}
            </button>
          </div>
          <div class="form-grid two">
            <label class="form-field">
              <span>Provider</span>
              <select v-model="provider" class="input">
                <option value="api">OpenAI-compatible API</option>
                <option value="onnx">Local ONNX Model</option>
              </select>
            </label>
            <label v-if="provider === 'api'" class="form-field">
              <span>Model</span>
              <input v-model="apiModel" class="input" placeholder="gpt-4o-mini" />
            </label>
            <label v-if="provider === 'api'" class="form-field wide">
              <span>Base URL</span>
              <input v-model="apiBaseUrl" class="input" placeholder="https://api.openai.com/v1" />
            </label>
            <label v-if="provider === 'api'" class="form-field wide">
              <span>API Key</span>
              <input v-model="apiKey" type="password" class="input" placeholder="sk-..." />
            </label>

            <label v-if="provider === 'onnx'" class="form-field wide">
              <span>Model Path</span>
              <input v-model="modelPath" class="input" placeholder="models/model.onnx" />
            </label>
            <label v-if="provider === 'onnx'" class="form-field wide">
              <span>Tokenizer Path</span>
              <input v-model="tokenizerPath" class="input" placeholder="models/tokenizer.json" />
            </label>
            <label v-if="provider === 'onnx'" class="check-row">
              <input type="checkbox" v-model="useDirectML" />
              <span>Use DirectML</span>
            </label>
          </div>
        </div>
    <section class="panel">
      <span class="eyebrow">Voice / TTS</span>
      <p class="muted">Configure text-to-speech for character voices. Select a provider and set language, speed, and pitch defaults.</p>
      <div class="field-grid">
        <div class="field">
          <label>TTS Provider</label>
          <select v-model="ttsConfig.provider" class="input">
            <option value="system">System</option>
            <option value="api">API</option>
            <option value="local">Local Model</option>
          </select>
        </div>
        <div class="field">
          <label>Language</label>
          <select v-model="ttsConfig.language" class="input">
            <option value="ja">Japanese</option>
            <option value="en">English</option>
            <option value="zh">Chinese</option>
          </select>
        </div>
        <div class="field">
          <label>Speed</label>
          <input type="range" v-model.number="ttsConfig.speed" min="0.5" max="2.0" step="0.1" />
          <span class="muted">{{ ttsConfig.speed.toFixed(1) }}x</span>
        </div>
        <div class="field">
          <label>Pitch</label>
          <input type="range" v-model.number="ttsConfig.pitch" min="0.5" max="2.0" step="0.1" />
          <span class="muted">{{ ttsConfig.pitch.toFixed(1) }}</span>
        </div>
      </div>
      <div class="actions" style="margin-top:12px">
        <button class="btn btn-primary btn-sm" @click="saveTts">Save TTS Config</button>
      </div>
    </section>

      </section>

      <aside class="inspector">
        <section class="panel">
          <div class="panel-head">
            <div>
              <span class="eyebrow">Runtime</span>
              <strong>{{ engineStatus?.active_ai_engine || 'No AI engine' }}</strong>
            </div>
          </div>
          <div class="metric-grid">
            <div><span>Characters</span><strong>{{ engineStatus?.character_count ?? 0 }}</strong></div>
            <div><span>Dialogues</span><strong>{{ engineStatus?.dialogue_count ?? 0 }}</strong></div>
            <div><span>Knowledge</span><strong>{{ engineStatus?.knowledge_count ?? 0 }}</strong></div>
            <div><span>Engines</span><strong>{{ engineStatus?.ai_engines.length ?? 0 }}</strong></div>
          </div>
        </section>

        <section class="panel">
          <div class="panel-head">
            <div>
              <span class="eyebrow">Diagnostics</span>
              <strong :class="projectState?.valid ? 'ok-text' : 'bad-text'">{{ projectState?.valid ? 'Clean' : 'Attention' }}</strong>
            </div>
          </div>
          <div v-if="projectState?.issues.length" class="issue-list">
            <div v-for="(issue, index) in projectState.issues" :key="`${issue.code}-${index}`" class="issue-item" :class="issue.severity">
              <span>{{ issue.severity }} · {{ issue.code }}</span>
              <strong>{{ issue.path || 'project' }}</strong>
              <p>{{ issue.message }}</p>
            </div>
          </div>
          <p v-else class="muted">No project issues detected.</p>
        </section>

        <section class="panel">
          <div class="panel-head">
            <div>
              <span class="eyebrow">Save Target</span>
              <strong>settings.json</strong>
            </div>
          </div>
          <p class="muted">{{ projectState?.settings_path || 'Not loaded' }}</p>
          <p v-if="statusMessage" class="status-message" :class="{ error: !statusOk }">{{ statusMessage }}</p>
        </section>
      </aside>
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { invokeCommand } from '../lib/tauri'

interface EngineStatus {
  initialized: boolean
  character_count: number
  dialogue_count: number
  knowledge_count: number
  ai_engines: string[]
  active_ai_engine: string | null
}

interface ProjectPathStatus {
  key: string
  label: string
  relative_path: string
  absolute_path: string
  exists: boolean
  item_count: number
  required: boolean
}

interface ProjectConfigIssue {
  severity: string
  code: string
  path: string | null
  message: string
}

interface ProjectConfigState {
  project_path: string
  settings_path: string
  settings_exists: boolean
  valid: boolean
  error_count: number
  warning_count: number
  config: Record<string, any>
  paths: ProjectPathStatus[]
  issues: ProjectConfigIssue[]
}

const projectPath = ref('./data')
const projectState = ref<ProjectConfigState | null>(null)
const engineStatus = ref<EngineStatus | null>(null)
const savingProject = ref(false)
const savingAI = ref(false)
const initializing = ref(false)
const statusMessage = ref('')
const statusOk = ref(true)

const projectTitle = ref('LLM Galgame Engine')
const targetFps = ref(60)
const provider = ref('api')
const apiBaseUrl = ref('https://api.openai.com/v1')
const apiKey = ref('')
const apiModel = ref('gpt-4o-mini')
const modelPath = ref('models/model.onnx')
const tokenizerPath = ref('models/tokenizer.json')
const useDirectML = ref(true)
const ttsConfig = ref({ provider: 'system', language: 'ja', speed: 1.0, pitch: 1.0 })

async function saveTts() {
  try {
    await invokeCommand('configure_tts', { config: ttsConfig.value })
    statusMessage.value = 'TTS configured'
    statusOk.value = true
  } catch (e: any) {
    statusMessage.value = 'TTS save failed: ' + e
    statusOk.value = false
  }
}

const pathEdits = reactive<Record<string, string>>({
  characters: 'characters',
  dialogue: 'dialogue',
  knowledge: 'knowledge',
  scenes: 'scenes',
  assets: 'assets',
  saves: 'saves',
})

const previewState: ProjectConfigState = {
  project_path: 'Browser preview',
  settings_path: 'Browser preview/settings.json',
  settings_exists: true,
  valid: true,
  error_count: 0,
  warning_count: 1,
  config: {
    render: { title: 'LLM Galgame Engine' },
    engine: { target_fps: 60 },
    ai: {
      provider: 'api',
      api: { base_url: 'https://api.openai.com/v1', api_key: '', model: 'gpt-4o-mini' },
      onnx: { model_path: 'models/model.onnx', tokenizer_path: 'models/tokenizer.json', use_directml: true },
    },
    paths: {
      characters: 'characters',
      dialogue: 'dialogue',
      knowledge: 'knowledge',
      scenes: 'scenes',
      assets: 'assets',
      saves: 'saves',
    },
  },
  paths: [
    { key: 'characters', label: 'Characters', relative_path: 'characters', absolute_path: '', exists: true, item_count: 1, required: true },
    { key: 'dialogue', label: 'Dialogue', relative_path: 'dialogue', absolute_path: '', exists: true, item_count: 1, required: true },
    { key: 'knowledge', label: 'Knowledge', relative_path: 'knowledge', absolute_path: '', exists: true, item_count: 1, required: true },
    { key: 'scenes', label: 'Scenes', relative_path: 'scenes', absolute_path: '', exists: true, item_count: 2, required: false },
    { key: 'assets', label: 'Assets', relative_path: 'assets', absolute_path: '', exists: true, item_count: 1, required: true },
    { key: 'saves', label: 'Saves', relative_path: 'saves', absolute_path: '', exists: false, item_count: 0, required: false },
  ],
  issues: [{ severity: 'warning', code: 'api_key_missing', path: 'ai.api.api_key', message: 'API key is not configured.' }],
}

const providerLabel = computed(() => provider.value === 'api' ? 'OpenAI-compatible API' : 'Local ONNX Model')
const issueCount = computed(() => (projectState.value?.error_count || 0) + (projectState.value?.warning_count || 0))
const editablePaths = computed(() => projectState.value?.paths || previewState.paths)

async function refreshAll() {
  await Promise.all([loadProjectConfig(), refreshStatus()])
}

async function loadProjectConfig() {
  try {
    projectState.value = await invokeCommand<ProjectConfigState>(
      'get_project_config',
      { projectPath: projectPath.value },
      previewState
    )
    applyProjectState(projectState.value)
  } catch (e) {
    statusMessage.value = String(e)
    statusOk.value = false
  }
}

function applyProjectState(state: ProjectConfigState) {
  const config = state.config
  projectTitle.value = getConfigValue(config, ['render', 'title'])
    || getConfigValue(config, ['engine', 'title'])
    || projectTitle.value
  targetFps.value = Number(getConfigValue(config, ['engine', 'target_fps']) || getConfigValue(config, ['engine', 'targetFps']) || targetFps.value)
  provider.value = getConfigValue(config, ['ai', 'provider']) || provider.value
  apiBaseUrl.value = getConfigValue(config, ['ai', 'api', 'base_url']) || getConfigValue(config, ['ai', 'api', 'baseUrl']) || apiBaseUrl.value
  apiKey.value = getConfigValue(config, ['ai', 'api', 'api_key']) || getConfigValue(config, ['ai', 'api', 'apiKey']) || apiKey.value
  apiModel.value = getConfigValue(config, ['ai', 'api', 'model']) || apiModel.value
  modelPath.value = getConfigValue(config, ['ai', 'onnx', 'model_path']) || getConfigValue(config, ['ai', 'onnx', 'modelPath']) || modelPath.value
  tokenizerPath.value = getConfigValue(config, ['ai', 'onnx', 'tokenizer_path']) || getConfigValue(config, ['ai', 'onnx', 'tokenizerPath']) || tokenizerPath.value
  useDirectML.value = Boolean(getConfigValue(config, ['ai', 'onnx', 'use_directml']) ?? getConfigValue(config, ['ai', 'onnx', 'useDirectML']) ?? useDirectML.value)
  for (const path of state.paths) {
    pathEdits[path.key] = path.relative_path
  }
}

async function saveProject() {
  if (!projectState.value) return
  savingProject.value = true
  try {
    const config = buildConfigForSave(projectState.value.config)
    projectState.value = await invokeCommand<ProjectConfigState>(
      'save_project_config',
      { projectPath: projectPath.value, config },
      () => ({ ...previewState, config })
    )
    applyProjectState(projectState.value)
    statusMessage.value = 'Project settings saved'
    statusOk.value = true
  } catch (e) {
    statusMessage.value = String(e)
    statusOk.value = false
  } finally {
    savingProject.value = false
  }
}

async function saveAI() {
  savingAI.value = true
  try {
    if (provider.value === 'api') {
      await invokeCommand<void>('configure_api', { baseUrl: apiBaseUrl.value, apiKey: apiKey.value, model: apiModel.value })
    } else {
      await invokeCommand<void>('configure_onnx', { modelPath: modelPath.value, tokenizerPath: tokenizerPath.value })
    }
    statusMessage.value = 'AI backend configured'
    statusOk.value = true
    await refreshStatus()
  } catch (e) {
    statusMessage.value = String(e)
    statusOk.value = false
  } finally {
    savingAI.value = false
  }
}

async function initEngine() {
  initializing.value = true
  try {
    await invokeCommand<void>('initialize_engine', { projectPath: projectPath.value })
    statusMessage.value = 'Engine initialized'
    statusOk.value = true
    await Promise.all([refreshStatus(), loadProjectConfig()])
  } catch (e) {
    statusMessage.value = String(e)
    statusOk.value = false
  } finally {
    initializing.value = false
  }
}

async function refreshStatus() {
  try {
    engineStatus.value = await invokeCommand<EngineStatus>('get_engine_status', undefined, {
      initialized: false,
      character_count: 0,
      dialogue_count: 0,
      knowledge_count: 0,
      ai_engines: [],
      active_ai_engine: null,
    })
  } catch {}
}

function buildConfigForSave(source: Record<string, any>) {
  const config = JSON.parse(JSON.stringify(source || {}))
  setConfigValue(config, ['render', 'title'], projectTitle.value)
  setConfigValue(config, ['engine', 'target_fps'], Number(targetFps.value) || 60)
  setConfigValue(config, ['ai', 'provider'], provider.value)
  setConfigValue(config, ['ai', 'api', 'base_url'], apiBaseUrl.value)
  setConfigValue(config, ['ai', 'api', 'api_key'], apiKey.value)
  setConfigValue(config, ['ai', 'api', 'model'], apiModel.value)
  setConfigValue(config, ['ai', 'onnx', 'model_path'], modelPath.value)
  setConfigValue(config, ['ai', 'onnx', 'tokenizer_path'], tokenizerPath.value)
  setConfigValue(config, ['ai', 'onnx', 'use_directml'], useDirectML.value)
  for (const [key, value] of Object.entries(pathEdits)) {
    setConfigValue(config, ['paths', key], value)
  }
  return config
}

function getConfigValue(config: Record<string, any>, path: string[]) {
  return path.reduce<any>((current, key) => current?.[key], config)
}

function setConfigValue(config: Record<string, any>, path: string[], value: any) {
  let current = config
  for (const key of path.slice(0, -1)) {
    if (!current[key] || typeof current[key] !== 'object') current[key] = {}
    current = current[key]
  }
  current[path[path.length - 1]] = value
}

onMounted(refreshAll)
</script>

<style scoped>
.settings-page {
  max-width: 1280px;
  margin: 0 auto;
  padding: 34px 40px;
}

.page-header {
  display: flex;
  justify-content: space-between;
  gap: 18px;
  align-items: flex-start;
  margin-bottom: 22px;
}

.page-header h1 {
  color: var(--text-primary);
  font-size: 28px;
  line-height: 1.15;
}

.page-header p,
.muted {
  overflow: hidden;
  color: var(--text-tertiary);
  font-size: 13px;
  text-overflow: ellipsis;
}

.page-header p {
  max-width: 760px;
  white-space: nowrap;
}

.eyebrow {
  display: block;
  color: var(--text-tertiary);
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0;
  text-transform: uppercase;
}

.header-actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

.status-strip {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12px;
  margin-bottom: 18px;
}

.status-cell,
.panel {
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-1);
}

.status-cell {
  display: grid;
  gap: 4px;
  min-height: 78px;
  padding: 15px;
}

.status-cell span {
  color: var(--text-tertiary);
  font-size: 11px;
  font-weight: 800;
  text-transform: uppercase;
}

.status-cell strong {
  overflow: hidden;
  color: var(--brand-light);
  font-size: 22px;
  line-height: 1;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.status-cell.ok strong,
.ok-text {
  color: var(--success);
}

.status-cell.warning strong {
  color: var(--warning);
}

.status-cell.danger strong,
.bad-text {
  color: var(--danger);
}

.settings-layout {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 340px;
  gap: 18px;
  align-items: start;
}

.settings-stack,
.inspector {
  display: grid;
  gap: 14px;
}

.inspector {
  position: sticky;
  top: 18px;
}

.panel {
  display: grid;
  gap: 16px;
  padding: 16px;
}

.panel-head {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: center;
}

.panel-head strong {
  color: var(--text-primary);
  font-size: 15px;
}

.form-grid {
  display: grid;
  gap: 12px;
}

.form-grid.two {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.form-field {
  display: grid;
  gap: 6px;
}

.form-field.wide {
  grid-column: 1 / -1;
}

.form-field span,
.check-row span {
  color: var(--text-secondary);
  font-size: 12px;
  font-weight: 800;
}

.path-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}

.path-row {
  display: grid;
  gap: 8px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-2);
  padding: 12px;
}

.path-row.missing {
  border-color: rgba(239,68,68,0.38);
}

.path-row > span {
  display: flex;
  justify-content: space-between;
  gap: 10px;
}

.path-row b {
  color: var(--text-primary);
  font-size: 13px;
}

.path-row small {
  color: var(--text-tertiary);
  font-size: 11px;
  font-weight: 800;
}

.check-row {
  display: flex;
  gap: 8px;
  align-items: center;
}

.check-row input {
  width: 18px;
  height: 18px;
  accent-color: var(--brand);
}

select.input {
  cursor: pointer;
}

.metric-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
}

.metric-grid div {
  display: grid;
  gap: 3px;
  border-radius: var(--radius-sm);
  background: var(--surface-2);
  padding: 12px;
}

.metric-grid span {
  color: var(--text-tertiary);
  font-size: 11px;
  font-weight: 800;
  text-transform: uppercase;
}

.metric-grid strong {
  color: var(--brand-light);
  font-size: 20px;
}

.issue-list {
  display: grid;
  gap: 8px;
}

.issue-item {
  display: grid;
  gap: 3px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
  padding: 10px;
}

.issue-item span {
  color: var(--text-tertiary);
  font-size: 10px;
  font-weight: 900;
  text-transform: uppercase;
}

.issue-item.error span {
  color: var(--danger);
}

.issue-item.warning span {
  color: var(--warning);
}

.issue-item strong {
  overflow: hidden;
  color: var(--text-primary);
  font-size: 12px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.issue-item p,
.status-message {
  color: var(--text-secondary);
  font-size: 12px;
  line-height: 1.35;
}

.status-message {
  color: var(--success);
}

.status-message.error {
  color: var(--danger);
}

@media (max-width: 1060px) {
  .settings-layout {
    grid-template-columns: 1fr;
  }

  .inspector {
    position: static;
  }
}

@media (max-width: 720px) {
  .settings-page {
    padding: 22px 16px;
  }

  .page-header {
    flex-direction: column;
  }

  .header-actions {
    width: 100%;
  }

  .header-actions .btn {
    justify-content: center;
    flex: 1;
  }

  .status-strip,
  .form-grid.two,
  .path-grid {
    grid-template-columns: 1fr;
  }
}
</style>
