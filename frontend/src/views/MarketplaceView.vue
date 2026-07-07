<template>
  <div class="marketplace-page">
    <header class="page-header">
      <div>
        <span class="eyebrow">Templates</span>
        <h1>{{ t("marketplace.title", "Marketplace") }}</h1>
        <p>Browse and import shareable workflows, characters, and project templates.</p>
      </div>
      <div class="header-actions">
        <select v-model="filterType" class="input input-sm">
          <option value="">All Types</option>
          <option value="full_project">Full Projects</option>
          <option value="character">Characters</option>
          <option value="workflow">Workflows</option>
        </select>
        <button class="btn btn-secondary btn-sm" @click="refresh">Refresh</button>
      </div>
    </header>
    <div v-if="loading" class="loading-state"><div class="spinner"></div><span>Loading templates...</span></div>
    <div v-else-if="entries.length === 0" class="empty-state"><p>No templates found.</p></div>
    <div v-else class="entry-grid">
      <div v-for="entry in filtered" :key="entry.id" class="entry-card" @click="selected = entry">
        <div class="entry-icon">{{ entry.entry_type.charAt(0).toUpperCase() }}</div>
        <div class="entry-info">
          <h3>{{ entry.name }}</h3>
          <p class="entry-desc">{{ entry.description }}</p>
          <div class="entry-meta">
            <span class="tag">{{ entry.entry_type }}</span>
            <span v-for="t in entry.tags.slice(0, 3)" :key="t" class="tag">{{ t }}</span>
          </div>
          <div class="entry-stats">
            <span>{{ entry.author }}</span>
            <span>v{{ entry.version }}</span>
            <span>{{ entry.rating.toFixed(1) }}</span>
          </div>
        </div>
      </div>
    </div>
    <div v-if="selected" class="detail-overlay" @click.self="selected = null">
      <div class="detail-panel">
        <div class="detail-header">
          <h2>{{ selected.name }}</h2>
          <button class="btn btn-ghost btn-sm" @click="selected = null">Close</button>
        </div>
        <div class="detail-body">
          <div class="field"><label>Description</label><p>{{ selected.description }}</p></div>
          <div class="field"><label>Author</label><p>{{ selected.author }}</p></div>
          <div class="field"><label>Type</label><p>{{ selected.entry_type }}</p></div>
          <div class="field"><label>Tags</label><div class="tag-list"><span v-for="t in selected.tags" :key="t" class="tag">{{ t }}</span></div></div>
          <div class="field"><label>Rating</label><p>{{ selected.rating.toFixed(1) }} / 5.0</p></div>
          <div class="actions"><button class="btn btn-primary" @click="importEntry(selected)">Import Template</button></div>
        </div>
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invokeCommand } from '../lib/tauri'
import { useI18n } from '../lib/i18n'

const { t } = useI18n()
interface MarketEntry { id: string; name: string; description: string; author: string; version: string; entry_type: string; tags: string[]; download_count: number; rating: number }
const entries = ref<MarketEntry[]>([])
const loading = ref(true)
const filterType = ref('')
const selected = ref<MarketEntry | null>(null)
const filtered = computed(() => filterType.value ? entries.value.filter(e => e.entry_type === filterType.value) : entries.value)
const preview: MarketEntry[] = [
  { id: 'sakura_demo', name: 'Sakura Park Demo', description: 'Complete demo with Sakura character, park scene, and cherry blossom dialogue', author: 'Monogatari', version: '1.0.0', entry_type: 'full_project', tags: ['demo', 'romance', 'nature'], download_count: 0, rating: 5.0 },
  { id: 'luna_stargazing', name: 'Luna Stargazing', description: 'Luna character with observatory scene and constellation dialogue', author: 'Monogatari', version: '1.0.0', entry_type: 'character', tags: ['sci-fi', 'poetic', 'night'], download_count: 0, rating: 4.5 },
]
async function refresh() {
  loading.value = true
  try { entries.value = await invokeCommand<MarketEntry[]>('list_marketplace_entries', { entryType: filterType.value || undefined }, preview) } catch { entries.value = preview }
  loading.value = false
}
async function importEntry(entry: MarketEntry) {
  try { await invokeCommand('import_template', { templatePath: entry.id }); alert('Imported: ' + entry.name) } catch (e) { console.error(e) }
}
onMounted(refresh)
</script>
<style scoped>
.marketplace-page { max-width: 1180px; margin: 0 auto; padding: 34px 40px; }
.page-header { display: flex; justify-content: space-between; gap: 18px; align-items: flex-start; margin-bottom: 22px; }
.page-header h1 { color: var(--text-primary); font-size: 28px; line-height: 1.15; }
.page-header p { color: var(--text-tertiary); font-size: 13px; margin-top: 4px; }
.eyebrow { display: block; color: var(--text-tertiary); font-size: 11px; font-weight: 800; text-transform: uppercase; }
.header-actions { display: flex; gap: 8px; flex-shrink: 0; align-items: center; }
.input-sm { padding: 5px 10px; font-size: 12px; }
.loading-state, .empty-state { display: flex; align-items: center; gap: 12px; justify-content: center; padding: 80px 0; color: var(--text-secondary); }
.entry-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(340px, 1fr)); gap: 16px; }
.entry-card { display: flex; gap: 16px; padding: 20px; background: var(--surface-1); border: 1px solid var(--border); border-radius: var(--radius); cursor: pointer; transition: all var(--transition-fast); }
.entry-card:hover { border-color: var(--brand); background: var(--surface-2); }
.entry-icon { width: 48px; height: 48px; border-radius: var(--radius-sm); display: flex; align-items: center; justify-content: center; font-size: 20px; font-weight: 700; color: white; flex-shrink: 0; background: var(--brand); }
.entry-info { flex: 1; min-width: 0; }
.entry-info h3 { font-size: 15px; font-weight: 700; margin-bottom: 4px; }
.entry-desc { font-size: 12px; color: var(--text-secondary); margin-bottom: 8px; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; }
.entry-meta { display: flex; gap: 4px; flex-wrap: wrap; margin-bottom: 6px; }
.entry-stats { display: flex; gap: 12px; font-size: 11px; color: var(--text-tertiary); }
.detail-overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.6); display: flex; align-items: center; justify-content: center; z-index: 100; }
.detail-panel { background: var(--surface-1); border: 1px solid var(--border); border-radius: var(--radius-lg); width: 90%; max-width: 500px; max-height: 80vh; overflow-y: auto; }
.detail-header { display: flex; align-items: center; justify-content: space-between; padding: 24px; border-bottom: 1px solid var(--border); }
.detail-header h2 { font-size: 20px; font-weight: 700; }
.detail-body { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.field label { display: block; font-size: 11px; font-weight: 700; color: var(--text-tertiary); text-transform: uppercase; margin-bottom: 4px; }
.field p { font-size: 13px; line-height: 1.6; color: var(--text-secondary); }
.tag-list { display: flex; gap: 4px; flex-wrap: wrap; }
.actions { margin-top: 8px; }
</style>
