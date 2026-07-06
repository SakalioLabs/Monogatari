<template>
  <div class="settings-page">
    <header class="page-header">
      <h1>Settings</h1>
      <p>Configure AI models, API keys, and engine options</p>
    </header>

    <section class="settings-section">
      <h2>AI Backend</h2>
      <div class="card">
        <div class="form-row">
          <label>Provider</label>
          <select v-model="provider" class="input">
            <option value="api">OpenAI-compatible API</option>
            <option value="onnx">Local ONNX Model</option>
          </select>
        </div>
        <template v-if="provider === 'api'">
          <div class="form-row">
            <label>Base URL</label>
            <input v-model="apiBaseUrl" class="input" placeholder="https://api.openai.com/v1" />
          </div>
          <div class="form-row">
            <label>API Key</label>
            <input v-model="apiKey" type="password" class="input" placeholder="sk-..." />
          </div>
          <div class="form-row">
            <label>Model</label>
            <input v-model="apiModel" class="input" placeholder="gpt-4o-mini" />
          </div>
        </template>
        <template v-else>
          <div class="form-row">
            <label>Model Path</label>
            <input v-model="modelPath" class="input" placeholder="models/model.onnx" />
          </div>
          <div class="form-row">
            <label>Tokenizer Path</label>
            <input v-model="tokenizerPath" class="input" placeholder="models/tokenizer.json" />
          </div>
          <div class="form-row">
            <label>Use DirectML</label>
            <input type="checkbox" v-model="useDirectML" />
          </div>
        </template>
        <div class="form-actions">
          <button class="btn btn-primary" @click="saveAI" :disabled="saving">
            {{ saving ? 'Saving...' : 'Save & Connect' }}
          </button>
          <span v-if="aiStatus" class="status-msg" :class="aiStatusOk ? 'ok' : 'err'">{{ aiStatus }}</span>
        </div>
      </div>
    </section>

    <section class="settings-section">
      <h2>Engine</h2>
      <div class="card">
        <div class="form-row">
          <label>Project Data Path</label>
          <input v-model="projectPath" class="input" placeholder="./data" />
        </div>
        <div class="form-actions">
          <button class="btn btn-primary" @click="initEngine">Initialize Engine</button>
        </div>
      </div>
    </section>

    <section class="settings-section" v-if="aiStatusObj">
      <h2>Connection Status</h2>
      <div class="card">
        <div class="status-grid">
          <div class="sg-item"><span class="sg-label">Initialized</span><span class="sg-val">{{ aiStatusObj.initialized ? 'Yes' : 'No' }}</span></div>
          <div class="sg-item"><span class="sg-label">Characters</span><span class="sg-val">{{ aiStatusObj.character_count }}</span></div>
          <div class="sg-item"><span class="sg-label">Dialogues</span><span class="sg-val">{{ aiStatusObj.dialogue_count }}</span></div>
          <div class="sg-item"><span class="sg-label">AI Engine</span><span class="sg-val">{{ aiStatusObj.active_ai_engine || 'None' }}</span></div>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const provider = ref('api')
const apiBaseUrl = ref('https://api.openai.com/v1')
const apiKey = ref('')
const apiModel = ref('gpt-4o-mini')
const modelPath = ref('')
const tokenizerPath = ref('')
const useDirectML = ref(true)
const projectPath = ref('./data')
const saving = ref(false)
const aiStatus = ref('')
const aiStatusOk = ref(false)
const aiStatusObj = ref<any>(null)

async function saveAI() {
  saving.value = true
  aiStatus.value = ''
  try {
    if (provider.value === 'api') {
      await invoke('configure_api', { baseUrl: apiBaseUrl.value, apiKey: apiKey.value, model: apiModel.value })
    } else {
      await invoke('configure_onnx', { modelPath: modelPath.value, tokenizerPath: tokenizerPath.value })
    }
    aiStatus.value = 'Connected!'
    aiStatusOk.value = true
  } catch (e: any) {
    aiStatus.value = String(e)
    aiStatusOk.value = false
  } finally {
    saving.value = false
  }
}

async function initEngine() {
  try {
    await invoke('initialize_engine', { projectPath: projectPath.value })
    aiStatus.value = 'Engine initialized'
    aiStatusOk.value = true
    refreshStatus()
  } catch (e: any) {
    aiStatus.value = String(e)
    aiStatusOk.value = false
  }
}

async function refreshStatus() {
  try { aiStatusObj.value = await invoke('get_engine_status') } catch {}
}

onMounted(refreshStatus)
</script>

<style scoped>
.settings-page { max-width: 720px; margin: 0 auto; padding: 32px 40px; }
.page-header { margin-bottom: 32px; }
.page-header h1 { font-size: 24px; font-weight: 700; }
.page-header p { color: var(--text-tertiary); font-size: 14px; margin-top: 4px; }
.settings-section { margin-bottom: 32px; }
.settings-section h2 { font-size: 16px; font-weight: 600; color: var(--brand-light); margin-bottom: 12px; }
.form-row { margin-bottom: 16px; }
.form-row label { display: block; font-size: 12px; color: var(--text-secondary); margin-bottom: 6px; text-transform: uppercase; letter-spacing: 0.04em; }
.form-row input[type="checkbox"] { width: 18px; height: 18px; accent-color: var(--brand); }
select.input { cursor: pointer; }
.form-actions { display: flex; align-items: center; gap: 12px; margin-top: 8px; }
.status-msg { font-size: 13px; }
.status-msg.ok { color: var(--success); }
.status-msg.err { color: var(--danger); }
.status-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; }
.sg-item { text-align: center; padding: 14px; background: var(--surface-2); border-radius: var(--radius-sm); }
.sg-label { display: block; font-size: 11px; color: var(--text-tertiary); text-transform: uppercase; margin-bottom: 4px; }
.sg-val { display: block; font-size: 18px; font-weight: 700; color: var(--brand-light); }
</style>
