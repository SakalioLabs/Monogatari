<template>
  <div id="app" :class="{ 'sidebar-collapsed': sidebarCollapsed }">
    <aside v-if="showSidebar" class="app-sidebar">
      <div class="sidebar-header">
        <div class="logo-mark">M</div>
        <div class="logo-text" v-show="!sidebarCollapsed">
          <span class="logo-name">Monogatari</span>
          <span class="logo-badge">v0.2</span>
        </div>
      </div>
      <nav class="sidebar-nav">
        <router-link v-for="item in navItems" :key="item.path" :to="item.path" class="nav-item" :class="{ active: $route.path === item.path }">
          <span class="nav-icon" v-html="item.icon"></span>
          <span class="nav-label" v-show="!sidebarCollapsed">{{ item.label }}</span>
          <span v-if="item.badge && !sidebarCollapsed" class="nav-badge">{{ item.badge }}</span>
        </router-link>
      </nav>
      <div class="sidebar-footer">
        <button class="nav-item" @click="sidebarCollapsed = !sidebarCollapsed">
          <span class="nav-icon" v-html="sidebarCollapsed ? '&rsaquo;' : '&lsaquo;'"></span>
          <span class="nav-label" v-show="!sidebarCollapsed">Collapse</span>
        </button>
      </div>
    </aside>
    <main class="app-main">
      <router-view v-slot="{ Component }">
        <transition name="page" mode="out-in">
          <component :is="Component" />
        </transition>
      </router-view>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRoute } from 'vue-router'

const route = useRoute()
const sidebarCollapsed = ref(false)
const showSidebar = computed(() => route.name !== 'game')

const navItems = [
  { path: '/', label: 'Dashboard', icon: '&#9632;' },
  { path: '/chat', label: 'AI Chat', icon: '&#9993;', badge: 'New' },
  { path: '/game', label: 'Story Mode', icon: '&#9654;' },
  { path: '/editor', label: 'Workflow', icon: '&#9776;' },
  { path: '/settings', label: 'Settings', icon: '&#9881;' },
]
</script>

<style scoped>
#app { display: flex; min-height: 100vh; }
.app-sidebar {
  width: var(--sidebar-width); min-width: var(--sidebar-width);
  background: var(--surface-1); border-right: 1px solid var(--border);
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
  background: linear-gradient(135deg, var(--brand), var(--accent));
  display: flex; align-items: center; justify-content: center;
  font-weight: 800; font-size: 18px; color: white; flex-shrink: 0;
}
.logo-text { overflow: hidden; white-space: nowrap; }
.logo-name { font-weight: 700; font-size: 16px; color: var(--text-primary); display: block; }
.logo-badge { font-size: 10px; color: var(--text-tertiary); }
.sidebar-nav { flex: 1; padding: 12px 8px; display: flex; flex-direction: column; gap: 2px; }
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
  background: rgba(124,92,252,0.12); color: var(--brand-light);
}
.nav-item.active::before {
  content: ''; position: absolute; left: 0; top: 8px; bottom: 8px;
  width: 3px; border-radius: 0 3px 3px 0; background: var(--brand);
}
.nav-icon { font-size: 16px; width: 20px; text-align: center; flex-shrink: 0; }
.nav-label { overflow: hidden; white-space: nowrap; }
.nav-badge {
  margin-left: auto; padding: 1px 6px; border-radius: 100px;
  font-size: 10px; font-weight: 700; background: var(--brand); color: white;
}
.sidebar-footer { padding: 8px; border-top: 1px solid var(--border); }
.app-main { flex: 1; overflow: auto; min-width: 0; }
.page-enter-active { animation: fadeIn 0.2s ease; }
.page-leave-active { animation: fadeIn 0.15s ease reverse; }
</style>
