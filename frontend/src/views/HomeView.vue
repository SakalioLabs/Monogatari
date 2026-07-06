<template>
  <div class="home">
    <header class="hero">
      <div class="hero-content">
        <h1>Monogatari</h1>
        <p class="subtitle">LLM-Powered Visual Novel Engine - Rust + Tauri + Live2D</p>
        <div class="hero-badges">
          <span class="badge">AI Dialogue</span>
          <span class="badge">Live2D</span>
          <span class="badge">Visual Editor</span>
          <span class="badge">High Performance</span>
        </div>
      </div>
    </header>

    <div class="features">
      <div class="card feature-card" @click="$router.push('/chat')">
        <div class="feature-icon">C</div>
        <h3>AI Chat</h3>
        <p>Talk freely with AI-driven characters. Your conversations are scored to unlock special events.</p>
        <div class="feature-arrow">&rarr;</div>
      </div>
      <div class="card feature-card" @click="$router.push('/game')">
        <div class="feature-icon">G</div>
        <h3>Story Mode</h3>
        <p>Experience branching visual novel stories with AI-enhanced dialogue.</p>
        <div class="feature-arrow">&rarr;</div>
      </div>
      <div class="card feature-card" @click="$router.push('/editor')">
        <div class="feature-icon">W</div>
        <h3>Workflow Editor</h3>
        <p>Create stories visually with drag-and-drop nodes. No coding required.</p>
        <div class="feature-arrow">&rarr;</div>
      </div>
      <div class="card feature-card" @click="$router.push('/settings')">
        <div class="feature-icon">S</div>
        <h3>Settings</h3>
        <p>Configure AI models, API keys, and engine options.</p>
        <div class="feature-arrow">&rarr;</div>
      </div>
    </div>

    <div class="card status-card">
      <div class="status-header">
        <h3>Engine Status</h3>
        <button class="btn btn-secondary btn-sm" @click="refreshStatus">Refresh</button>
      </div>
      <div v-if="status" class="status-grid">
        <div class="status-item">
          <div class="status-value">{{ status.character_count }}</div>
          <div class="status-label">Characters</div>
        </div>
        <div class="status-item">
          <div class="status-value">{{ status.dialogue_count }}</div>
          <div class="status-label">Dialogues</div>
        </div>
        <div class="status-item">
          <div class="status-value">{{ status.knowledge_count }}</div>
          <div class="status-label">Knowledge</div>
        </div>
        <div class="status-item">
          <div class="status-value status-ai">{{ status.active_ai_engine || 'Not configured' }}</div>
          <div class="status-label">AI Engine</div>
        </div>
      </div>
      <p v-else class="loading">Loading...</p>
    </div>

    <footer class="footer">
      <p>Monogatari v0.1.0 - Built with Rust + Tauri + Vue 3</p>
    </footer>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface EngineStatus {
  initialized: boolean
  character_count: number
  dialogue_count: number
  knowledge_count: number
  ai_engines: string[]
  active_ai_engine: string | null
}

const status = ref<EngineStatus | null>(null)

async function refreshStatus() {
  try { status.value = await invoke('get_engine_status') } catch (e) { console.error(e) }
}

onMounted(() => { refreshStatus() })
</script>

<style scoped>
.home { padding: 0; max-width: 1200px; margin: 0 auto; }
.hero { position: relative; padding: 60px 40px; text-align: center; overflow: hidden; background: linear-gradient(135deg, rgba(108,92,231,0.15), rgba(0,206,201,0.08)); }
.hero h1 { font-size: 48px; margin-bottom: 12px; background: linear-gradient(135deg, var(--primary), var(--secondary)); -webkit-background-clip: text; -webkit-text-fill-color: transparent; font-weight: 800; }
.subtitle { font-size: 18px; color: var(--text-muted); margin-bottom: 20px; }
.hero-badges { display: flex; gap: 12px; justify-content: center; flex-wrap: wrap; }
.badge { padding: 6px 16px; background: rgba(108,92,231,0.15); border: 1px solid rgba(108,92,231,0.3); border-radius: 20px; font-size: 14px; color: var(--primary); }
.features { display: grid; grid-template-columns: repeat(4, 1fr); gap: 20px; padding: 0 40px; margin: 40px 0; }
.feature-card { text-align: center; padding: 28px 20px; cursor: pointer; transition: all 0.3s; position: relative; overflow: hidden; }
.feature-card:hover { transform: translateY(-4px); border-color: var(--primary); box-shadow: 0 8px 24px rgba(108,92,231,0.2); }
.feature-icon { font-size: 36px; font-weight: bold; color: var(--primary); margin-bottom: 12px; width: 56px; height: 56px; border-radius: 50%; background: rgba(108,92,231,0.1); display: inline-flex; align-items: center; justify-content: center; }
.feature-card h3 { margin-bottom: 8px; color: var(--primary); font-size: 16px; }
.feature-card p { color: var(--text-muted); font-size: 13px; }
.feature-arrow { position: absolute; bottom: 12px; right: 12px; font-size: 18px; color: var(--primary); opacity: 0; transition: all 0.3s; }
.feature-card:hover .feature-arrow { opacity: 1; }
.status-card { margin: 0 40px 40px; padding: 24px; }
.status-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }
.status-header h3 { color: var(--secondary); font-size: 18px; }
.btn-sm { padding: 6px 12px; font-size: 12px; }
.status-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 16px; }
.status-item { text-align: center; padding: 16px; background: var(--bg-input); border-radius: 12px; }
.status-value { font-size: 28px; font-weight: bold; color: var(--primary); margin-bottom: 6px; }
.status-ai { font-size: 14px; color: var(--secondary); }
.status-label { font-size: 13px; color: var(--text-muted); }
.loading { text-align: center; color: var(--text-muted); padding: 20px; }
.footer { text-align: center; padding: 20px; color: var(--text-muted); font-size: 13px; }
@media (max-width: 768px) { .features { grid-template-columns: 1fr 1fr; padding: 0 20px; } .status-grid { grid-template-columns: repeat(2, 1fr); } .hero h1 { font-size: 32px; } }
</style>
