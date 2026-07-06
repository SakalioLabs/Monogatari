<template>
  <div class="home">
    <header class="hero">
      <div class="hero-bg"></div>
      <div class="hero-content">
        <h1>🎮 LLM Galgame Engine</h1>
        <p class="subtitle">AI驱动的视觉小说引擎 · Rust + Tauri + Live2D</p>
        <div class="hero-badges">
          <span class="badge">🤖 LLM对话</span>
          <span class="badge">🎭 Live2D</span>
          <span class="badge">🔧 可视化编辑</span>
          <span class="badge">⚡ 高性能</span>
        </div>
      </div>
    </header>

    <div class="features">
      <div class="card feature-card" @click="$router.push('/game')">
        <div class="feature-icon">🎭</div>
        <h3>开始游戏</h3>
        <p>体验AI驱动的视觉小说游戏</p>
        <div class="feature-arrow">→</div>
      </div>
      <div class="card feature-card" @click="$router.push('/editor')">
        <div class="feature-icon">🔧</div>
        <h3>工作流编辑器</h3>
        <p>可视化创建故事，无需编码</p>
        <div class="feature-arrow">→</div>
      </div>
      <div class="card feature-card" @click="$router.push('/settings')">
        <div class="feature-icon">⚙️</div>
        <h3>设置</h3>
        <p>配置AI模型和引擎选项</p>
        <div class="feature-arrow">→</div>
      </div>
    </div>

    <div class="card status-card">
      <div class="status-header">
        <h3>📊 引擎状态</h3>
        <button class="btn btn-secondary btn-sm" @click="refreshStatus">刷新</button>
      </div>
      <div v-if="status" class="status-grid">
        <div class="status-item">
          <div class="status-value">{{ status.character_count }}</div>
          <div class="status-label">角色</div>
        </div>
        <div class="status-item">
          <div class="status-value">{{ status.dialogue_count }}</div>
          <div class="status-label">对话</div>
        </div>
        <div class="status-item">
          <div class="status-value">{{ status.knowledge_count }}</div>
          <div class="status-label">知识</div>
        </div>
        <div class="status-item">
          <div class="status-value status-ai">{{ status.active_ai_engine || '未配置' }}</div>
          <div class="status-label">AI引擎</div>
        </div>
      </div>
      <p v-else class="loading">
        <span class="loading-spinner"></span>
        加载中...
      </p>
    </div>

    <footer class="footer">
      <p>LLM Galgame Engine v0.1.0 · Built with Rust + Tauri + Vue 3</p>
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
  try {
    status.value = await invoke('get_engine_status')
  } catch (e) {
    console.error('Failed to get engine status:', e)
  }
}

onMounted(() => {
  refreshStatus()
})
</script>

<style scoped>
.home {
  padding: 0;
  max-width: 1200px;
  margin: 0 auto;
}

.hero {
  position: relative;
  padding: 60px 40px;
  text-align: center;
  overflow: hidden;
}

.hero-bg {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(135deg, rgba(108, 92, 231, 0.2), rgba(0, 206, 201, 0.1));
  z-index: 0;
}

.hero-content {
  position: relative;
  z-index: 1;
}

.hero h1 {
  font-size: 48px;
  margin-bottom: 12px;
  background: linear-gradient(135deg, var(--primary), var(--secondary));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  font-weight: 800;
}

.subtitle {
  font-size: 18px;
  color: var(--text-muted);
  margin-bottom: 20px;
}

.hero-badges {
  display: flex;
  gap: 12px;
  justify-content: center;
  flex-wrap: wrap;
}

.badge {
  padding: 6px 16px;
  background: rgba(108, 92, 231, 0.15);
  border: 1px solid rgba(108, 92, 231, 0.3);
  border-radius: 20px;
  font-size: 14px;
  color: var(--primary);
}

.features {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 20px;
  padding: 0 40px;
  margin-bottom: 40px;
}

.feature-card {
  text-align: center;
  padding: 32px 24px;
  cursor: pointer;
  transition: all 0.3s;
  position: relative;
  overflow: hidden;
}

.feature-card:hover {
  transform: translateY(-4px);
  border-color: var(--primary);
  box-shadow: 0 8px 24px rgba(108, 92, 231, 0.2);
}

.feature-icon {
  font-size: 48px;
  margin-bottom: 16px;
}

.feature-card h3 {
  margin-bottom: 8px;
  color: var(--primary);
  font-size: 18px;
}

.feature-card p {
  color: var(--text-muted);
  font-size: 14px;
}

.feature-arrow {
  position: absolute;
  bottom: 16px;
  right: 16px;
  font-size: 20px;
  color: var(--primary);
  opacity: 0;
  transform: translateX(-8px);
  transition: all 0.3s;
}

.feature-card:hover .feature-arrow {
  opacity: 1;
  transform: translateX(0);
}

.status-card {
  margin: 0 40px 40px;
  padding: 24px;
}

.status-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.status-header h3 {
  color: var(--secondary);
  font-size: 18px;
}

.btn-sm {
  padding: 6px 12px;
  font-size: 12px;
}

.status-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
}

.status-item {
  text-align: center;
  padding: 20px;
  background: var(--bg-input);
  border-radius: 12px;
  transition: transform 0.2s;
}

.status-item:hover {
  transform: scale(1.02);
}

.status-value {
  font-size: 32px;
  font-weight: bold;
  color: var(--primary);
  margin-bottom: 8px;
}

.status-ai {
  font-size: 16px;
  color: var(--secondary);
}

.status-label {
  font-size: 13px;
  color: var(--text-muted);
}

.loading {
  text-align: center;
  color: var(--text-muted);
  padding: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.loading-spinner {
  display: inline-block;
  width: 16px;
  height: 16px;
  border: 2px solid var(--border);
  border-top-color: var(--primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.footer {
  text-align: center;
  padding: 20px;
  color: var(--text-muted);
  font-size: 13px;
}

@media (max-width: 768px) {
  .features {
    grid-template-columns: 1fr;
    padding: 0 20px;
  }
  
  .status-grid {
    grid-template-columns: repeat(2, 1fr);
  }
  
  .hero h1 {
    font-size: 32px;
  }
}
</style>
