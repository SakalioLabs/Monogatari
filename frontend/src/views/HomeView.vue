<template>
  <div class="dashboard">
    <header class="dash-header">
      <div>
        <h1>Dashboard</h1>
        <p class="dash-subtitle">Monogatari LLM Galgame Engine</p>
      </div>
      <div class="dash-actions">
        <button class="btn btn-secondary btn-sm" @click="refreshStatus">Refresh</button>
        <button class="btn btn-primary btn-sm" @click="$router.push('/chat')">Start Chat</button>
      </div>
    </header>

    <section class="stat-row">
      <div class="stat-card" v-for="s in stats" :key="s.label">
        <div class="stat-value" :style="{ color: s.color }">{{ s.value }}</div>
        <div class="stat-label">{{ s.label }}</div>
      </div>
    </section>

    <section class="grid-2">
      <div class="card feature-card" v-for="f in features" :key="f.title" @click="$router.push(f.path)">
        <div class="fc-header">
          <span class="fc-icon" :style="{ background: f.bg }" v-html="f.icon"></span>
          <div>
            <h3>{{ f.title }}</h3>
            <p>{{ f.desc }}</p>
          </div>
        </div>
        <span class="fc-arrow">&rarr;</span>
      </div>
    </section>

    <section class="card engine-card">
      <div class="ec-header">
        <h3>Engine Status</h3>
        <span v-if="status?.initialized" class="badge badge-accent">Online</span>
        <span v-else class="badge badge-warning">Not Initialized</span>
      </div>
      <div class="ec-grid" v-if="status">
        <div class="ec-item"><span class="ec-num">{{ status.character_count }}</span><span class="ec-lbl">Characters</span></div>
        <div class="ec-item"><span class="ec-num">{{ status.dialogue_count }}</span><span class="ec-lbl">Dialogues</span></div>
        <div class="ec-item"><span class="ec-num">{{ status.knowledge_count }}</span><span class="ec-lbl">Knowledge</span></div>
        <div class="ec-item"><span class="ec-num ec-ai">{{ status.active_ai_engine || 'Not set' }}</span><span class="ec-lbl">AI Backend</span></div>
      </div>
      <div v-else class="ec-loading"><div class="spinner"></div></div>
    </section>

    <footer class="dash-footer">
      <span>Monogatari v0.2.0</span>
      <span>Rust + Tauri + Vue 3</span>
    </footer>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface EngineStatus {
  initialized: boolean; character_count: number; dialogue_count: number;
  knowledge_count: number; ai_engines: string[]; active_ai_engine: string | null;
}

const status = ref<EngineStatus | null>(null)

const stats = computed(() => [
  { label: 'Characters', value: status.value?.character_count ?? '-', color: 'var(--brand-light)' },
  { label: 'Dialogues', value: status.value?.dialogue_count ?? '-', color: 'var(--accent)' },
  { label: 'Knowledge', value: status.value?.knowledge_count ?? '-', color: 'var(--warning)' },
  { label: 'AI Engine', value: status.value?.active_ai_engine ?? 'N/A', color: 'var(--info)' },
])

const features = [
  { title: 'AI Chat', desc: 'Talk with LLM-driven characters. Conversations scored for events.', path: '/chat', icon: '&#9993;', bg: 'var(--brand)' },
  { title: 'Story Mode', desc: 'Play branching visual novels with AI-enhanced dialogue.', path: '/game', icon: '&#9654;', bg: 'var(--accent)' },
  { title: 'Workflow Editor', desc: 'Design stories visually with drag-and-drop nodes.', path: '/editor', icon: '&#9776;', bg: 'var(--warning)' },
  { title: 'Settings', desc: 'Configure AI models, API keys, and engine options.', path: '/settings', icon: '&#9881;', bg: 'var(--info)' },
]

async function refreshStatus() {
  try { status.value = await invoke('get_engine_status') } catch (e) { console.error(e) }
}

onMounted(refreshStatus)
</script>

<style scoped>
.dashboard { max-width: 960px; margin: 0 auto; padding: 32px 40px; }
.dash-header { display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 32px; }
.dash-header h1 { font-size: 24px; font-weight: 700; color: var(--text-primary); }
.dash-subtitle { font-size: 13px; color: var(--text-tertiary); margin-top: 2px; }
.dash-actions { display: flex; gap: 8px; }
.stat-row { display: grid; grid-template-columns: repeat(4, 1fr); gap: 16px; margin-bottom: 32px; }
.stat-card { background: var(--surface-1); border: 1px solid var(--border); border-radius: var(--radius); padding: 20px; text-align: center; }
.stat-value { font-size: 28px; font-weight: 800; margin-bottom: 4px; }
.stat-label { font-size: 12px; color: var(--text-tertiary); text-transform: uppercase; letter-spacing: 0.05em; }
.grid-2 { display: grid; grid-template-columns: repeat(2, 1fr); gap: 16px; margin-bottom: 32px; }
.feature-card { display: flex; justify-content: space-between; align-items: center; cursor: pointer; transition: all var(--transition); }
.feature-card:hover { border-color: var(--brand); box-shadow: var(--shadow-brand); transform: translateY(-2px); }
.fc-header { display: flex; gap: 14px; align-items: center; }
.fc-icon { width: 40px; height: 40px; border-radius: var(--radius-sm); display: flex; align-items: center; justify-content: center; font-size: 18px; color: white; flex-shrink: 0; }
.fc-header h3 { font-size: 14px; font-weight: 600; color: var(--text-primary); margin-bottom: 2px; }
.fc-header p { font-size: 12px; color: var(--text-secondary); }
.fc-arrow { font-size: 18px; color: var(--text-tertiary); transition: all var(--transition-fast); }
.feature-card:hover .fc-arrow { color: var(--brand); transform: translateX(4px); }
.engine-card { margin-bottom: 32px; }
.ec-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }
.ec-header h3 { font-size: 15px; font-weight: 600; }
.ec-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 16px; }
.ec-item { text-align: center; padding: 16px; background: var(--surface-2); border-radius: var(--radius-sm); }
.ec-num { display: block; font-size: 24px; font-weight: 700; color: var(--brand-light); margin-bottom: 4px; }
.ec-ai { font-size: 13px; color: var(--accent); }
.ec-lbl { font-size: 11px; color: var(--text-tertiary); text-transform: uppercase; }
.ec-loading { display: flex; justify-content: center; padding: 24px; }
.dash-footer { display: flex; justify-content: space-between; font-size: 12px; color: var(--text-tertiary); padding-top: 16px; border-top: 1px solid var(--border); }
@media (max-width: 768px) { .stat-row, .ec-grid { grid-template-columns: repeat(2, 1fr); } .grid-2 { grid-template-columns: 1fr; } .dashboard { padding: 20px; } }
</style>
