<template>
  <div class="market-workbench">
    <header class="workbench-header">
      <div class="title-lockup">
        <span class="title-icon"><Store :size="17" /></span>
        <div>
          <span class="eyebrow">{{ t('marketplace.eyebrow', 'Project templates') }}</span>
          <h1>{{ t('marketplace.title', 'Marketplace') }}</h1>
        </div>
      </div>

      <div class="header-metrics">
        <span><strong>{{ entries.length }}</strong>{{ t('marketplace.templates', 'Templates') }}</span>
        <span><strong>{{ entryTypes.length }}</strong>{{ t('marketplace.types', 'Types') }}</span>
        <span><strong>{{ importedIds.length }}</strong>{{ t('marketplace.imported', 'Imported') }}</span>
      </div>

      <button class="icon-command" :disabled="loading" :title="t('common.refresh', 'Refresh')" :aria-label="t('common.refresh', 'Refresh')" @click="refresh(true)"><RefreshCw :class="{ spinner: loading }" :size="14" /></button>
    </header>

    <aside class="template-rail" :aria-label="t('marketplace.catalog', 'Template catalog')">
      <div class="rail-search">
        <label class="search-field">
          <Search :size="14" />
          <input v-model="searchQuery" :placeholder="t('marketplace.search', 'Search templates...')" />
          <button v-if="searchQuery" :title="t('marketplace.clear-search', 'Clear search')" :aria-label="t('marketplace.clear-search', 'Clear search')" @click="searchQuery = ''"><X :size="12" /></button>
        </label>
      </div>

      <div class="type-filter">
        <label>
          <LayoutGrid :size="12" />
          <select v-model="filterType" :aria-label="t('marketplace.filter-type', 'Template type')">
            <option value="">{{ t('marketplace.type.all', 'All types') }}</option>
            <option value="full_project">{{ t('marketplace.type.full-project', 'Full projects') }}</option>
            <option value="character">{{ t('marketplace.type.character', 'Characters') }}</option>
            <option value="workflow">{{ t('marketplace.type.workflow', 'Workflows') }}</option>
          </select>
        </label>
      </div>

      <div class="rail-summary">{{ t('marketplace.visible-count', '{visible} of {total}', { visible: filtered.length, total: entries.length }) }}</div>

      <div class="template-list">
        <div v-if="loading && entries.length === 0" class="rail-empty"><LoaderCircle class="spinner" :size="19" /><span>{{ t('marketplace.loading', 'Loading templates...') }}</span></div>
        <div v-else-if="filtered.length === 0" class="rail-empty"><PackageOpen :size="21" /><span>{{ t('marketplace.empty', 'No templates found') }}</span></div>

        <button
          v-for="entry in filtered"
          v-else
          :key="entry.id"
          class="template-row"
          :class="{ selected: selected?.id === entry.id }"
          :aria-current="selected?.id === entry.id ? 'true' : undefined"
          @click="selectEntry(entry)"
        >
          <span class="template-thumb">
            <img v-if="previewAvailable(entry)" :src="previewUrl(entry) || ''" :alt="entry.name" @error="markPreviewFailed(entry)" />
            <Package v-else :size="15" />
          </span>
          <span class="template-copy">
            <strong>{{ entry.name }}</strong>
            <small>{{ entry.id }}</small>
            <span>{{ typeLabel(entry.entry_type) }} · {{ entry.author }}</span>
          </span>
          <span v-if="isImported(entry.id)" class="imported-mark" :title="t('marketplace.imported', 'Imported')"><Check :size="12" /></span>
        </button>
      </div>
    </aside>

    <main class="template-inspector" :class="{ 'compact-open': compactInspectorOpen }" :aria-label="t('marketplace.inspector', 'Template inspector')">
      <header v-if="selected" class="inspector-header">
        <div>
          <span class="eyebrow">{{ t('marketplace.inspector', 'Template inspector') }}</span>
          <h2>{{ selected.name }}</h2>
        </div>
        <button class="icon-command inspector-close" :title="t('marketplace.close-inspector', 'Close inspector')" :aria-label="t('marketplace.close-inspector', 'Close inspector')" @click="compactInspectorOpen = false"><X :size="14" /></button>
      </header>

      <header v-else class="inspector-header empty-header">
        <div><span class="eyebrow">{{ t('marketplace.inspector', 'Template inspector') }}</span><h2>{{ t('marketplace.no-selection', 'No template selected') }}</h2></div>
      </header>

      <div v-if="selected" class="inspector-scroll template-detail">
        <div class="template-preview">
          <img v-if="previewAvailable(selected)" :src="previewUrl(selected) || ''" :alt="selected.name" @error="markPreviewFailed(selected)" />
          <div v-else><Package :size="28" /><span>{{ typeLabel(selected.entry_type) }}</span></div>
        </div>

        <div class="detail-identity">
          <div>
            <h3>{{ selected.name }}</h3>
            <code>{{ selected.id }}</code>
          </div>
          <button
            class="primary-command"
            :disabled="importingId === selected.id || isImported(selected.id)"
            :title="isImported(selected.id) ? t('marketplace.imported', 'Imported') : t('marketplace.import', 'Import Template')"
            @click="importEntry(selected)"
          >
            <LoaderCircle v-if="importingId === selected.id" class="spinner" :size="14" />
            <Check v-else-if="isImported(selected.id)" :size="14" />
            <Download v-else :size="14" />
            {{ importingId === selected.id ? t('marketplace.importing', 'Importing') : isImported(selected.id) ? t('marketplace.imported', 'Imported') : t('marketplace.import', 'Import Template') }}
          </button>
        </div>

        <p class="template-description">{{ selected.description }}</p>

        <div class="detail-facts">
          <span><UserRound :size="12" /><strong>{{ t('marketplace.author', 'Author') }}</strong>{{ selected.author }}</span>
          <span><Package :size="12" /><strong>{{ t('marketplace.type', 'Type') }}</strong>{{ typeLabel(selected.entry_type) }}</span>
          <span><GitBranch :size="12" /><strong>{{ t('marketplace.version', 'Version') }}</strong>{{ selected.version }}</span>
          <span><Star :size="12" /><strong>{{ t('marketplace.rating', 'Rating') }}</strong>{{ selected.rating.toFixed(1) }} / 5.0</span>
          <span><Download :size="12" /><strong>{{ t('marketplace.downloads', 'Downloads') }}</strong>{{ selected.download_count }}</span>
        </div>

        <section v-if="selected.tags.length" class="detail-section">
          <span class="section-label">{{ t('marketplace.tags', 'Tags') }}</span>
          <div class="tag-list"><span v-for="tag in selected.tags" :key="tag"><Hash :size="10" />{{ tag }}</span></div>
        </section>
      </div>

      <div v-else class="inspector-empty"><PackageOpen :size="23" /><span>{{ t('marketplace.no-selection', 'No template selected') }}</span></div>
    </main>

    <Transition name="fade"><button v-if="statusMessage" class="status-toast" :class="statusType" @click="statusMessage = ''">{{ statusMessage }}</button></Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { Check, Download, GitBranch, Hash, LayoutGrid, LoaderCircle, Package, PackageOpen, RefreshCw, Search, Star, Store, UserRound, X } from '@lucide/vue'
import { resolveAssetUrl } from '../lib/assets'
import { useI18n } from '../lib/i18n'
import { hasTauriRuntime, invokeCommand } from '../lib/tauri'

interface MarketEntry {
  id: string
  name: string
  description: string
  author: string
  version: string
  entry_type: string
  tags: string[]
  download_count: number
  rating: number
  preview_image_path?: string
}

const { t } = useI18n()
const entries = ref<MarketEntry[]>([])
const loading = ref(true)
const importingId = ref('')
const searchQuery = ref('')
const filterType = ref('')
const selected = ref<MarketEntry | null>(null)
const compactInspectorOpen = ref(false)
const importedIds = ref<string[]>([])
const failedPreviewUrls = ref<string[]>([])
const statusMessage = ref('')
const statusType = ref<'success' | 'error'>('success')
let statusTimer: number | null = null

const preview: MarketEntry[] = [
  {
    id: 'sakura_demo',
    name: 'Sakura Park Demo',
    description: 'Complete demo with Sakura character, park scene, and cherry blossom dialogue',
    author: 'Monogatari',
    version: '1.0.0',
    entry_type: 'full_project',
    tags: ['demo', 'romance', 'nature'],
    download_count: 0,
    rating: 5.0,
    preview_image_path: 'assets/backgrounds/sakura_park.svg',
  },
  {
    id: 'luna_stargazing',
    name: 'Luna Stargazing',
    description: 'Luna character with observatory scene and constellation dialogue',
    author: 'Monogatari',
    version: '1.0.0',
    entry_type: 'character',
    tags: ['sci-fi', 'poetic', 'night'],
    download_count: 0,
    rating: 4.5,
    preview_image_path: 'assets/backgrounds/studio_night.svg',
  },
]

const entryTypes = computed(() => [...new Set(entries.value.map(entry => entry.entry_type))])
const filtered = computed(() => {
  const query = searchQuery.value.trim().toLowerCase()
  return entries.value.filter(entry => (
    (!filterType.value || entry.entry_type === filterType.value)
    && (!query
      || entry.id.toLowerCase().includes(query)
      || entry.name.toLowerCase().includes(query)
      || entry.description.toLowerCase().includes(query)
      || entry.author.toLowerCase().includes(query)
      || entry.tags.some(tag => tag.toLowerCase().includes(query)))
  ))
})

async function refresh(showNotice = false) {
  loading.value = true
  try {
    entries.value = await invokeCommand<MarketEntry[]>('list_marketplace_entries', { entryType: undefined }, preview)
    const selectedId = selected.value?.id
    selected.value = entries.value.find(entry => entry.id === selectedId) || entries.value[0] || null
    failedPreviewUrls.value = []
    if (showNotice) notify('success', t('marketplace.notice.refreshed', 'Template catalog refreshed.'))
  } catch (error) {
    entries.value = preview
    selected.value = preview[0] || null
    notify('error', t('marketplace.notice.load-failed', 'Templates could not be loaded: {error}', { error: String(error) }))
  } finally {
    loading.value = false
  }
}

function selectEntry(entry: MarketEntry) {
  selected.value = entry
  compactInspectorOpen.value = true
}

async function importEntry(entry: MarketEntry) {
  if (importingId.value || isImported(entry.id)) return
  importingId.value = entry.id
  try {
    if (hasTauriRuntime()) {
      await invokeCommand('import_template', { templatePath: entry.id })
    }
    importedIds.value = [...new Set([...importedIds.value, entry.id])]
    notify('success', t('marketplace.notice.imported', 'Template "{name}" imported.', { name: entry.name }))
  } catch (error) {
    notify('error', t('marketplace.notice.import-failed', 'Template could not be imported: {error}', { error: String(error) }))
  } finally {
    importingId.value = ''
  }
}

function typeLabel(type: string): string {
  if (type === 'full_project') return t('marketplace.type.full-project', 'Full projects')
  if (type === 'character') return t('marketplace.type.character', 'Characters')
  if (type === 'workflow') return t('marketplace.type.workflow', 'Workflows')
  return type
}

function previewUrl(entry: MarketEntry): string | null {
  return resolveAssetUrl(entry.preview_image_path)
}

function previewAvailable(entry: MarketEntry): boolean {
  const url = previewUrl(entry)
  return Boolean(url && !failedPreviewUrls.value.includes(url))
}

function markPreviewFailed(entry: MarketEntry) {
  const url = previewUrl(entry)
  if (url && !failedPreviewUrls.value.includes(url)) failedPreviewUrls.value = [...failedPreviewUrls.value, url]
}

function isImported(id: string): boolean {
  return importedIds.value.includes(id)
}

function notify(type: 'success' | 'error', message: string) {
  statusType.value = type
  statusMessage.value = message
  if (statusTimer !== null) window.clearTimeout(statusTimer)
  statusTimer = window.setTimeout(() => { statusMessage.value = '' }, 3600)
}

onMounted(() => refresh())
onUnmounted(() => {
  if (statusTimer !== null) window.clearTimeout(statusTimer)
})
</script>

<style scoped>
.market-workbench { position: relative; display: grid; height: calc(100svh - 56px); min-height: 0; grid-template-columns: 320px minmax(0, 1fr); grid-template-rows: 54px minmax(0, 1fr); overflow: hidden; background: var(--surface-0); }
.workbench-header { display: grid; min-width: 0; grid-column: 1 / -1; grid-template-columns: minmax(190px, 1fr) auto 32px; align-items: center; gap: 12px; padding: 7px 11px; border-bottom: 1px solid var(--border); background: var(--surface-1); }
.title-lockup, .header-metrics, .detail-facts span, .tag-list, .tag-list span { display: flex; align-items: center; }
.title-lockup { min-width: 0; gap: 9px; }
.title-icon { display: inline-grid; width: 32px; height: 32px; flex: 0 0 32px; place-items: center; border-radius: 6px; background: color-mix(in srgb, var(--brand) 14%, var(--surface-2)); color: var(--brand-light); }
.title-lockup > div, .inspector-header > div { display: grid; min-width: 0; gap: 2px; }
.eyebrow { color: var(--text-tertiary); font-size: 8px; font-weight: 800; letter-spacing: 0; text-transform: uppercase; }
.title-lockup h1, .inspector-header h2 { margin: 0; overflow: hidden; color: var(--text-primary); text-overflow: ellipsis; white-space: nowrap; }
.title-lockup h1 { font-size: 14px; line-height: 1.2; }
.header-metrics { gap: 4px; }
.header-metrics > span { display: grid; min-width: 62px; gap: 1px; padding: 3px 7px; border-left: 1px solid var(--border); color: var(--text-tertiary); font-size: 7px; text-transform: uppercase; }
.header-metrics strong { color: var(--text-primary); font-family: var(--font-mono); font-size: 10px; }
.icon-command, .primary-command { display: inline-flex; min-width: 0; align-items: center; justify-content: center; gap: 6px; border-radius: 6px; font: inherit; font-size: 9px; font-weight: 800; cursor: pointer; white-space: nowrap; }
.icon-command { width: 32px; height: 32px; flex: 0 0 32px; padding: 0; border: 1px solid var(--border); background: var(--surface-2); color: var(--text-secondary); }
.primary-command { min-height: 32px; padding: 0 10px; border: 1px solid var(--brand); background: var(--brand); color: var(--surface-0); }
.icon-command:hover:not(:disabled) { border-color: var(--border-strong); color: var(--text-primary); }
.primary-command:hover:not(:disabled) { border-color: var(--brand-light); background: var(--brand-light); }
.icon-command:disabled, .primary-command:disabled { cursor: not-allowed; opacity: 0.42; }

.template-rail, .template-inspector { min-width: 0; min-height: 0; overflow: hidden; }
.template-rail { display: grid; grid-template-rows: 42px 38px 28px minmax(0, 1fr); border-right: 1px solid var(--border); background: var(--surface-1); }
.rail-search { padding: 6px 8px; border-bottom: 1px solid var(--border); }
.search-field { display: flex; height: 30px; align-items: center; gap: 6px; padding: 0 7px; border: 1px solid var(--border); border-radius: 6px; background: var(--surface-0); color: var(--text-tertiary); }
.search-field:focus-within { border-color: var(--border-strong); color: var(--text-secondary); }
.search-field input { width: 100%; min-width: 0; border: 0; outline: 0; background: transparent; color: var(--text-primary); font: inherit; font-size: 9px; }
.search-field input::placeholder { color: var(--text-tertiary); }
.search-field button { display: inline-grid; width: 22px; height: 22px; flex: 0 0 22px; place-items: center; padding: 0; border: 0; background: transparent; color: var(--text-tertiary); cursor: pointer; }
.type-filter { padding: 4px 7px; border-bottom: 1px solid var(--border); }
.type-filter label { display: flex; min-width: 0; height: 29px; align-items: center; gap: 5px; padding: 0 7px; border: 1px solid var(--border); border-radius: 5px; background: var(--surface-0); color: var(--text-tertiary); }
.type-filter select { width: 100%; min-width: 0; border: 0; outline: 0; background: transparent; color: var(--text-secondary); font: inherit; font-size: 8px; }
.rail-summary { padding: 4px 9px; color: var(--text-tertiary); font-size: 8px; }
.template-list, .inspector-scroll { min-height: 0; overflow-y: auto; scrollbar-width: none; }
.template-list::-webkit-scrollbar, .inspector-scroll::-webkit-scrollbar { display: none; }
.template-list { padding: 4px 6px 8px; }
.template-row { display: grid; width: 100%; min-width: 0; min-height: 64px; grid-template-columns: 64px minmax(0, 1fr) 16px; align-items: center; gap: 8px; margin-bottom: 3px; padding: 6px; border: 1px solid transparent; border-radius: 6px; background: transparent; color: inherit; font: inherit; text-align: left; cursor: pointer; }
.template-row:hover { background: var(--surface-2); }
.template-row.selected { border-color: color-mix(in srgb, var(--brand) 34%, var(--border)); background: color-mix(in srgb, var(--brand) 8%, var(--surface-1)); }
.template-thumb { display: grid; width: 64px; height: 44px; overflow: hidden; place-items: center; border-radius: 5px; background: var(--surface-3); color: var(--brand-light); }
.template-thumb img { width: 100%; height: 100%; object-fit: cover; }
.template-copy { display: grid; min-width: 0; gap: 2px; }
.template-copy strong, .template-copy small, .template-copy span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.template-copy strong { color: var(--text-primary); font-size: 9px; }
.template-copy small { color: var(--text-tertiary); font-family: var(--font-mono); font-size: 7px; }
.template-copy span { color: var(--text-secondary); font-size: 7px; }
.imported-mark { color: var(--success); }
.rail-empty, .inspector-empty { display: grid; min-height: 100%; place-items: center; align-content: center; gap: 7px; padding: 18px; color: var(--text-tertiary); font-size: 9px; text-align: center; }

.template-inspector { display: grid; grid-template-rows: 54px minmax(0, 1fr); background: var(--surface-1); }
.inspector-header { display: flex; min-width: 0; align-items: center; justify-content: space-between; gap: 9px; padding: 7px 10px; border-bottom: 1px solid var(--border); }
.inspector-header h2 { max-width: 520px; font-size: 11px; }
.inspector-close { display: none; }
.empty-header { justify-content: flex-start; }
.inspector-scroll { padding: 14px; }
.template-detail { display: grid; align-content: start; gap: 14px; }
.template-preview { display: grid; width: 100%; height: clamp(180px, 34vh, 320px); overflow: hidden; place-items: center; border: 1px solid var(--border); border-radius: 6px; background: var(--surface-0); }
.template-preview img { width: 100%; height: 100%; object-fit: contain; }
.template-preview > div { display: grid; justify-items: center; gap: 7px; color: var(--text-tertiary); font-size: 9px; }
.detail-identity { display: flex; min-width: 0; align-items: center; justify-content: space-between; gap: 10px; }
.detail-identity > div { display: grid; min-width: 0; gap: 3px; }
.detail-identity h3 { margin: 0; overflow: hidden; color: var(--text-primary); font-size: 15px; text-overflow: ellipsis; white-space: nowrap; }
.detail-identity code { color: var(--text-tertiary); font-family: var(--font-mono); font-size: 8px; }
.template-description { margin: 0; color: var(--text-secondary); font-size: 10px; line-height: 1.6; }
.detail-facts { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); border-top: 1px solid var(--border); border-left: 1px solid var(--border); }
.detail-facts span { min-width: 0; gap: 6px; padding: 8px; border-right: 1px solid var(--border); border-bottom: 1px solid var(--border); color: var(--text-secondary); font-size: 8px; }
.detail-facts svg { flex: 0 0 auto; color: var(--text-tertiary); }
.detail-facts strong { color: var(--text-tertiary); font-size: 7px; text-transform: uppercase; }
.detail-section { display: grid; gap: 8px; }
.section-label { color: var(--text-tertiary); font-size: 8px; font-weight: 800; text-transform: uppercase; }
.tag-list { flex-wrap: wrap; gap: 4px; }
.tag-list span { gap: 3px; padding: 4px 6px; border: 1px solid var(--border); border-radius: 5px; color: var(--text-secondary); font-size: 8px; }
.status-toast { position: fixed; z-index: 100; right: 18px; bottom: 18px; max-width: min(440px, calc(100vw - 28px)); padding: 9px 11px; border: 1px solid color-mix(in srgb, var(--success) 38%, var(--border)); border-radius: 6px; background: color-mix(in srgb, var(--success) 18%, var(--surface-1)); color: var(--text-primary); box-shadow: var(--shadow-lg); font: inherit; font-size: 9px; cursor: pointer; }
.status-toast.error { border-color: color-mix(in srgb, var(--danger) 44%, var(--border)); background: color-mix(in srgb, var(--danger) 18%, var(--surface-1)); }
.fade-enter-active, .fade-leave-active { transition: opacity 0.18s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
.spinner { animation: spin 0.8s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }

@media (max-width: 760px) {
  .market-workbench { height: calc(100svh - 56px - 60px - env(safe-area-inset-bottom, 0px)); grid-template-columns: 1fr; grid-template-rows: 56px minmax(0, 1fr); }
  .workbench-header { grid-template-columns: minmax(0, 1fr) 32px; gap: 7px; padding: 7px 9px; }
  .header-metrics { display: none; }
  .template-rail { grid-row: 2; border-right: 0; }
  .template-inspector { position: absolute; z-index: 40; inset: 56px 0 0; display: none; border-top: 1px solid var(--border-strong); box-shadow: var(--shadow-lg); }
  .template-inspector.compact-open { display: grid; }
  .inspector-close { display: inline-flex; }
  .inspector-scroll { padding: 10px; }
  .template-preview { height: clamp(170px, 30vh, 250px); }
  .detail-identity h3 { font-size: 13px; }
  .status-toast { right: 14px; bottom: calc(70px + env(safe-area-inset-bottom, 0px)); }
}

@media (max-width: 430px) {
  .detail-identity .primary-command { width: 34px; padding: 0; font-size: 0; }
  .detail-facts { grid-template-columns: 1fr; }
}
</style>
