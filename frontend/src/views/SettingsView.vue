<template>
  <div class="settings">
    <h2>⚙️ Settings</h2>

    <!-- AI Configuration -->
    <div class="card section">
      <h3>🤖 AI Configuration</h3>
      <div class="tabs">
        <button
          class="tab"
          :class="{ active: aiTab === 'api' }"
          @click="aiTab = 'api'"
        >
          API (OpenAI)
        </button>
        <button
          class="tab"
          :class="{ active: aiTab === 'onnx' }"
          @click="aiTab = 'onnx'"
        >
          ONNX (Local)
        </button>
      </div>

      <div v-if="aiTab === 'api'" class="form">
        <div class="form-group">
          <label>Base URL</label>
          <input class="input" v-model="apiConfig.baseUrl" placeholder="https://api.openai.com/v1" />
        </div>
        <div class="form-group">
          <label>API Key</label>
          <input class="input" type="password" v-model="apiConfig.apiKey" placeholder="sk-..." />
        </div>
        <div class="form-group">
          <label>Model</label>
          <input class="input" v-model="apiConfig.model" placeholder="gpt-3.5-turbo" />
        </div>
        <button class="btn btn-primary" @click="configureAPI">Connect API</button>
      </div>

      <div v-if="aiTab === 'onnx'" class="form">
        <div class="form-group">
          <label>Model Path</label>
          <input class="input" v-model="onnxConfig.modelPath" placeholder="path/to/model.onnx" />
        </div>
        <div class="form-group">
          <label>Tokenizer Path</label>
          <input class="input" v-model="onnxConfig.tokenizerPath" placeholder="path/to/tokenizer.json" />
        </div>
        <button class="btn btn-primary" @click="configureONNX">Load Model</button>
      </div>

      <div class="ai-status" v-if="aiStatus">
        <p>Active Engine: <strong>{{ aiStatus.active_engine || 'None' }}</strong></p>
        <p>Available Engines: {{ aiStatus.engines.map(e => e.name).join(', ') || 'None' }}</p>
      </div>
    </div>

    <!-- Project Configuration -->
    <div class="card section">
      <h3>📁 Project</h3>
      <div class="form-group">
        <label>Project Directory</label>
        <div class="input-group">
          <input class="input" v-model="projectPath" placeholder="path/to/project" />
          <button class="btn btn-secondary" @click="initializeEngine">Load</button>
        </div>
      </div>
      <div v-if="engineStatus" class="engine-status">
        <p>Characters: {{ engineStatus.character_count }}</p>
        <p>Dialogues: {{ engineStatus.dialogue_count }}</p>
        <p>Knowledge: {{ engineStatus.knowledge_count }}</p>
      </div>
    </div>

    <!-- Engine Status -->
    <div class="card section">
      <h3>📊 Engine Status</h3>
      <button class="btn btn-secondary" @click="refreshStatus">Refresh</button>
      <pre v-if="engineStatus" class="status-json">{{ JSON.stringify(engineStatus, null, 2) }}</pre>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface AIStatus {
  engines: { name: string; ready: boolean }[]
  active_engine: string | null
}

interface EngineStatus {
  initialized: boolean
  character_count: number
  dialogue_count: number
  knowledge_count: number
  ai_engines: string[]
  active_ai_engine: string | null
}

const aiTab = ref<'api' | 'onnx'>('api')
const projectPath = ref('')
const aiStatus = ref<AIStatus | null>(null)
const engineStatus = ref<EngineStatus | null>(null)

const apiConfig = ref({
  baseUrl: 'https://api.openai.com/v1',
  apiKey: '',
  model: 'gpt-3.5-turbo',
})

const onnxConfig = ref({
  modelPath: '',
  tokenizerPath: '',
})

async function configureAPI() {
  try {
    const result = await invoke<string>('configure_api', {
      baseUrl: apiConfig.value.baseUrl,
      apiKey: apiConfig.value.apiKey,
      model: apiConfig.value.model,
    })
    alert(result)
    await refreshAIStatus()
  } catch (e) {
    alert(`Error: ${e}`)
  }
}

async function configureONNX() {
  try {
    const result = await invoke<string>('configure_onnx', {
      modelPath: onnxConfig.value.modelPath,
      tokenizerPath: onnxConfig.value.tokenizerPath,
    })
    alert(result)
    await refreshAIStatus()
  } catch (e) {
    alert(`Error: ${e}`)
  }
}

async function initializeEngine() {
  if (!projectPath.value) {
    alert('Please enter a project path')
    return
  }
  try {
    const result = await invoke<string>('initialize_engine', {
      projectPath: projectPath.value,
    })
    alert(result)
    await refreshStatus()
  } catch (e) {
    alert(`Error: ${e}`)
  }
}

async function refreshAIStatus() {
  try {
    aiStatus.value = await invoke('get_ai_status')
  } catch (e) {
    console.error('Failed to get AI status:', e)
  }
}

async function refreshStatus() {
  try {
    engineStatus.value = await invoke('get_engine_status')
  } catch (e) {
    console.error('Failed to get engine status:', e)
  }
}

onMounted(() => {
  refreshAIStatus()
  refreshStatus()
})
</script>

<style scoped>
.settings {
  padding: 30px;
  max-width: 800px;
  margin: 0 auto;
}

.settings h2 {
  margin-bottom: 30px;
  color: var(--primary);
}

.section {
  margin-bottom: 20px;
}

.section h3 {
  margin-bottom: 20px;
  color: var(--secondary);
}

.tabs {
  display: flex;
  gap: 4px;
  margin-bottom: 20px;
}

.tab {
  padding: 8px 16px;
  background: var(--bg-input);
  border: none;
  border-radius: var(--radius);
  color: var(--text-muted);
  cursor: pointer;
}

.tab.active {
  background: var(--primary);
  color: white;
}

.form {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.form-group label {
  font-size: 13px;
  color: var(--text-muted);
}

.input-group {
  display: flex;
  gap: 10px;
}

.input-group .input {
  flex: 1;
}

.ai-status, .engine-status {
  margin-top: 15px;
  padding: 15px;
  background: var(--bg-input);
  border-radius: var(--radius);
}

.ai-status p, .engine-status p {
  font-size: 13px;
  color: var(--text-muted);
  margin-bottom: 5px;
}

.status-json {
  margin-top: 15px;
  padding: 15px;
  background: var(--bg-input);
  border-radius: var(--radius);
  font-size: 12px;
  overflow-x: auto;
  color: var(--secondary);
}
</style>
