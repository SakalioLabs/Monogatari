<template>
  <div class="dashboard">
    <header class="dash-header">
      <div>
        <span class="eyebrow">Monogatari Engine</span>
        <h1>Production Desk</h1>
        <p>LLM character runtime, story workflow, knowledge context, and game session tools.</p>
      </div>
      <div class="dash-actions">
        <button class="btn btn-secondary btn-sm" @click="refreshStatus">Refresh</button>
        <button class="btn btn-primary btn-sm" @click="$router.push('/chat')">Open Chat</button>
      </div>
    </header>

    <section class="status-strip">
      <div class="status-copy">
        <span class="eyebrow">Runtime</span>
        <strong>{{ statusLabel }}</strong>
      </div>
      <div class="status-metrics">
        <div v-for="s in stats" :key="s.label" class="stat-card">
          <span class="stat-value">{{ s.value }}</span>
          <span class="stat-label">{{ s.label }}</span>
        </div>
      </div>
    </section>

    <section class="desk-grid">
      <button v-for="f in features" :key="f.title" class="desk-tile" @click="$router.push(f.path)">
        <span class="tile-icon">{{ f.icon }}</span>
        <span class="tile-copy">
          <strong>{{ f.title }}</strong>
          <span>{{ f.desc }}</span>
        </span>
        <span class="tile-arrow">›</span>
      </button>
    </section>

    <section class="ops-grid">
      <div class="ops-panel">
        <div class="panel-head">
          <span class="eyebrow">Commercialization</span>
          <strong>{{ commercialProgress }}%</strong>
        </div>
        <div class="progress-track"><div class="progress-fill" :style="{ width: commercialProgress + '%' }"></div></div>
        <div class="readiness-list">
          <span v-for="item in readinessItems" :key="item.name" :class="{ done: item.done }">
            <b>{{ item.done ? 'Done' : 'Next' }}</b>{{ item.name }}
          </span>
        </div>
      </div>

      <div class="ops-panel">
        <div class="panel-head">
          <span class="eyebrow">Pipeline</span>
          <strong>{{ status?.active_ai_engine || 'Unset' }}</strong>
        </div>
        <div class="pipeline-list">
          <span><b>Characters</b>{{ status?.character_count ?? '-' }}</span>
          <span><b>Dialogues</b>{{ status?.dialogue_count ?? '-' }}</span>
          <span><b>Knowledge</b>{{ status?.knowledge_count ?? '-' }}</span>
          <span><b>AI Engines</b>{{ status?.ai_engines?.length ?? 0 }}</span>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { invokeCommand } from '../lib/tauri'

interface EngineStatus {
  initialized: boolean
  character_count: number
  dialogue_count: number
  knowledge_count: number
  ai_engines: string[]
  active_ai_engine: string | null
}

const status = ref<EngineStatus | null>(null)

const statusLabel = computed(() => {
  if (!status.value) return 'Checking engine'
  return status.value.initialized ? 'Ready for authoring' : 'Waiting for project data'
})

const stats = computed(() => [
  { label: 'Characters', value: status.value?.character_count ?? '-' },
  { label: 'Dialogues', value: status.value?.dialogue_count ?? '-' },
  { label: 'Knowledge', value: status.value?.knowledge_count ?? '-' },
  { label: 'Backend', value: status.value?.active_ai_engine ?? 'N/A' },
])

const features = [
  { title: 'AI Chat', desc: 'Streaming character sessions and relationship state.', path: '/chat', icon: 'C' },
  { title: 'Story Mode', desc: 'Branching dialogue playback with visual novel controls.', path: '/game', icon: 'S' },
  { title: 'Workflow', desc: 'Node graph authoring for story logic and triggers.', path: '/editor', icon: 'W' },
  { title: 'Scene Assets', desc: 'Background catalog, scene metadata, and runtime selection.', path: '/assets', icon: 'A' },
  { title: 'Settings', desc: 'Project config, path readiness, AI backend, and runtime setup.', path: '/settings', icon: 'T' },
]

const readinessItems = computed(() => [
  { name: 'Core runtime architecture', done: true },
  { name: 'Commercial workbench UI', done: true },
  { name: 'Streaming chat and evaluation surface', done: true },
  { name: 'Browser preview fallback', done: true },
  { name: 'Workflow import validation', done: true },
  { name: 'Scene asset management', done: true },
  { name: 'Project configuration panel', done: true },
  { name: 'Multi-character group chat', done: true },
  { name: 'Character gallery with personality visualization', done: true },
  { name: 'TTS integration scaffold', done: true },
  { name: '21 workflow node types with execution', done: true },
  { name: 'Commercial packaging checklist', done: false },
])

const commercialProgress = computed(() => {
  const done = readinessItems.value.filter((item) => item.done).length
  return Math.round((done / readinessItems.value.length) * 100)
})

async function refreshStatus() {
  try {
    status.value = await invokeCommand<EngineStatus>('get_engine_status', undefined, {
      initialized: false,
      character_count: 0,
      dialogue_count: 0,
      knowledge_count: 0,
      ai_engines: [],
      active_ai_engine: null,
    })
  } catch (e) {
    console.error(e)
  }
}

onMounted(refreshStatus)
</script>

<style scoped>
.dashboard {
  max-width: 1180px;
  margin: 0 auto;
  padding: 34px 40px;
}

.dash-header {
  display: flex;
  justify-content: space-between;
  gap: 20px;
  align-items: flex-start;
  margin-bottom: 24px;
}

.eyebrow {
  display: block;
  color: var(--text-tertiary);
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0;
  text-transform: uppercase;
}

.dash-header h1 {
  margin-top: 3px;
  color: var(--text-primary);
  font-size: 28px;
  line-height: 1.15;
}

.dash-header p {
  max-width: 620px;
  color: var(--text-secondary);
  font-size: 13px;
  margin-top: 7px;
}

.dash-actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

.status-strip {
  display: grid;
  grid-template-columns: 240px minmax(0, 1fr);
  gap: 16px;
  align-items: stretch;
  margin-bottom: 16px;
}

.status-copy,
.stat-card,
.desk-tile,
.ops-panel {
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-1);
}

.status-copy {
  display: grid;
  align-content: center;
  gap: 6px;
  padding: 18px;
}

.status-copy strong {
  color: var(--brand-light);
  font-size: 18px;
}

.status-metrics {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12px;
}

.stat-card {
  display: grid;
  align-content: center;
  gap: 4px;
  min-height: 86px;
  padding: 16px;
}

.stat-value {
  overflow: hidden;
  color: var(--text-primary);
  font-size: 24px;
  font-weight: 850;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.stat-label {
  color: var(--text-tertiary);
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
}

.desk-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12px;
  margin-bottom: 16px;
}

.desk-tile {
  display: grid;
  grid-template-columns: 38px minmax(0, 1fr) auto;
  gap: 12px;
  align-items: center;
  min-height: 104px;
  padding: 16px;
  color: var(--text-primary);
  cursor: pointer;
  text-align: left;
  transition: border-color var(--transition-fast), transform var(--transition-fast), background var(--transition-fast);
}

.desk-tile:hover {
  border-color: var(--brand);
  background: var(--surface-2);
  transform: translateY(-1px);
}

.tile-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 38px;
  height: 38px;
  border-radius: var(--radius-sm);
  background: var(--surface-3);
  color: var(--brand-light);
  font-weight: 900;
}

.tile-copy {
  min-width: 0;
  display: grid;
  gap: 4px;
}

.tile-copy strong {
  color: var(--text-primary);
  font-size: 14px;
}

.tile-copy span {
  color: var(--text-tertiary);
  font-size: 12px;
  line-height: 1.35;
}

.tile-arrow {
  color: var(--text-tertiary);
  font-size: 24px;
}

.ops-grid {
  display: grid;
  grid-template-columns: 1.3fr 1fr;
  gap: 16px;
}

.ops-panel {
  padding: 18px;
}

.panel-head {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  align-items: baseline;
  margin-bottom: 14px;
}

.panel-head strong {
  color: var(--text-primary);
}

.progress-track {
  height: 8px;
  overflow: hidden;
  border-radius: 999px;
  background: var(--surface-3);
}

.progress-fill {
  height: 100%;
  border-radius: inherit;
  background: var(--brand);
}

.readiness-list,
.pipeline-list {
  display: grid;
  gap: 10px;
  margin-top: 16px;
}

.readiness-list span,
.pipeline-list span {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  color: var(--text-secondary);
  font-size: 13px;
}

.readiness-list b,
.pipeline-list b {
  color: var(--text-tertiary);
  font-size: 12px;
}

.readiness-list .done {
  color: var(--text-primary);
}

.readiness-list .done b {
  color: var(--success);
}

@media (max-width: 980px) {
  .status-strip,
  .ops-grid {
    grid-template-columns: 1fr;
  }

  .desk-grid,
  .status-metrics {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 640px) {
  .dashboard {
    padding: 22px;
  }

  .dash-header {
    display: grid;
  }

  .desk-grid,
  .status-metrics {
    grid-template-columns: 1fr;
  }
}
</style>
