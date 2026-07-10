<template>
  <div class="plugin-workbench">
    <header class="workbench-header">
      <div class="title-lockup">
        <span class="title-icon"><Blocks :size="17" /></span>
        <div>
          <span class="eyebrow">{{ t('plugins.eyebrow', 'Extensibility') }}</span>
          <h1>{{ t('plugins.title', 'Plugins') }}</h1>
        </div>
      </div>

      <div class="header-metrics">
        <span><strong>{{ plugins.length }}</strong>{{ t('plugins.total', 'Installed') }}</span>
        <span><strong>{{ enabledCount }}</strong>{{ t('plugins.enabled', 'Enabled') }}</span>
        <span><strong>{{ pluginCategories.length }}</strong>{{ t('plugins.types', 'Types') }}</span>
      </div>

      <div class="header-actions">
        <button class="icon-command" :disabled="loading" :title="t('common.refresh', 'Refresh')" :aria-label="t('common.refresh', 'Refresh')" @click="refreshPlugins(true)"><RefreshCw :class="{ spinner: loading }" :size="14" /></button>
        <button class="primary-command" :title="t('plugins.register', 'Register Plugin')" :aria-label="t('plugins.register', 'Register Plugin')" @click="openRegister"><Plus :size="14" /><span>{{ t('plugins.register', 'Register Plugin') }}</span></button>
      </div>
    </header>

    <aside class="plugin-rail" :aria-label="t('plugins.catalog', 'Plugin catalog')">
      <div class="rail-search">
        <label class="search-field">
          <Search :size="14" />
          <input v-model="searchQuery" :placeholder="t('plugins.search', 'Search plugins...')" />
          <button v-if="searchQuery" :title="t('plugins.clear-search', 'Clear search')" :aria-label="t('plugins.clear-search', 'Clear search')" @click="searchQuery = ''"><X :size="12" /></button>
        </label>
      </div>

      <div class="status-filters" :aria-label="t('plugins.filters', 'Plugin filters')">
        <button :class="{ active: statusFilter === 'all' }" @click="statusFilter = 'all'">{{ t('plugins.filter-all', 'All') }}</button>
        <button :class="{ active: statusFilter === 'enabled' }" @click="statusFilter = 'enabled'">{{ t('plugins.enabled', 'Enabled') }}</button>
        <button :class="{ active: statusFilter === 'disabled' }" @click="statusFilter = 'disabled'">{{ t('plugins.disabled', 'Disabled') }}</button>
      </div>

      <div class="rail-summary">{{ t('plugins.visible-count', '{visible} of {total}', { visible: filteredPlugins.length, total: plugins.length }) }}</div>

      <div class="plugin-list">
        <div v-if="loading && plugins.length === 0" class="rail-empty"><LoaderCircle class="spinner" :size="19" /><span>{{ t('common.loading', 'Loading...') }}</span></div>
        <div v-else-if="filteredPlugins.length === 0" class="rail-empty"><Blocks :size="21" /><span>{{ plugins.length === 0 ? t('plugins.empty', 'No plugins installed') : t('plugins.no-results', 'No plugins found') }}</span></div>

        <button
          v-for="plugin in filteredPlugins"
          v-else
          :key="plugin.id"
          class="plugin-row"
          :class="{ selected: selectedPlugin?.id === plugin.id }"
          :aria-current="selectedPlugin?.id === plugin.id ? 'true' : undefined"
          @click="selectPlugin(plugin)"
        >
          <span class="plugin-glyph"><Blocks :size="14" /></span>
          <span class="plugin-copy">
            <strong>{{ plugin.name }}</strong>
            <small>{{ plugin.id }}</small>
            <span>{{ categoryLabel(plugin.category || plugin.node_type) }}</span>
          </span>
          <span class="status-dot" :class="{ enabled: plugin.enabled }" :title="plugin.enabled ? t('plugins.enabled', 'Enabled') : t('plugins.disabled', 'Disabled')"></span>
        </button>
      </div>
    </aside>

    <main class="plugin-inspector" :class="{ 'compact-open': compactInspectorOpen }" :aria-label="t('plugins.inspector', 'Plugin inspector')">
      <header v-if="registering" class="inspector-header">
        <div>
          <span class="eyebrow">{{ t('plugins.registering', 'Register plugin') }}</span>
          <h2>{{ registerName || registerId || t('plugins.new-plugin', 'New plugin') }}</h2>
        </div>
        <div class="inspector-actions">
          <button class="secondary-command" :disabled="busy" @click="cancelRegister"><X :size="14" />{{ t('common.cancel', 'Cancel') }}</button>
          <button class="primary-command" :disabled="busy || !canRegister" @click="doRegister"><LoaderCircle v-if="busy" class="spinner" :size="14" /><Save v-else :size="14" />{{ busy ? t('plugins.registering-progress', 'Registering') : t('plugins.register', 'Register Plugin') }}</button>
        </div>
      </header>

      <header v-else-if="selectedPlugin" class="inspector-header">
        <div>
          <span class="eyebrow">{{ t('plugins.inspector', 'Plugin inspector') }}</span>
          <h2>{{ selectedPlugin.name }}</h2>
        </div>
        <div class="inspector-actions">
          <button class="secondary-command" :disabled="busy" :title="selectedPlugin.enabled ? t('plugins.disable', 'Disable') : t('plugins.enable', 'Enable')" @click="togglePlugin(selectedPlugin)"><Power :size="14" />{{ selectedPlugin.enabled ? t('plugins.disable', 'Disable') : t('plugins.enable', 'Enable') }}</button>
          <button class="icon-command danger-command" :disabled="busy" :title="t('common.delete', 'Delete')" :aria-label="t('common.delete', 'Delete')" @click="requestRemove(selectedPlugin)"><Trash2 :size="14" /></button>
          <button class="icon-command inspector-close" :title="t('plugins.close-inspector', 'Close inspector')" :aria-label="t('plugins.close-inspector', 'Close inspector')" @click="compactInspectorOpen = false"><X :size="14" /></button>
        </div>
      </header>

      <header v-else class="inspector-header empty-header">
        <div><span class="eyebrow">{{ t('plugins.inspector', 'Plugin inspector') }}</span><h2>{{ t('plugins.no-selection', 'No plugin selected') }}</h2></div>
      </header>

      <form v-if="registering" class="inspector-scroll register-form" @submit.prevent="doRegister">
        <div class="form-grid">
          <label class="form-field">
            <span>{{ t('plugins.name', 'Plugin name') }}</span>
            <input v-model.trim="registerName" class="input" maxlength="128" :placeholder="t('plugins.name-placeholder', 'Custom workflow node')" @input="syncRegisterId" />
          </label>
          <label class="form-field">
            <span>{{ t('plugins.id', 'Plugin ID') }}</span>
            <input v-model.trim="registerId" class="input mono" maxlength="64" :placeholder="t('plugins.id-placeholder', 'my_custom_node')" @input="registerIdManual = true" />
          </label>
        </div>

        <label class="form-field">
          <span>{{ t('plugins.type', 'Plugin type') }}</span>
          <select v-model="registerType" class="input">
            <option value="node">{{ t('plugins.type.node', 'Custom node') }}</option>
            <option value="trigger">{{ t('plugins.type.trigger', 'Event trigger') }}</option>
            <option value="action">{{ t('plugins.type.action', 'Action handler') }}</option>
          </select>
        </label>

        <label class="form-field">
          <span>{{ t('plugins.description', 'Description') }}</span>
          <textarea v-model="registerDesc" class="input" rows="7" maxlength="2048" :placeholder="t('plugins.description-placeholder', 'Plugin purpose and behavior')"></textarea>
        </label>

        <p v-if="registerValidation" class="validation-message"><AlertTriangle :size="13" />{{ registerValidation }}</p>
      </form>

      <div v-else-if="selectedPlugin" class="inspector-scroll plugin-detail">
        <div class="detail-identity">
          <span class="detail-icon"><Blocks :size="18" /></span>
          <div>
            <h3>{{ selectedPlugin.name }}</h3>
            <code>{{ selectedPlugin.id }}</code>
          </div>
          <span class="status-badge" :class="{ enabled: selectedPlugin.enabled }"><span></span>{{ selectedPlugin.enabled ? t('plugins.enabled', 'Enabled') : t('plugins.disabled', 'Disabled') }}</span>
        </div>

        <p class="plugin-description">{{ selectedPlugin.description || t('plugins.no-description', 'No description') }}</p>

        <div class="detail-facts">
          <span><strong>{{ t('plugins.version', 'Version') }}</strong>{{ selectedPlugin.version || '1.0.0' }}</span>
          <span><strong>{{ t('plugins.author', 'Author') }}</strong>{{ selectedPlugin.author || t('plugins.local-author', 'Local creator') }}</span>
          <span><strong>{{ t('plugins.type', 'Plugin type') }}</strong>{{ categoryLabel(selectedPlugin.category || selectedPlugin.node_type) }}</span>
          <span><strong>{{ t('plugins.node-type', 'Node type') }}</strong><code>{{ selectedPlugin.node_type }}</code></span>
        </div>

        <section v-if="selectedPlugin.script_path" class="detail-section">
          <span class="section-label">{{ t('plugins.script-path', 'Script path') }}</span>
          <code class="script-path">{{ selectedPlugin.script_path }}</code>
        </section>

        <section class="detail-section">
          <div class="section-heading"><span>{{ t('plugins.fields', 'Configurable fields') }}</span><strong>{{ selectedPlugin.configurable_fields.length }}</strong></div>
          <div v-if="selectedPlugin.configurable_fields.length" class="field-list">
            <div v-for="field in selectedPlugin.configurable_fields" :key="field.name">
              <span><strong>{{ field.label || field.name }}</strong><code>{{ field.name }}</code></span>
              <small>{{ field.field_type }} · {{ field.required ? t('plugins.required', 'Required') : t('plugins.optional', 'Optional') }}</small>
            </div>
          </div>
          <div v-else class="compact-empty"><SlidersHorizontal :size="19" /><span>{{ t('plugins.no-fields', 'No configurable fields') }}</span></div>
        </section>
      </div>

      <div v-else class="inspector-empty"><Blocks :size="23" /><span>{{ t('plugins.no-selection', 'No plugin selected') }}</span></div>
    </main>

    <Transition name="fade">
      <div v-if="pendingRemoval" class="modal-backdrop" @click.self="cancelRemove">
        <section class="confirm-dialog" role="dialog" aria-modal="true" :aria-label="t('common.confirm', 'Confirm')">
          <Trash2 :size="19" />
          <div><span class="eyebrow">{{ t('common.confirm', 'Confirm') }}</span><h2>{{ t('plugins.remove-confirm', 'Remove "{name}"?', { name: pendingRemoval.name }) }}</h2></div>
          <footer>
            <button class="secondary-command" :disabled="busy" @click="cancelRemove"><X :size="14" />{{ t('common.cancel', 'Cancel') }}</button>
            <button class="danger-action" :disabled="busy" @click="confirmRemove"><LoaderCircle v-if="busy" class="spinner" :size="14" /><Trash2 v-else :size="14" />{{ t('plugins.remove', 'Remove') }}</button>
          </footer>
        </section>
      </div>
    </Transition>

    <Transition name="fade"><button v-if="statusMessage" class="status-toast" :class="{ error: !statusOk }" @click="statusMessage = null">{{ statusMessage }}</button></Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { AlertTriangle, Blocks, LoaderCircle, Plus, Power, RefreshCw, Save, Search, SlidersHorizontal, Trash2, X } from '@lucide/vue'
import { useI18n } from '../lib/i18n'
import { hasTauriRuntime, invokeCommand } from '../lib/tauri'

interface PluginField {
  name: string
  field_type: string
  label: string
  default_value: unknown
  required: boolean
}

interface PluginManifest {
  id: string
  name: string
  version: string
  author: string
  description: string
  node_type: string
  category: string
  configurable_fields: PluginField[]
  script_path: string | null
  enabled: boolean
}

type StatusFilter = 'all' | 'enabled' | 'disabled'

const { t } = useI18n()
const plugins = ref<PluginManifest[]>([])
const selectedPlugin = ref<PluginManifest | null>(null)
const registering = ref(false)
const compactInspectorOpen = ref(false)
const searchQuery = ref('')
const statusFilter = ref<StatusFilter>('all')
const registerName = ref('')
const registerId = ref('')
const registerIdManual = ref(false)
const registerType = ref('node')
const registerDesc = ref('')
const loading = ref(false)
const busy = ref(false)
const pendingRemoval = ref<PluginManifest | null>(null)
const statusMessage = ref<string | null>(null)
const statusOk = ref(true)
const browserPluginsKey = 'monogatari.previewPlugins'
let statusTimer: number | null = null

const enabledCount = computed(() => plugins.value.filter(plugin => plugin.enabled).length)
const pluginCategories = computed(() => [...new Set(plugins.value.map(plugin => plugin.category || plugin.node_type).filter(Boolean))])
const filteredPlugins = computed(() => {
  const query = searchQuery.value.trim().toLowerCase()
  return plugins.value.filter(plugin => {
    if (statusFilter.value === 'enabled' && !plugin.enabled) return false
    if (statusFilter.value === 'disabled' && plugin.enabled) return false
    if (!query) return true
    return plugin.id.toLowerCase().includes(query)
      || plugin.name.toLowerCase().includes(query)
      || plugin.description.toLowerCase().includes(query)
      || plugin.category.toLowerCase().includes(query)
      || plugin.node_type.toLowerCase().includes(query)
  })
})
const registerValidation = computed(() => {
  if (!registerName.value.trim()) return t('plugins.validation.name', 'Plugin name is required.')
  if (!/^[a-z0-9_-]{1,64}$/.test(registerId.value.trim())) return t('plugins.validation.id', 'Plugin ID must use lowercase letters, numbers, underscores, or hyphens.')
  return ''
})
const canRegister = computed(() => !registerValidation.value)

async function refreshPlugins(showNotice = false) {
  loading.value = true
  try {
    plugins.value = hasTauriRuntime()
      ? await invokeCommand<PluginManifest[]>('list_plugins', undefined, [])
      : loadBrowserPlugins()
    const selectedId = selectedPlugin.value?.id
    selectedPlugin.value = plugins.value.find(plugin => plugin.id === selectedId) || plugins.value[0] || null
    if (showNotice) notify(true, t('plugins.notice.refreshed', 'Plugin catalog refreshed.'))
  } catch (error) {
    plugins.value = []
    selectedPlugin.value = null
    notify(false, t('plugins.notice.load-failed', 'Plugins could not be loaded: {error}', { error: String(error) }))
  } finally {
    loading.value = false
  }
}

function selectPlugin(plugin: PluginManifest) {
  registering.value = false
  selectedPlugin.value = plugin
  compactInspectorOpen.value = true
}

function openRegister() {
  registerName.value = ''
  registerId.value = ''
  registerIdManual.value = false
  registerType.value = 'node'
  registerDesc.value = ''
  registering.value = true
  compactInspectorOpen.value = true
}

function cancelRegister() {
  registering.value = false
}

function syncRegisterId() {
  if (!registerIdManual.value) registerId.value = pluginIdFromName(registerName.value)
}

function pluginIdFromName(name: string): string {
  return name
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9_-]+/g, '_')
    .replace(/^[-_]+|[-_]+$/g, '')
    .slice(0, 64)
}

function pluginManifestPayload(): PluginManifest {
  const id = registerId.value.trim() || pluginIdFromName(registerName.value) || 'custom_plugin'
  return {
    id,
    name: registerName.value.trim(),
    version: '1.0.0',
    author: t('plugins.local-author', 'Local creator'),
    description: registerDesc.value.trim(),
    node_type: id,
    category: registerType.value,
    configurable_fields: [],
    script_path: null,
    enabled: true,
  }
}

async function doRegister() {
  if (!canRegister.value || busy.value) return
  busy.value = true
  const manifest = pluginManifestPayload()
  try {
    if (hasTauriRuntime()) {
      await invokeCommand<void>('register_plugin', { manifest: pluginManifestPayload() })
    } else {
      saveBrowserPlugins([...loadBrowserPlugins().filter(plugin => plugin.id !== manifest.id), manifest])
    }
    registering.value = false
    await refreshPlugins()
    selectedPlugin.value = plugins.value.find(plugin => plugin.id === manifest.id) || null
    notify(true, t('plugins.notice.registered', 'Plugin "{name}" registered.', { name: manifest.name }))
  } catch (error) {
    notify(false, t('plugins.notice.register-failed', 'Plugin could not be registered: {error}', { error: String(error) }))
  } finally {
    busy.value = false
  }
}

async function togglePlugin(plugin: PluginManifest) {
  if (busy.value) return
  busy.value = true
  const updated = { ...plugin, enabled: !plugin.enabled }
  try {
    if (hasTauriRuntime()) await invokeCommand<void>('register_plugin', { manifest: updated })
    else saveBrowserPlugins([...loadBrowserPlugins().filter(item => item.id !== plugin.id), updated])
    await refreshPlugins()
    selectedPlugin.value = plugins.value.find(item => item.id === updated.id) || null
    notify(true, updated.enabled
      ? t('plugins.notice.enabled', 'Plugin "{name}" enabled.', { name: updated.name })
      : t('plugins.notice.disabled', 'Plugin "{name}" disabled.', { name: updated.name }))
  } catch (error) {
    notify(false, t('plugins.notice.update-failed', 'Plugin could not be updated: {error}', { error: String(error) }))
  } finally {
    busy.value = false
  }
}

function requestRemove(plugin: PluginManifest) {
  pendingRemoval.value = plugin
}

function cancelRemove() {
  if (!busy.value) pendingRemoval.value = null
}

async function confirmRemove() {
  const plugin = pendingRemoval.value
  if (!plugin || busy.value) return
  await removePlugin(plugin.id, plugin.name)
}

async function removePlugin(id: string, name: string) {
  busy.value = true
  try {
    if (hasTauriRuntime()) {
      await invokeCommand<void>('remove_plugin', { pluginId: id })
    } else {
      saveBrowserPlugins(loadBrowserPlugins().filter(plugin => plugin.id !== id))
    }
    pendingRemoval.value = null
    await refreshPlugins()
    notify(true, t('plugins.notice.removed', 'Plugin "{name}" removed.', { name }))
  } catch (error) {
    notify(false, t('plugins.notice.remove-failed', 'Plugin could not be removed: {error}', { error: String(error) }))
  } finally {
    busy.value = false
  }
}

function loadBrowserPlugins(): PluginManifest[] {
  try {
    const parsed = JSON.parse(localStorage.getItem(browserPluginsKey) || '[]')
    return Array.isArray(parsed) ? parsed as PluginManifest[] : []
  } catch {
    localStorage.removeItem(browserPluginsKey)
    return []
  }
}

function saveBrowserPlugins(value: PluginManifest[]) {
  localStorage.setItem(browserPluginsKey, JSON.stringify(value))
}

function categoryLabel(category: string): string {
  if (category === 'node') return t('plugins.type.node', 'Custom node')
  if (category === 'trigger') return t('plugins.type.trigger', 'Event trigger')
  if (category === 'action') return t('plugins.type.action', 'Action handler')
  return category
}

function notify(ok: boolean, message: string) {
  statusOk.value = ok
  statusMessage.value = message
  if (statusTimer !== null) window.clearTimeout(statusTimer)
  statusTimer = window.setTimeout(() => { statusMessage.value = null }, 3600)
}

onMounted(() => refreshPlugins())
onUnmounted(() => {
  if (statusTimer !== null) window.clearTimeout(statusTimer)
})
</script>

<style scoped>
.plugin-workbench { position: relative; display: grid; height: calc(100svh - 56px); min-height: 0; grid-template-columns: 300px minmax(0, 1fr); grid-template-rows: 54px minmax(0, 1fr); overflow: hidden; background: var(--surface-0); }
.workbench-header { display: grid; min-width: 0; grid-column: 1 / -1; grid-template-columns: minmax(190px, 1fr) auto auto; align-items: center; gap: 12px; padding: 7px 11px; border-bottom: 1px solid var(--border); background: var(--surface-1); }
.title-lockup, .header-metrics, .header-actions, .inspector-actions, .validation-message, .section-heading, .confirm-dialog footer { display: flex; align-items: center; }
.title-lockup { min-width: 0; gap: 9px; }
.title-icon, .plugin-glyph, .detail-icon { display: inline-grid; flex: 0 0 auto; place-items: center; border-radius: 6px; background: color-mix(in srgb, var(--brand) 14%, var(--surface-2)); color: var(--brand-light); }
.title-icon { width: 32px; height: 32px; }
.title-lockup > div, .inspector-header > div:first-child { display: grid; min-width: 0; gap: 2px; }
.eyebrow { color: var(--text-tertiary); font-size: 8px; font-weight: 800; letter-spacing: 0; text-transform: uppercase; }
.title-lockup h1, .inspector-header h2 { margin: 0; overflow: hidden; color: var(--text-primary); text-overflow: ellipsis; white-space: nowrap; }
.title-lockup h1 { font-size: 14px; line-height: 1.2; }
.header-metrics { gap: 4px; }
.header-metrics > span { display: grid; min-width: 62px; gap: 1px; padding: 3px 7px; border-left: 1px solid var(--border); color: var(--text-tertiary); font-size: 7px; text-transform: uppercase; }
.header-metrics strong { color: var(--text-primary); font-family: var(--font-mono); font-size: 10px; }
.header-actions, .inspector-actions { justify-content: flex-end; gap: 5px; }
.icon-command, .primary-command, .secondary-command, .danger-action { display: inline-flex; min-width: 0; align-items: center; justify-content: center; gap: 6px; border-radius: 6px; font: inherit; font-size: 9px; font-weight: 800; cursor: pointer; white-space: nowrap; }
.icon-command { width: 32px; height: 32px; flex: 0 0 32px; padding: 0; border: 1px solid var(--border); background: var(--surface-2); color: var(--text-secondary); }
.primary-command, .secondary-command, .danger-action { min-height: 32px; padding: 0 10px; }
.primary-command { border: 1px solid var(--brand); background: var(--brand); color: var(--surface-0); }
.secondary-command { border: 1px solid var(--border); background: var(--surface-2); color: var(--text-secondary); }
.danger-action { border: 1px solid var(--danger); background: var(--danger); color: white; }
.icon-command:hover:not(:disabled), .secondary-command:hover:not(:disabled) { border-color: var(--border-strong); color: var(--text-primary); }
.primary-command:hover:not(:disabled) { border-color: var(--brand-light); background: var(--brand-light); }
.danger-command:hover:not(:disabled) { border-color: color-mix(in srgb, var(--danger) 48%, var(--border)); color: var(--danger); }
.icon-command:disabled, .primary-command:disabled, .secondary-command:disabled, .danger-action:disabled { cursor: not-allowed; opacity: 0.42; }

.plugin-rail, .plugin-inspector { min-width: 0; min-height: 0; overflow: hidden; }
.plugin-rail { display: grid; grid-template-rows: 42px 36px 28px minmax(0, 1fr); border-right: 1px solid var(--border); background: var(--surface-1); }
.rail-search { padding: 6px 8px; border-bottom: 1px solid var(--border); }
.search-field { display: flex; height: 30px; align-items: center; gap: 6px; padding: 0 7px; border: 1px solid var(--border); border-radius: 6px; background: var(--surface-0); color: var(--text-tertiary); }
.search-field:focus-within { border-color: var(--border-strong); color: var(--text-secondary); }
.search-field input { width: 100%; min-width: 0; border: 0; outline: 0; background: transparent; color: var(--text-primary); font: inherit; font-size: 9px; }
.search-field input::placeholder { color: var(--text-tertiary); }
.search-field button { display: inline-grid; width: 22px; height: 22px; flex: 0 0 22px; place-items: center; padding: 0; border: 0; background: transparent; color: var(--text-tertiary); cursor: pointer; }
.status-filters { display: flex; align-items: center; gap: 3px; padding: 4px 7px; border-bottom: 1px solid var(--border); }
.status-filters button { min-width: 0; height: 27px; flex: 1; padding: 0 4px; border: 1px solid transparent; border-radius: 5px; background: transparent; color: var(--text-tertiary); font: inherit; font-size: 8px; cursor: pointer; }
.status-filters button.active { border-color: var(--border); background: var(--surface-2); color: var(--text-primary); }
.rail-summary { padding: 4px 9px; color: var(--text-tertiary); font-size: 8px; }
.plugin-list, .inspector-scroll { min-height: 0; overflow-y: auto; scrollbar-width: none; }
.plugin-list::-webkit-scrollbar, .inspector-scroll::-webkit-scrollbar { display: none; }
.plugin-list { padding: 4px 6px 8px; }
.plugin-row { display: grid; width: 100%; min-width: 0; min-height: 58px; grid-template-columns: 32px minmax(0, 1fr) 12px; align-items: center; gap: 8px; margin-bottom: 3px; padding: 7px; border: 1px solid transparent; border-radius: 6px; background: transparent; color: inherit; font: inherit; text-align: left; cursor: pointer; }
.plugin-row:hover { background: var(--surface-2); }
.plugin-row.selected { border-color: color-mix(in srgb, var(--brand) 34%, var(--border)); background: color-mix(in srgb, var(--brand) 8%, var(--surface-1)); }
.plugin-glyph { width: 32px; height: 32px; }
.plugin-copy { display: grid; min-width: 0; gap: 2px; }
.plugin-copy strong, .plugin-copy small, .plugin-copy span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.plugin-copy strong { color: var(--text-primary); font-size: 9px; }
.plugin-copy small { color: var(--text-tertiary); font-family: var(--font-mono); font-size: 7px; }
.plugin-copy span { color: var(--text-secondary); font-size: 7px; }
.status-dot { width: 7px; height: 7px; border-radius: 50%; background: var(--text-tertiary); }
.status-dot.enabled { background: var(--success); box-shadow: 0 0 0 3px color-mix(in srgb, var(--success) 12%, transparent); }
.rail-empty, .inspector-empty, .compact-empty { display: grid; min-height: 100%; place-items: center; align-content: center; gap: 7px; padding: 18px; color: var(--text-tertiary); font-size: 9px; text-align: center; }

.plugin-inspector { display: grid; grid-template-rows: 54px minmax(0, 1fr); background: var(--surface-1); }
.inspector-header { display: flex; min-width: 0; align-items: center; justify-content: space-between; gap: 9px; padding: 7px 10px; border-bottom: 1px solid var(--border); }
.inspector-header h2 { max-width: 520px; font-size: 11px; }
.inspector-close { display: none; }
.empty-header { justify-content: flex-start; }
.inspector-scroll { padding: 14px; }
.register-form { display: grid; align-content: start; gap: 13px; }
.form-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 10px; }
.form-field { display: grid; min-width: 0; gap: 5px; }
.form-field > span, .section-label { color: var(--text-tertiary); font-size: 8px; font-weight: 800; text-transform: uppercase; }
.input { width: 100%; min-width: 0; min-height: 34px; padding: 7px 8px; border: 1px solid var(--border); border-radius: 6px; outline: 0; background: var(--surface-0); color: var(--text-primary); font: inherit; font-size: 9px; }
.input:focus { border-color: var(--border-strong); box-shadow: var(--shadow-brand); }
.input[type='text'] { line-height: 1.4; }
textarea.input { min-height: 150px; resize: vertical; line-height: 1.55; }
.mono { font-family: var(--font-mono); }
.validation-message { gap: 6px; margin: 0; color: var(--warning); font-size: 8px; line-height: 1.45; }
.plugin-detail { display: grid; align-content: start; gap: 16px; }
.detail-identity { display: grid; min-width: 0; grid-template-columns: 40px minmax(0, 1fr) auto; align-items: center; gap: 10px; padding-bottom: 14px; border-bottom: 1px solid var(--border); }
.detail-icon { width: 40px; height: 40px; }
.detail-identity > div { display: grid; min-width: 0; gap: 3px; }
.detail-identity h3 { margin: 0; overflow: hidden; color: var(--text-primary); font-size: 15px; text-overflow: ellipsis; white-space: nowrap; }
.detail-identity code, .script-path { color: var(--text-tertiary); font-family: var(--font-mono); font-size: 8px; overflow-wrap: anywhere; }
.status-badge { display: inline-flex; align-items: center; gap: 5px; padding: 4px 7px; border: 1px solid var(--border); border-radius: 999px; color: var(--text-tertiary); font-size: 8px; font-weight: 800; }
.status-badge > span { width: 6px; height: 6px; border-radius: 50%; background: currentColor; }
.status-badge.enabled { border-color: color-mix(in srgb, var(--success) 28%, var(--border)); color: var(--success); }
.plugin-description { margin: 0; color: var(--text-secondary); font-size: 10px; line-height: 1.6; }
.detail-facts { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); border-top: 1px solid var(--border); border-left: 1px solid var(--border); }
.detail-facts > span { display: grid; min-width: 0; gap: 4px; padding: 9px; border-right: 1px solid var(--border); border-bottom: 1px solid var(--border); color: var(--text-secondary); font-size: 9px; overflow-wrap: anywhere; }
.detail-facts strong { color: var(--text-tertiary); font-size: 7px; text-transform: uppercase; }
.detail-facts code { font-family: var(--font-mono); font-size: 8px; }
.detail-section { display: grid; gap: 8px; }
.section-heading { justify-content: space-between; gap: 8px; color: var(--text-tertiary); font-size: 8px; font-weight: 800; text-transform: uppercase; }
.section-heading strong { color: var(--text-primary); font-family: var(--font-mono); font-size: 8px; }
.field-list { display: grid; gap: 4px; }
.field-list > div { display: grid; min-width: 0; grid-template-columns: minmax(0, 1fr) auto; gap: 4px 8px; padding: 8px; border-bottom: 1px solid var(--border); }
.field-list span { display: grid; min-width: 0; gap: 2px; }
.field-list strong { color: var(--text-primary); font-size: 9px; }
.field-list code, .field-list small { color: var(--text-tertiary); font-family: var(--font-mono); font-size: 7px; }
.compact-empty { min-height: 130px; }

.modal-backdrop { position: absolute; z-index: 80; inset: 0; display: grid; place-items: center; padding: 16px; background: color-mix(in srgb, var(--surface-0) 82%, transparent); }
.confirm-dialog { display: grid; width: min(390px, 100%); grid-template-columns: 30px minmax(0, 1fr); align-items: start; gap: 9px; padding: 16px; border: 1px solid var(--border-strong); border-radius: 6px; background: var(--surface-1); box-shadow: var(--shadow-lg); }
.confirm-dialog > svg { color: var(--danger); }
.confirm-dialog > div { display: grid; min-width: 0; gap: 4px; }
.confirm-dialog h2 { margin: 0; color: var(--text-primary); font-size: 13px; line-height: 1.45; }
.confirm-dialog footer { grid-column: 1 / -1; justify-content: flex-end; gap: 6px; margin-top: 4px; padding-top: 11px; border-top: 1px solid var(--border); }
.status-toast { position: fixed; z-index: 100; right: 18px; bottom: 18px; max-width: min(440px, calc(100vw - 28px)); padding: 9px 11px; border: 1px solid color-mix(in srgb, var(--success) 38%, var(--border)); border-radius: 6px; background: color-mix(in srgb, var(--success) 18%, var(--surface-1)); color: var(--text-primary); box-shadow: var(--shadow-lg); font: inherit; font-size: 9px; cursor: pointer; }
.status-toast.error { border-color: color-mix(in srgb, var(--danger) 44%, var(--border)); background: color-mix(in srgb, var(--danger) 18%, var(--surface-1)); }
.fade-enter-active, .fade-leave-active { transition: opacity 0.18s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
.spinner { animation: spin 0.8s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }

@media (max-width: 760px) {
  .plugin-workbench { height: calc(100svh - 56px - 60px - env(safe-area-inset-bottom, 0px)); grid-template-columns: 1fr; grid-template-rows: 56px minmax(0, 1fr); }
  .workbench-header { grid-template-columns: minmax(0, 1fr) auto; gap: 7px; padding: 7px 9px; }
  .header-metrics { display: none; }
  .header-actions { gap: 4px; }
  .header-actions .primary-command { width: 34px; padding: 0; }
  .header-actions .primary-command span { display: none; }
  .plugin-rail { grid-row: 2; border-right: 0; }
  .plugin-inspector { position: absolute; z-index: 40; inset: 56px 0 0; display: none; border-top: 1px solid var(--border-strong); box-shadow: var(--shadow-lg); }
  .plugin-inspector.compact-open { display: grid; }
  .inspector-close { display: inline-flex; }
  .inspector-header h2 { max-width: min(40vw, 180px); }
  .form-grid { grid-template-columns: 1fr; }
  .inspector-scroll { padding: 11px; }
  .status-toast { right: 14px; bottom: calc(70px + env(safe-area-inset-bottom, 0px)); }
}

@media (max-width: 430px) {
  .inspector-actions { gap: 3px; }
  .inspector-actions .secondary-command { width: 34px; padding: 0; font-size: 0; }
  .inspector-actions .secondary-command svg { width: 14px; height: 14px; }
  .detail-facts { grid-template-columns: 1fr; }
}
</style>
