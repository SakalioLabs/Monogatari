<template>
  <Transition name="fade">
    <div v-if="show" class="wn-overlay" @click.self="dismiss">
      <div class="wn-panel">
        <div class="wn-header">
          <span class="wn-version">v{{ currentVersion }}</span>
          <h2>What's New</h2>
          <button class="close-btn" @click="dismiss">Close</button>
        </div>
        <div class="wn-body">
          <div v-for="(release, i) in releases" :key="i" class="wn-release">
            <div class="wn-release-header">
              <strong class="wn-tag">{{ release.version }}</strong>
              <span class="wn-date">{{ release.date }}</span>
            </div>
            <ul class="wn-items">
              <li v-for="item in release.items" :key="item">{{ item }}</li>
            </ul>
          </div>
        </div>
        <div class="wn-footer">
          <button class="btn btn-primary" @click="dismiss">Got it</button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'

const show = ref(false)
const currentVersion = '0.9.2'

const releases = [
  {
    version: '0.9.2', date: '2026-07-07',
    items: [
      'Nori the postmaster with 12-node branching dialogue and town history',
      'Achievement unlock toast notifications when milestones are reached',
      'GameView pause menu (Escape key) with Resume, Save, Load, Backlog, Title',
      'Quick save (S) and quick load (L) during gameplay',
      'Keyboard shortcuts help modal (press ?)',
      'Sora the astronomer with 12-node dialogue and observatory lore',
      'Full i18n in all 20 views with 280+ translation keys in 4 locales',
    ]
  },
  {
    version: '0.9.1', date: '2026-07-07',
    items: [
      'Achievements system with 15 unlockable milestones',
      'Hana tea shop owner with 13-node dialogue and lore',
      'Kai wandering musician with 12-node dialogue and songs',
    ]
  },
  {
    version: '0.9.0', date: '2026-07-07',
    items: [
      'Title Screen, CG Gallery, Backlog viewer',
      'Mio festival organizer with 15-node dialogue',
      'Auto-save during gameplay and SVG scene backgrounds',
      'SVG favicon, OpenGraph meta tags, and SEO polish',
    ]
  }
]

function checkVersion() {
  const seen = localStorage.getItem('monogatari-version-seen')
  if (seen !== currentVersion) {
    show.value = true
  }
}

function dismiss() {
  show.value = false
  localStorage.setItem('monogatari-version-seen', currentVersion)
}

onMounted(checkVersion)
</script>

<style scoped>
.wn-overlay {
  position: fixed; inset: 0; z-index: 110;
  display: flex; align-items: center; justify-content: center;
  background: rgba(0,0,0,0.7); backdrop-filter: blur(8px);
}
.wn-panel {
  width: min(640px, calc(100vw - 32px));
  max-height: calc(100vh - 80px); overflow-y: auto;
  border: 1px solid var(--border); border-radius: var(--radius-lg);
  background: var(--surface-1);
}
.wn-header { display: flex; align-items: center; gap: 14px; padding: 20px 24px; border-bottom: 1px solid var(--border); }
.wn-header h2 { flex: 1; color: var(--text-primary); font-size: 20px; margin: 0; }
.wn-version { padding: 2px 10px; border: 1px solid var(--brand); border-radius: 100px; background: rgba(45,212,191,0.1); color: var(--brand-light); font-size: 12px; font-weight: 800; }
.close-btn { padding: 6px 14px; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--surface-2); color: var(--text-secondary); cursor: pointer; font: inherit; font-weight: 700; font-size: 13px; }
.close-btn:hover { border-color: var(--brand); color: var(--brand-light); }
.wn-body { padding: 16px 24px; display: grid; gap: 20px; }
.wn-release { display: grid; gap: 8px; }
.wn-release-header { display: flex; gap: 12px; align-items: baseline; }
.wn-tag { color: var(--brand-light); font-size: 14px; }
.wn-date { color: var(--text-tertiary); font-size: 12px; }
.wn-items { margin: 0; padding-left: 20px; display: grid; gap: 6px; }
.wn-items li { color: var(--text-secondary); font-size: 13px; line-height: 1.5; }
.wn-footer { padding: 16px 24px; border-top: 1px solid var(--border); display: flex; justify-content: flex-end; }
.btn { min-height: 36px; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--surface-2); color: var(--text-secondary); cursor: pointer; font: inherit; font-weight: 700; padding: 8px 20px; transition: all 0.15s; }
.btn:hover { border-color: var(--brand); color: var(--brand-light); }
.btn-primary { background: var(--brand); color: var(--surface-0); border-color: var(--brand); }
.btn-primary:hover { background: var(--brand-light); }
.fade-enter-active, .fade-leave-active { transition: opacity 0.2s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>
