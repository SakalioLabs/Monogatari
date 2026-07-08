<template>
  <div id="app" :class="{ 'sidebar-collapsed': sidebarCollapsed }">
    <aside v-if="showSidebar" class="app-sidebar">
      <div class="sidebar-header">
        <div class="logo-mark">M</div>
        <div class="logo-text" v-show="!sidebarCollapsed">
          <span class="logo-name">Monogatari</span>
          <span class="logo-badge">{{ t('app.version', 'Engine v0.9.5') }}</span>
        </div>
      </div>
      <nav class="sidebar-nav">
        <router-link v-for="item in navItems" :key="item.path" :to="item.path" class="nav-item" :class="{ active: $route.path === item.path }" :title="item.label">
          <span class="nav-icon" v-html="item.icon"></span>
          <span class="nav-label" v-show="!sidebarCollapsed">{{ item.label }}</span>
          <span v-if="item.badge && !sidebarCollapsed" class="nav-badge">{{ item.badge }}</span>
        </router-link>
      </nav>
      <div class="sidebar-search">
        <GlobalSearch />
      </div>
      <div class="sidebar-footer">
        <button class="nav-item" @click="sidebarCollapsed = !sidebarCollapsed" :title="t('app.compact', 'Compact')">
          <span class="nav-icon" v-html="sidebarCollapsed ? '&rsaquo;' : '&lsaquo;'"></span>
          <span class="nav-label" v-show="!sidebarCollapsed">{{ t('app.compact', 'Compact') }}</span>
        </button>
      </div>
    </aside>
    <main class="app-main">
      <ToastNotification />
      <ErrorBoundary>
        <router-view v-slot="{ Component }">
          <transition name="page" mode="out-in">
            <component :is="Component" />
          </transition>
        </router-view>
      </ErrorBoundary>
    </main>
    <Transition name="fade">
      <div v-if="achievementToastVisible" class="achievement-toast" @click="achievementToastVisible = false">
        &#127942; {{ achievementToast }}
      </div>
    </Transition>
    <KeyboardShortcutsHelp :visible="showShortcuts" @close="showShortcuts = false" />
    <WhatsNew />
    <BackToTop />
    <ProgressBar />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import ToastNotification from './components/ToastNotification.vue'
import ErrorBoundary from './components/ErrorBoundary.vue'
import KeyboardShortcutsHelp from './components/KeyboardShortcutsHelp.vue'
import WhatsNew from './components/WhatsNew.vue'
import GlobalSearch from './components/GlobalSearch.vue'
import BackToTop from './components/BackToTop.vue'
import ProgressBar from './components/ProgressBar.vue'
import { useRoute } from 'vue-router'
import { useI18n } from './lib/i18n'

const { t } = useI18n()
const route = useRoute()
const sidebarCollapsed = ref(localStorage.getItem('monogatari-sidebar') === 'collapsed')
watch(sidebarCollapsed, (val) => { localStorage.setItem('monogatari-sidebar', val ? 'collapsed' : 'expanded') })
const showShortcuts = ref(false)
const achievementToast = ref('')
const achievementToastVisible = ref(false)
let achievementToastTimer: number | null = null

// Listen for achievement unlock events
if (typeof window !== 'undefined') {
  window.addEventListener('achievement-unlock', ((e: CustomEvent) => {
    const { id, name } = e.detail
    achievementToast.value = `Achievement unlocked: ${name || id}`
    achievementToastVisible.value = true
    if (achievementToastTimer) clearTimeout(achievementToastTimer)
    achievementToastTimer = window.setTimeout(() => {
      achievementToastVisible.value = false
    }, 4000)
  }) as EventListener)
}
const showSidebar = computed(() => route.name !== 'game' && route.name !== 'title')

// Global keyboard handler
function handleGlobalKeydown(e: KeyboardEvent) {
  if (e.key === '?' && !e.ctrlKey && !e.metaKey && !showShortcuts.value) {
    e.preventDefault()
    showShortcuts.value = true
  }
  if (e.key === 'Escape' && showShortcuts.value) {
    showShortcuts.value = false
  }
}

const navItems = computed(() => [
  { path: '/', label: t('nav.dashboard', 'Dashboard'), icon: '&#9632;' },
  { path: '/chat', label: t('nav.chat', 'AI Chat'), icon: '&#9670;', badge: t('badge.live', 'Live') },
  { path: '/game', label: t('nav.story', 'Story Mode'), icon: '&#9654;' },
  { path: '/editor', label: t('nav.workflow', 'Workflow'), icon: '&#8942;' },
  { path: '/assets', label: t('nav.assets', 'Scene Assets'), icon: '&#9638;' },
  { path: '/character-editor', label: t('nav.editor', 'Editor'), icon: '&#9998;' },
  { path: '/characters', label: t('nav.characters', 'Characters'), icon: '&#9786;' },
  { path: '/group-chat', label: t('nav.group', 'Group Chat'), icon: '&#9733;' },
  { path: '/settings', label: t('nav.settings', 'Settings'), icon: '&#9881;' },
  { path: '/analytics', label: t('nav.analytics', 'Analytics'), icon: '&#9636;' },
  { path: '/quality', label: t('nav.quality', 'Quality'), icon: '&#10003;', badge: t('badge.gate', 'Gate') },
  { path: '/plugins', label: t('nav.plugins', 'Plugins'), icon: '&#128295;' },
  { path: '/scene-editor', label: t('nav.scenes', 'Scenes'), icon: '&#127912;' },
  { path: '/dialogue-editor', label: t('nav.dialogues', 'Dialogues'), icon: '&#128172;' },
  { path: '/knowledge', label: t('nav.knowledge', 'Knowledge'), icon: '&#128218;' },
  { path: '/audio', label: t('nav.audio', 'Audio'), icon: '&#9835;' },
  { path: '/marketplace', label: t('nav.marketplace', 'Marketplace'), icon: '&#9733;', badge: t('badge.new', 'New') },
  { path: '/cg-gallery', label: t('nav.cg-gallery', 'CG Gallery'), icon: '&#127912;' },
  { path: '/backlog', label: t('nav.backlog', 'Backlog'), icon: '&#128214;' },
  { path: '/achievements', label: t('nav.achievements', 'Achievements'), icon: '&#127942;' },
])
</script>

<style scoped>
#app { display: flex; min-height: 100vh; min-height: 100svh; }
.app-sidebar {
  width: var(--sidebar-width); min-width: var(--sidebar-width);
  background: rgba(21,25,34,0.96); border-right: 1px solid var(--border);
  display: flex; flex-direction: column; transition: all var(--transition);
  z-index: 10;
}
.sidebar-collapsed .app-sidebar { width: 60px; min-width: 60px; }
.sidebar-header {
  display: flex; align-items: center; gap: 12px;
  padding: 20px 16px 16px; border-bottom: 1px solid var(--border);
}
.logo-mark {
  width: 36px; height: 36px; border-radius: var(--radius-sm);
  background: var(--brand);
  display: flex; align-items: center; justify-content: center;
  font-weight: 800; font-size: 18px; color: var(--surface-0); flex-shrink: 0;
}
.logo-text { overflow: hidden; white-space: nowrap; }
.logo-name { font-weight: 700; font-size: 16px; color: var(--text-primary); display: block; }
.logo-badge { font-size: 10px; color: var(--text-tertiary); }
.sidebar-nav { flex: 1; padding: 12px 8px; display: flex; flex-direction: column; gap: 2px; overflow-y: auto; }
.nav-item {
  display: flex; align-items: center; gap: 12px;
  padding: 10px 12px; border-radius: var(--radius-sm);
  color: var(--text-secondary); text-decoration: none;
  transition: all var(--transition-fast); cursor: pointer;
  border: none; background: none; font-size: 13px; font-weight: 500;
  font-family: var(--font-sans); width: 100%; position: relative;
}
.nav-item:hover { background: var(--surface-2); color: var(--text-primary); }
.nav-item.active, .router-link-exact-active.nav-item {
  background: var(--surface-2); color: var(--brand-light);
}
.nav-item.active::before {
  content: ''; position: absolute; left: 0; top: 8px; bottom: 8px;
  width: 3px; border-radius: 0 3px 3px 0; background: var(--brand);
}
.nav-icon { font-size: 16px; width: 20px; text-align: center; flex-shrink: 0; }
.nav-label { overflow: hidden; white-space: nowrap; }
.nav-badge {
  margin-left: auto; padding: 1px 6px; border-radius: 100px;
  font-size: 10px; font-weight: 700; background: rgba(45,212,191,0.16); color: var(--brand-light);
}
.sidebar-search { padding: 4px 8px; display: flex; justify-content: center; }
.sidebar-footer { padding: 8px; border-top: 1px solid var(--border); }
.app-main { flex: 1; overflow: auto; min-width: 0; min-height: 0; }
.page-enter-active { animation: fadeIn 0.2s ease; }
.page-leave-active { animation: fadeIn 0.15s ease reverse; }
.achievement-toast {
  position: fixed; top: 18px; left: 50%; transform: translateX(-50%); z-index: 200;
  padding: 14px 24px; border: 1px solid rgba(45,212,191,0.4); border-radius: var(--radius);
  background: rgba(15,118,110,0.96); color: white; font-weight: 700; font-size: 14px;
  box-shadow: var(--shadow-lg); cursor: pointer;
}

@media (max-width: 720px) {
  #app { flex-direction: column; }
  .app-sidebar, .sidebar-collapsed .app-sidebar {
    width: 100%; min-width: 0; flex-direction: row; align-items: stretch;
    border-right: none; border-bottom: 1px solid var(--border);
  }
  .sidebar-header { flex-shrink: 0; padding: 10px 12px; border-right: 1px solid var(--border); border-bottom: none; }
  .logo-mark { width: 32px; height: 32px; }
  .logo-text { display: none; }
  .sidebar-nav { flex: 1; min-width: 0; flex-direction: row; gap: 4px; overflow-x: auto; padding: 8px; }
  .nav-item { width: auto; min-width: max-content; padding: 8px 10px; }
  .nav-icon { width: 16px; }
  .nav-badge { display: none; }
  .nav-item.active::before { left: 8px; right: 8px; top: auto; bottom: 0; width: auto; height: 3px; border-radius: 3px 3px 0 0; }
  .sidebar-footer { display: none; }
  .app-main { width: 100%; min-height: 0; }
}
</style>
