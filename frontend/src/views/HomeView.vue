<template>
  <div class="dashboard">
    <header class="dash-header">
      <div>
        <span class="eyebrow">Monogatari Engine v0.8</span>
        <h1>Production Desk</h1>
        <p>LLM-driven visual novel engine. Create characters, build stories, and let AI drive the narrative.</p>
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

    <section class="quick-actions">
      <span class="eyebrow">Quick Actions</span>
      <div class="action-row">
        <button class="action-btn" @click="$router.push('/chat')">
          <span class="action-icon">C</span>
          <span>Start AI Chat</span>
        </button>
        <button class="action-btn" @click="$router.push('/editor')">
          <span class="action-icon">W</span>
          <span>Edit Workflow</span>
        </button>
        <button class="action-btn" @click="$router.push('/character-editor')">
          <span class="action-icon">E</span>
          <span>Edit Characters</span>
        </button>
        <button class="action-btn" @click="$router.push('/knowledge')">
          <span class="action-icon">K</span>
          <span>Knowledge Base</span>
        </button>
        <button class="action-btn" @click="$router.push('/settings')">
          <span class="action-icon">S</span>
          <span>Settings</span>
        </button>
        <button class="action-btn" @click="$router.push('/game')">
          <span class="action-icon">P</span>
          <span>Preview Story</span>
        </button>
      </div>
    </section>

    <section class="desk-grid">
      <button v-for="f in features" :key="f.title" class="desk-tile" @click="$router.push(f.path)">
        <span class="tile-icon" :style="{ background: f.color }">{{ f.icon }}</span>
        <span class="tile-copy">
          <strong>{{ f.title }}</strong>
          <span>{{ f.desc }}</span>
        </span>
        <span class="tile-arrow">&rsaquo;</span>
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
          <span><b>Scenes</b>{{ sceneCount }}</span>
          <span><b>AI Engines</b>{{ status?.ai_engines?.length ?? 0 }}</span>
        </div>
        <div class="pipeline-actions">
          <button class="btn btn-secondary btn-sm" @click="$router.push('/settings')">Configure AI</button>
          <button class="btn btn-secondary btn-sm" @click="$router.push('/assets')">Scene Assets</button>
        </div>
      </div>
    </section>

    <section class="getting-started" v-if="!status?.initialized">
      <div class="panel-head">
        <span class="eyebrow">Getting Started</span>
        <strong>Quick Setup Guide</strong>
      </div>
      <div class="steps-grid">
        <div class="step-item" v-for="(step, i) in setupSteps" :key="i" :class="{ completed: step.done }">
          <span class="step-num">{{ i + 1 }}</span>
          <div class="step-content">
            <strong>{{ step.title }}</strong>
            <p>{{ step.desc }}</p>
          </div>
          <button class="btn btn-sm" :class="step.done ? 'btn-secondary' : 'btn-primary'" @click="$router.push(step.route)">
            {{ step.done ? 'View' : 'Start' }}
          </button>
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
const sceneCount = ref(0)

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
  { title: 'AI Chat', desc: 'Streaming character sessions and relationship state.', path: '/chat', icon: 'C', color: 'rgba(45,212,191,0.2)' },
  { title: 'Story Mode', desc: 'Branching dialogue playback with visual novel controls.', path: '/game', icon: 'S', color: 'rgba(96,165,250,0.2)' },
  { title: 'Workflow', desc: 'Node graph authoring for story logic and triggers.', path: '/editor', icon: 'W', color: 'rgba(251,146,60,0.2)' },
  { title: 'Characters', desc: 'Professional editor with personality, emotions, knowledge.', path: '/character-editor', icon: 'E', color: 'rgba(168,85,247,0.2)' },
  { title: 'Knowledge', desc: 'World lore, character backgrounds, and AI context entries.', path: '/knowledge', icon: 'K', color: 'rgba(34,197,94,0.2)' },
  { title: 'Scene Assets', desc: 'Background catalog, scene metadata, and runtime selection.', path: '/assets', icon: 'A', color: 'rgba(244,114,182,0.2)' },
  { title: 'Group Chat', desc: 'Multi-character simultaneous conversations.', path: '/group-chat', icon: 'M', color: 'rgba(251,191,36,0.2)' },
  { title: 'Analytics', desc: 'Player engagement metrics and conversation patterns.', path: '/analytics', icon: 'D', color: 'rgba(56,189,248,0.2)' },
  { title: 'Settings', desc: 'Project config, AI backend, TTS, cloud sync.', path: '/settings', icon: 'T', color: 'rgba(148,163,184,0.2)' },
  { title: 'Marketplace', desc: 'Community templates, characters, and story modules.', path: '/marketplace', icon: 'K', color: 'rgba(253,186,116,0.2)' },
  { title: 'Plugins', desc: 'Custom workflow node types and event handlers.', path: '/plugins', icon: 'P', color: 'rgba(129,140,248,0.2)' },
  { title: 'Audio', desc: 'BGM, SFX, and voice management with mixer.', path: '/audio', icon: 'B', color: 'rgba(244,63,94,0.2)' },
]

const readinessItems = computed(() => [
  { name: 'Core runtime architecture', done: true },
  { name: 'Commercial workbench UI', done: true },
  { name: 'Streaming chat and evaluation', done: true },
  { name: 'Visual workflow editor', done: true },
  { name: 'Scene asset management', done: true },
  { name: 'Multi-character group chat', done: true },
  { name: 'Knowledge base management', done: true },
  { name: 'Professional character editor', done: true },
  { name: 'TTS integration scaffold', done: true },
  { name: '21 workflow node types', done: true },
  { name: 'Analytics dashboard', done: true },
  { name: 'i18n multi-language scaffold', done: true },
  { name: 'Plugin system for custom nodes', done: true },
  { name: 'Cloud save sync', done: true },
  { name: 'Installer signing and distribution', done: false },
  { name: 'Mobile deployment (Tauri mobile)', done: false },
])

const commercialProgress = computed(() => {
  const done = readinessItems.value.filter((item) => item.done).length
  return Math.round((done / readinessItems.value.length) * 100)
})

const setupSteps = computed(() => [
  { title: 'Configure AI Backend', desc: 'Set up OpenAI-compatible API or local ONNX model in Settings.', route: '/settings', done: !!status.value?.active_ai_engine },
  { title: 'Create Characters', desc: 'Design characters with personality, background, and knowledge.', route: '/character-editor', done: (status.value?.character_count ?? 0) > 0 },
  { title: 'Build Knowledge Base', desc: 'Add world lore and context for AI-driven storytelling.', route: '/knowledge', done: (status.value?.knowledge_count ?? 0) > 0 },
  { title: 'Design Workflows', desc: 'Create branching story logic with drag-and-drop nodes.', route: '/editor', done: false },
  { title: 'Test in Chat', desc: 'Chat with characters and watch evaluation scores.', route: '/chat', done: false },
  { title: 'Preview Story', desc: 'Run the visual novel runtime with Live2D and dialogue.', route: '/game', done: false },
])

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
    try {
      const scenes = await invokeCommand<any[]>('list_scene_assets', undefined, [])
      sceneCount.value = scenes.length
    } catch { sceneCount.value = 0 }
  } catch (e) {
    console.error(e)
  }
}

onMounted(refreshStatus)
</script>

<style scoped>
.dashboard {
  max-width: 1280px;
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
  font-weight: 700;
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

.quick-actions {
  margin-bottom: 16px;
}

.quick-actions > .eyebrow {
  margin-bottom: 10px;
}

.action-row {
  display: grid;
  grid-template-columns: repeat(6, minmax(0, 1fr));
  gap: 10px;
}

.action-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 16px 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-1);
  color: var(--text-primary);
  cursor: pointer;
  font-size: 12px;
  font-weight: 600;
  transition: border-color var(--transition-fast), background var(--transition-fast), transform var(--transition-fast);
}

.action-btn:hover {
  border-color: var(--brand);
  background: var(--surface-2);
  transform: translateY(-2px);
}

.action-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border-radius: var(--radius-sm);
  background: var(--surface-3);
  color: var(--brand-light);
  font-weight: 900;
  font-size: 16px;
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
  margin-bottom: 16px;
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
  transition: width var(--transition);
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

.pipeline-actions {
  display: flex;
  gap: 8px;
  margin-top: 14px;
}

.getting-started {
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-1);
  padding: 18px;
}

.steps-grid {
  display: grid;
  gap: 12px;
  margin-top: 14px;
}

.step-item {
  display: grid;
  grid-template-columns: 32px 1fr auto;
  gap: 14px;
  align-items: center;
  padding: 14px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-2);
}

.step-item.completed {
  border-color: rgba(34,197,94,0.28);
  background: rgba(34,197,94,0.05);
}

.step-num {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: var(--surface-3);
  color: var(--brand-light);
  font-weight: 800;
  font-size: 14px;
}

.step-item.completed .step-num {
  background: var(--success);
  color: white;
}

.step-content strong {
  display: block;
  color: var(--text-primary);
  font-size: 13px;
  margin-bottom: 2px;
}

.step-content p {
  color: var(--text-tertiary);
  font-size: 12px;
  line-height: 1.3;
}

@media (max-width: 1060px) {
  .action-row {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }
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
  .action-row {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
  .desk-grid,
  .status-metrics {
    grid-template-columns: 1fr;
  }
}
</style>
