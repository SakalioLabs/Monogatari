<template>
  <div class="plugin-page">
    <header class="page-header">
      <div>
        <span class="eyebrow">Extensibility</span>
        <h1>Plugins</h1>
        <p>Manage custom node types, workflow extensions, and third-party integrations.</p>
      </div>
      <div class="header-actions">
        <button class="btn btn-secondary btn-sm" @click="refreshPlugins">Refresh</button>
        <button class="btn btn-primary btn-sm" @click="showRegister = true">Register Plugin</button>
      </div>
    </header>

    <section class="plugin-grid" v-if="plugins.length > 0">
      <div v-for="plugin in plugins" :key="plugin.name" class="plugin-card">
        <div class="plugin-head">
          <span class="plugin-icon">P</span>
          <div class="plugin-meta">
            <strong>{{ plugin.name }}</strong>
            <span>{{ plugin.version || 'v1.0' }}</span>
          </div>
          <span class="plugin-status" :class="{ active: plugin.enabled }">{{ plugin.enabled ? 'Active' : 'Inactive' }}</span>
        </div>
        <p class="plugin-desc">{{ plugin.description || 'No description provided.' }}</p>
        <div class="plugin-footer">
          <span class="plugin-type">{{ plugin.plugin_type || 'node' }}</span>
          <button class="btn btn-danger btn-sm" @click="removePlugin(plugin.name)">Remove</button>
        </div>
      </div>
    </section>

    <section v-else class="empty-state">
      <span class="empty-mark">PL</span>
      <h2>No plugins installed</h2>
      <p>Register a plugin to extend the workflow editor with custom node types and integrations.</p>
    </section>

    <Transition name="fade">
      <div v-if="showRegister" class="modal-overlay" @click.self="showRegister = false">
        <div class="modal">
          <div class="modal-head">
            <span class="eyebrow">Register</span>
            <button class="close-btn" @click="showRegister = false">Close</button>
          </div>
          <div class="form-stack">
            <label class="form-field">
              <span>Plugin Name</span>
              <input v-model="registerName" class="input" placeholder="my_custom_node" />
            </label>
            <label class="form-field">
              <span>Plugin Type</span>
              <select v-model="registerType" class="input">
                <option value="node">Custom Node</option>
                <option value="trigger">Event Trigger</option>
                <option value="action">Action Handler</option>
              </select>
            </label>
            <label class="form-field">
              <span>Description</span>
              <textarea v-model="registerDesc" class="input" rows="3" placeholder="What does this plugin do?"></textarea>
            </label>
            <button class="btn btn-primary" :disabled="!registerName.trim()" @click="doRegister">Register</button>
          </div>
        </div>
      </div>
    </Transition>

    <Transition name="fade">
      <div v-if="statusMessage" class="toast" :class="{ error: !statusOk }" @click="statusMessage = null">{{ statusMessage }}</div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { invokeCommand } from '../lib/tauri'

interface PluginInfo {
  name: string
  version: string
  description: string
  plugin_type: string
  enabled: boolean
}

const plugins = ref<PluginInfo[]>([])
const showRegister = ref(false)
const registerName = ref('')
const registerType = ref('node')
const registerDesc = ref('')
const statusMessage = ref<string | null>(null)
const statusOk = ref(true)

async function refreshPlugins() {
  try {
    plugins.value = await invokeCommand<PluginInfo[]>('list_plugins', undefined, [])
  } catch (e) {
    statusMessage.value = String(e)
    statusOk.value = false
  }
}

async function doRegister() {
  if (!registerName.value.trim()) return
  try {
    await invokeCommand<void>('register_plugin', {
      name: registerName.value.trim(),
      pluginType: registerType.value,
      description: registerDesc.value.trim(),
    })
    statusMessage.value = `Plugin "${registerName.value}" registered`
    statusOk.value = true
    showRegister.value = false
    registerName.value = ''
    registerDesc.value = ''
    await refreshPlugins()
  } catch (e) {
    statusMessage.value = String(e)
    statusOk.value = false
  }
}

async function removePlugin(name: string) {
  try {
    await invokeCommand<void>('remove_plugin', { name })
    statusMessage.value = `Plugin "${name}" removed`
    statusOk.value = true
    await refreshPlugins()
  } catch (e) {
    statusMessage.value = String(e)
    statusOk.value = false
  }
}

onMounted(refreshPlugins)
</script>

<style scoped>
.plugin-page {
  max-width: 1180px;
  margin: 0 auto;
  padding: 34px 40px;
}
.page-header {
  display: flex;
  justify-content: space-between;
  gap: 18px;
  align-items: flex-start;
  margin-bottom: 24px;
}
.page-header h1 { color: var(--text-primary); font-size: 28px; line-height: 1.15; }
.page-header p { color: var(--text-tertiary); font-size: 13px; margin-top: 6px; }
.eyebrow { display: block; color: var(--text-tertiary); font-size: 11px; font-weight: 800; text-transform: uppercase; }
.header-actions { display: flex; gap: 8px; flex-shrink: 0; }
.plugin-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(340px, 1fr)); gap: 14px; }
.plugin-card {
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-1);
  padding: 18px;
  display: grid;
  gap: 12px;
}
.plugin-head { display: flex; gap: 12px; align-items: center; }
.plugin-icon {
  width: 40px; height: 40px; border-radius: var(--radius-sm);
  background: var(--surface-3); color: var(--brand-light);
  display: flex; align-items: center; justify-content: center;
  font-weight: 900; font-size: 16px;
}
.plugin-meta { flex: 1; min-width: 0; }
.plugin-meta strong { display: block; color: var(--text-primary); font-size: 14px; }
.plugin-meta span { color: var(--text-tertiary); font-size: 11px; }
.plugin-status {
  padding: 3px 8px; border-radius: 100px;
  font-size: 10px; font-weight: 800; text-transform: uppercase;
  color: var(--text-tertiary); background: var(--surface-3);
}
.plugin-status.active { color: var(--success); background: rgba(34,197,94,0.12); }
.plugin-desc { color: var(--text-secondary); font-size: 13px; line-height: 1.5; }
.plugin-footer { display: flex; justify-content: space-between; align-items: center; }
.plugin-type {
  padding: 2px 8px; border-radius: 100px;
  font-size: 10px; font-weight: 700; text-transform: uppercase;
  color: var(--brand-light); background: rgba(45,212,191,0.12);
}
.empty-state {
  display: grid; place-items: center; align-content: center; gap: 12px;
  min-height: 360px; text-align: center; color: var(--text-tertiary);
}
.empty-state h2 { color: var(--text-primary); }
.empty-mark {
  display: inline-flex; align-items: center; justify-content: center;
  min-width: 48px; height: 48px; border: 1px solid var(--border);
  border-radius: var(--radius); background: var(--surface-2);
  color: var(--brand-light); font-family: var(--font-mono); font-weight: 900;
}
.modal-overlay {
  position: fixed; inset: 0; z-index: 40;
  display: grid; place-items: center;
  background: rgba(0,0,0,0.66); backdrop-filter: blur(5px);
}
.modal {
  width: min(460px, calc(100vw - 32px));
  border: 1px solid var(--border); border-radius: var(--radius-lg);
  background: var(--surface-1); box-shadow: var(--shadow-lg); padding: 20px;
}
.modal-head { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
.close-btn { border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--surface-1); color: var(--text-secondary); cursor: pointer; padding: 6px 10px; font-weight: 700; }
.close-btn:hover { border-color: var(--brand); color: var(--brand-light); }
.form-stack { display: grid; gap: 14px; }
.form-field { display: grid; gap: 6px; }
.form-field span { color: var(--text-secondary); font-size: 12px; font-weight: 800; }
.toast {
  position: fixed; bottom: 18px; left: 50%; transform: translateX(-50%);
  z-index: 80; min-width: min(380px, calc(100vw - 32px));
  border: 1px solid rgba(45,212,191,0.36); border-radius: var(--radius);
  background: rgba(15,118,110,0.96); color: white;
  box-shadow: var(--shadow-lg); padding: 12px 14px; text-align: center;
}
.toast.error { border-color: rgba(239,68,68,0.42); background: rgba(127,29,29,0.96); }
.fade-enter-active, .fade-leave-active { transition: opacity 0.2s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
@media (max-width: 720px) {
  .plugin-page { padding: 22px 16px; }
  .page-header { flex-direction: column; }
  .plugin-grid { grid-template-columns: 1fr; }
}
</style>
