<template>
  <div class="title-screen">
    <div class="title-bg">
      <div class="title-particles" ref="particlesRef"></div>
    </div>

    <div class="title-content">
      <div class="title-brand">
        <div class="title-logo">M</div>
        <h1 class="title-name">Monogatari</h1>
        <p class="title-tagline">LLM-Powered Visual Novel Engine</p>
        <span class="title-ver">v0.9.0</span>
      </div>

      <nav class="title-menu">
        <button class="title-btn primary" @click="$router.push('/game')">
          <span class="btn-icon">&#9654;</span>
          <span class="btn-label">{{ t('title.start', 'Start Game') }}</span>
        </button>
        <button class="title-btn" @click="$router.push('/chat')">
          <span class="btn-icon">&#9670;</span>
          <span class="btn-label">{{ t('title.continue', 'Continue') }}</span>
        </button>
        <button class="title-btn" @click="$router.push('/editor')">
          <span class="btn-icon">&#8942;</span>
          <span class="btn-label">{{ t('nav.workflow', 'Workflow') }}</span>
        </button>
        <button class="title-btn" @click="$router.push('/characters')">
          <span class="btn-icon">&#9786;</span>
          <span class="btn-label">{{ t('title.gallery', 'Gallery') }}</span>
        </button>
        <button class="title-btn" @click="$router.push('/settings')">
          <span class="btn-icon">&#9881;</span>
          <span class="btn-label">{{ t('title.settings', 'Settings') }}</span>
        </button>
      </nav>

      <footer class="title-footer">
        <span>{{ t('title.credits', 'Credits') }}</span>
        <span class="sep">|</span>
        <span>{{ t('title.version', 'Version') }} 0.9.0</span>
        <span class="sep">|</span>
        <span>MIT License</span>
      </footer>
    </div>

    <div class="title-scene-info" v-if="activeScene">
      <span class="scene-label">{{ activeScene.name }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useI18n } from '../lib/i18n'
import { invokeCommand } from '../lib/tauri'

const { t } = useI18n()

interface SceneInfo {
  id: string
  name: string
  background_path: string | null
}

const activeScene = ref<SceneInfo | null>(null)
const particlesRef = ref<HTMLDivElement>()

async function loadScene() {
  try {
    const active = await invokeCommand<{ scene: SceneInfo | null }>('get_current_scene', undefined, { scene: null })
    if (active.scene) activeScene.value = active.scene
  } catch {}
}

function spawnParticles() {
  if (!particlesRef.value) return
  for (let i = 0; i < 30; i++) {
    const p = document.createElement('span')
    p.className = 'particle'
    p.style.left = Math.random() * 100 + '%'
    p.style.animationDelay = Math.random() * 8 + 's'
    p.style.animationDuration = 6 + Math.random() * 8 + 's'
    p.style.opacity = String(0.15 + Math.random() * 0.35)
    p.style.width = p.style.height = 2 + Math.random() * 4 + 'px'
    particlesRef.value.appendChild(p)
  }
}

onMounted(() => {
  loadScene()
  spawnParticles()
})
</script>

<style scoped>
.title-screen {
  position: relative;
  width: 100%;
  height: 100vh;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--surface-0);
}

.title-bg {
  position: absolute;
  inset: 0;
  background:
    radial-gradient(ellipse at 30% 20%, rgba(45,212,191,0.12), transparent 50%),
    radial-gradient(ellipse at 70% 80%, rgba(96,165,250,0.10), transparent 50%),
    linear-gradient(180deg, rgba(15,17,21,0.9), var(--surface-0));
}

.title-particles {
  position: absolute;
  inset: 0;
  overflow: hidden;
}

.title-particles :deep(.particle) {
  position: absolute;
  bottom: -10px;
  border-radius: 50%;
  background: var(--brand);
  animation: floatUp linear infinite;
}

@keyframes floatUp {
  0% { transform: translateY(0) scale(1); opacity: 0; }
  10% { opacity: 1; }
  90% { opacity: 1; }
  100% { transform: translateY(-100vh) scale(0.3); opacity: 0; }
}

.title-content {
  position: relative;
  z-index: 2;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 48px;
  text-align: center;
}

.title-brand {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.title-logo {
  width: 72px;
  height: 72px;
  border-radius: 16px;
  background: var(--brand);
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 900;
  font-size: 36px;
  color: var(--surface-0);
  box-shadow: 0 8px 32px rgba(45,212,191,0.25);
  animation: logoGlow 3s ease-in-out infinite alternate;
}

@keyframes logoGlow {
  from { box-shadow: 0 8px 32px rgba(45,212,191,0.25); }
  to { box-shadow: 0 8px 48px rgba(45,212,191,0.45); }
}

.title-name {
  font-size: 56px;
  font-weight: 900;
  color: var(--text-primary);
  letter-spacing: -1px;
  line-height: 1;
  margin: 0;
}

.title-tagline {
  font-size: 16px;
  color: var(--text-tertiary);
  font-weight: 500;
  margin: 0;
}

.title-ver {
  font-size: 11px;
  color: var(--brand-light);
  font-weight: 700;
  padding: 2px 10px;
  border: 1px solid rgba(45,212,191,0.3);
  border-radius: 100px;
  background: rgba(45,212,191,0.08);
}

.title-menu {
  display: flex;
  flex-direction: column;
  gap: 8px;
  width: 280px;
}

.title-btn {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 14px 20px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: rgba(21,25,34,0.85);
  color: var(--text-secondary);
  cursor: pointer;
  font: inherit;
  font-size: 15px;
  font-weight: 600;
  transition: all 0.2s ease;
  backdrop-filter: blur(12px);
}

.title-btn:hover {
  border-color: var(--brand);
  color: var(--brand-light);
  background: rgba(45,212,191,0.08);
  transform: translateX(4px);
}

.title-btn.primary {
  border-color: var(--brand);
  background: rgba(45,212,191,0.15);
  color: var(--brand-light);
  font-weight: 700;
}

.title-btn.primary:hover {
  background: rgba(45,212,191,0.25);
}

.btn-icon {
  font-size: 18px;
  width: 24px;
  text-align: center;
  flex-shrink: 0;
}

.title-footer {
  display: flex;
  gap: 12px;
  align-items: center;
  color: var(--text-tertiary);
  font-size: 12px;
}

.title-footer .sep {
  opacity: 0.4;
}

.title-scene-info {
  position: absolute;
  bottom: 24px;
  right: 24px;
  z-index: 2;
}

.scene-label {
  font-size: 11px;
  color: var(--text-tertiary);
  padding: 4px 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: rgba(21,25,34,0.7);
}

@media (max-width: 480px) {
  .title-name { font-size: 36px; }
  .title-menu { width: 240px; }
  .title-btn { padding: 12px 16px; font-size: 14px; }
}
</style>
