<template>
  <div id="app" class="app-frame" :class="{ 'sidebar-collapsed': sidebarCollapsed, 'fullscreen-route': !showShell }">
    <Transition name="fade">
      <button
        v-if="showShell && mobileNavigationOpen"
        class="mobile-nav-backdrop"
        :aria-label="t('app.close-navigation', 'Close navigation')"
        @click="mobileNavigationOpen = false"
      />
    </Transition>

    <aside v-if="showShell" class="app-sidebar" :class="{ 'mobile-open': mobileNavigationOpen }">
      <div class="sidebar-header">
        <router-link to="/" class="brand-lockup" :aria-label="t('app.title', 'Monogatari Engine')">
          <span class="logo-mark">M</span>
          <span v-if="!sidebarCollapsed" class="logo-copy">
            <strong>Monogatari</strong>
            <small>{{ t('app.workspace', 'Creator workspace') }}</small>
          </span>
        </router-link>
        <button
          class="icon-button mobile-close"
          :title="t('app.close-navigation', 'Close navigation')"
          @click="mobileNavigationOpen = false"
        >
          <X :size="18" aria-hidden="true" />
        </button>
      </div>

      <div v-if="!sidebarCollapsed" class="workspace-status">
        <span class="status-dot" />
        <span>
          <small>{{ t('app.workspace-label', 'Workspace') }}</small>
          <strong>{{ t('app.local-project', 'Local project') }}</strong>
        </span>
      </div>

      <nav class="sidebar-nav" :aria-label="t('app.primary-navigation', 'Primary navigation')">
        <section v-for="group in navGroups" :key="group.id" class="nav-group">
          <button
            v-if="group.collapsible && !sidebarCollapsed"
            class="nav-group-toggle"
            :aria-expanded="isGroupExpanded(group.id)"
            @click="toggleGroup(group.id)"
          >
            <span>{{ group.label }}</span>
            <ChevronDown :size="14" :class="{ rotated: isGroupExpanded(group.id) }" aria-hidden="true" />
          </button>
          <span v-else-if="!sidebarCollapsed" class="nav-group-label">{{ group.label }}</span>
          <div v-show="sidebarCollapsed || !group.collapsible || isGroupExpanded(group.id)" class="nav-group-items">
            <router-link
              v-for="item in group.items"
              :key="item.path"
              :to="item.path"
              class="nav-item"
              :title="sidebarCollapsed ? item.label : undefined"
              @click="mobileNavigationOpen = false"
            >
              <component :is="item.icon" :size="17" :stroke-width="1.8" aria-hidden="true" />
              <span v-if="!sidebarCollapsed" class="nav-label">{{ item.label }}</span>
              <span v-if="item.badge && !sidebarCollapsed" class="nav-badge">{{ item.badge }}</span>
            </router-link>
          </div>
        </section>
      </nav>

      <div class="sidebar-footer">
        <button
          class="sidebar-collapse"
          :title="sidebarCollapsed ? t('app.expand-sidebar', 'Expand sidebar') : t('app.collapse-sidebar', 'Collapse sidebar')"
          @click="sidebarCollapsed = !sidebarCollapsed"
        >
          <PanelLeftOpen v-if="sidebarCollapsed" :size="18" aria-hidden="true" />
          <PanelLeftClose v-else :size="18" aria-hidden="true" />
          <span v-if="!sidebarCollapsed">{{ t('app.collapse-sidebar', 'Collapse sidebar') }}</span>
        </button>
        <span v-if="!sidebarCollapsed" class="version-label">v0.9.5</span>
      </div>
    </aside>

    <div class="app-workspace">
      <header v-if="showShell" class="app-topbar">
        <div class="topbar-context">
          <button
            class="icon-button mobile-menu"
            :title="t('app.open-navigation', 'Open navigation')"
            @click="mobileNavigationOpen = true"
          >
            <Menu :size="19" aria-hidden="true" />
          </button>
          <div class="route-context">
            <span>{{ currentGroupLabel }}</span>
            <strong>{{ currentPageLabel }}</strong>
          </div>
        </div>

        <div class="topbar-actions">
          <GlobalSearch />
          <label class="locale-control" :title="t('app.language', 'Language')">
            <Languages :size="16" aria-hidden="true" />
            <select :value="locale" :aria-label="t('app.language', 'Language')" @change="changeLocale">
              <option v-for="item in supportedLocales" :key="item.code" :value="item.code">
                {{ item.label }}
              </option>
            </select>
          </label>
          <button
            class="icon-button"
            :title="currentTheme === 'dark' ? t('app.use-light-theme', 'Use light theme') : t('app.use-dark-theme', 'Use dark theme')"
            @click="toggleTheme"
          >
            <Sun v-if="currentTheme === 'dark'" :size="18" aria-hidden="true" />
            <Moon v-else :size="18" aria-hidden="true" />
          </button>
        </div>
      </header>

      <main class="app-main">
        <ErrorBoundary>
          <router-view v-slot="{ Component }">
            <transition name="page" mode="out-in">
              <component :is="Component" />
            </transition>
          </router-view>
        </ErrorBoundary>
      </main>
    </div>

    <nav v-if="showShell" class="mobile-bottom-nav" :aria-label="t('app.mobile-navigation', 'Mobile navigation')">
      <router-link v-for="item in mobilePrimaryItems" :key="item.path" :to="item.path" @click="mobileNavigationOpen = false">
        <component :is="item.icon" :size="19" aria-hidden="true" />
        <span>{{ item.mobileLabel }}</span>
      </router-link>
      <button :class="{ active: mobileNavigationOpen }" @click="mobileNavigationOpen = true">
        <MoreHorizontal :size="19" aria-hidden="true" />
        <span>{{ t('app.more', 'More') }}</span>
      </button>
    </nav>

    <ToastNotification />
    <Transition name="fade">
      <button v-if="achievementToastVisible" class="achievement-toast" @click="achievementToastVisible = false">
        <Trophy :size="18" aria-hidden="true" />
        <span>{{ achievementToast }}</span>
      </button>
    </Transition>
    <KeyboardShortcutsHelp :visible="showShortcuts" @close="showShortcuts = false" />
    <WhatsNew />
    <BackToTop />
    <ProgressBar />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref, watch, type Component } from 'vue'
import { useRoute } from 'vue-router'
import {
  AudioLines,
  Blocks,
  BotMessageSquare,
  ChartNoAxesCombined,
  ChevronDown,
  Clapperboard,
  Flag,
  History,
  Image as ImageIcon,
  Images,
  Languages,
  LayoutDashboard,
  Library,
  Menu,
  MessageSquareText,
  MessagesSquare,
  Moon,
  MoreHorizontal,
  PanelLeftClose,
  PanelLeftOpen,
  Play,
  Settings as SettingsIcon,
  ShieldCheck,
  Store,
  Sun,
  Trophy,
  UserRoundPen,
  Users,
  Workflow as WorkflowIcon,
  X,
  Zap,
} from '@lucide/vue'
import ToastNotification from './components/ToastNotification.vue'
import ErrorBoundary from './components/ErrorBoundary.vue'
import KeyboardShortcutsHelp from './components/KeyboardShortcutsHelp.vue'
import WhatsNew from './components/WhatsNew.vue'
import GlobalSearch from './components/GlobalSearch.vue'
import BackToTop from './components/BackToTop.vue'
import ProgressBar from './components/ProgressBar.vue'
import { useI18n } from './lib/i18n'

type NavGroupId = 'overview' | 'create' | 'test' | 'manage'

interface NavItem {
  path: string
  label: string
  mobileLabel: string
  icon: Component
  group: NavGroupId
  badge?: string
}

const route = useRoute()
const { locale, supportedLocales, t, setLocale } = useI18n()
const sidebarCollapsed = ref(localStorage.getItem('monogatari-sidebar') === 'collapsed')
const mobileNavigationOpen = ref(false)
const showShortcuts = ref(false)
const achievementToast = ref('')
const achievementToastVisible = ref(false)
const currentTheme = ref(localStorage.getItem('monogatari-theme') || 'dark')
const expandedGroups = reactive<Record<NavGroupId, boolean>>({
  overview: true,
  create: true,
  test: false,
  manage: false,
})
let achievementToastTimer: number | null = null

const showShell = computed(() => route.name !== 'game' && route.name !== 'title')

const navItems = computed<NavItem[]>(() => [
  { path: '/', label: t('nav.dashboard', 'Dashboard'), mobileLabel: t('nav.mobile.home', 'Home'), icon: LayoutDashboard, group: 'overview' },
  { path: '/character-editor', label: t('nav.editor', 'Character Editor'), mobileLabel: t('nav.mobile.create', 'Create'), icon: UserRoundPen, group: 'create' },
  { path: '/scene-editor', label: t('nav.scenes', 'Scenes'), mobileLabel: t('nav.scenes', 'Scenes'), icon: Clapperboard, group: 'create' },
  { path: '/dialogue-editor', label: t('nav.dialogues', 'Dialogues'), mobileLabel: t('nav.dialogues', 'Dialogues'), icon: MessageSquareText, group: 'create' },
  { path: '/story-events', label: t('nav.events', 'Story Events'), mobileLabel: t('nav.events', 'Events'), icon: Zap, group: 'create' },
  { path: '/endings', label: t('nav.endings', 'Endings'), mobileLabel: t('nav.endings', 'Endings'), icon: Flag, group: 'create' },
  { path: '/editor', label: t('nav.workflow', 'Workflow'), mobileLabel: t('nav.mobile.flow', 'Flow'), icon: WorkflowIcon, group: 'create' },
  { path: '/knowledge', label: t('nav.knowledge', 'Knowledge'), mobileLabel: t('nav.knowledge', 'Knowledge'), icon: Library, group: 'create' },
  { path: '/audio', label: t('nav.audio', 'Audio'), mobileLabel: t('nav.audio', 'Audio'), icon: AudioLines, group: 'create' },
  { path: '/game', label: t('nav.story', 'Story Mode'), mobileLabel: t('nav.mobile.preview', 'Preview'), icon: Play, group: 'test' },
  { path: '/chat', label: t('nav.chat', 'AI Chat'), mobileLabel: t('nav.mobile.test', 'Test'), icon: BotMessageSquare, group: 'test', badge: t('badge.live', 'Live') },
  { path: '/group-chat', label: t('nav.group', 'Group Chat'), mobileLabel: t('nav.group', 'Group'), icon: MessagesSquare, group: 'test' },
  { path: '/quality', label: t('nav.quality', 'Quality'), mobileLabel: t('nav.quality', 'Quality'), icon: ShieldCheck, group: 'test', badge: t('badge.gate', 'Gate') },
  { path: '/analytics', label: t('nav.analytics', 'Analytics'), mobileLabel: t('nav.analytics', 'Analytics'), icon: ChartNoAxesCombined, group: 'test' },
  { path: '/assets', label: t('nav.assets', 'Scene Assets'), mobileLabel: t('nav.assets', 'Assets'), icon: Images, group: 'manage' },
  { path: '/characters', label: t('nav.characters', 'Characters'), mobileLabel: t('nav.characters', 'Characters'), icon: Users, group: 'manage' },
  { path: '/cg-gallery', label: t('nav.cg-gallery', 'CG Gallery'), mobileLabel: t('nav.cg-gallery', 'Gallery'), icon: ImageIcon, group: 'manage' },
  { path: '/backlog', label: t('nav.backlog', 'Backlog'), mobileLabel: t('nav.backlog', 'Backlog'), icon: History, group: 'manage' },
  { path: '/achievements', label: t('nav.achievements', 'Achievements'), mobileLabel: t('nav.achievements', 'Goals'), icon: Trophy, group: 'manage' },
  { path: '/marketplace', label: t('nav.marketplace', 'Marketplace'), mobileLabel: t('nav.marketplace', 'Market'), icon: Store, group: 'manage' },
  { path: '/plugins', label: t('nav.plugins', 'Plugins'), mobileLabel: t('nav.plugins', 'Plugins'), icon: Blocks, group: 'manage' },
  { path: '/settings', label: t('nav.settings', 'Settings'), mobileLabel: t('nav.mobile.settings', 'Settings'), icon: SettingsIcon, group: 'manage' },
])

const navGroups = computed(() => [
  { id: 'overview' as const, label: t('nav.group.overview', 'Overview'), collapsible: false, items: navItems.value.filter((item) => item.group === 'overview') },
  { id: 'create' as const, label: t('nav.group.create', 'Create'), collapsible: true, items: navItems.value.filter((item) => item.group === 'create') },
  { id: 'test' as const, label: t('nav.group.test', 'Test'), collapsible: true, items: navItems.value.filter((item) => item.group === 'test') },
  { id: 'manage' as const, label: t('nav.group.manage', 'Manage'), collapsible: true, items: navItems.value.filter((item) => item.group === 'manage') },
])

const activeNavItem = computed(() => navItems.value.find((item) => item.path === route.path))
const currentPageLabel = computed(() => activeNavItem.value?.label || t('app.title', 'Monogatari Engine'))
const currentGroupLabel = computed(() => (
  navGroups.value.find((group) => group.id === activeNavItem.value?.group)?.label || t('nav.group.overview', 'Overview')
))
const mobilePrimaryItems = computed(() => ['/', '/character-editor', '/chat', '/settings']
  .map((path) => navItems.value.find((item) => item.path === path))
  .filter((item): item is NavItem => Boolean(item)))

watch(sidebarCollapsed, (value) => {
  localStorage.setItem('monogatari-sidebar', value ? 'collapsed' : 'expanded')
})

watch(() => route.path, () => {
  mobileNavigationOpen.value = false
  const group = activeNavItem.value?.group
  if (group) expandedGroups[group] = true
})

function isGroupExpanded(group: NavGroupId) {
  return expandedGroups[group]
}

function toggleGroup(group: NavGroupId) {
  expandedGroups[group] = !expandedGroups[group]
}

async function changeLocale(event: Event) {
  await setLocale((event.target as HTMLSelectElement).value)
}

function applyTheme() {
  document.documentElement.setAttribute('data-theme', currentTheme.value)
  localStorage.setItem('monogatari-theme', currentTheme.value)
}

function toggleTheme() {
  currentTheme.value = currentTheme.value === 'dark' ? 'light' : 'dark'
  applyTheme()
}

function handleAchievement(event: Event) {
  const detail = (event as CustomEvent<{ id?: string; name?: string }>).detail || {}
  const name = detail.name || detail.id || t('app.achievement', 'Achievement')
  achievementToast.value = t('app.achievement-unlocked', 'Achievement unlocked: {name}', { name })
  achievementToastVisible.value = true
  if (achievementToastTimer) window.clearTimeout(achievementToastTimer)
  achievementToastTimer = window.setTimeout(() => {
    achievementToastVisible.value = false
  }, 4000)
}

function handleGlobalKeydown(event: KeyboardEvent) {
  if (event.key === '?' && !event.ctrlKey && !event.metaKey && !showShortcuts.value) {
    event.preventDefault()
    showShortcuts.value = true
  }
  if (event.key === 'Escape') {
    showShortcuts.value = false
    mobileNavigationOpen.value = false
  }
}

onMounted(() => {
  applyTheme()
  window.addEventListener('keydown', handleGlobalKeydown)
  window.addEventListener('achievement-unlock', handleAchievement)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleGlobalKeydown)
  window.removeEventListener('achievement-unlock', handleAchievement)
  if (achievementToastTimer) window.clearTimeout(achievementToastTimer)
})
</script>

<style scoped>
.app-frame {
  display: grid;
  grid-template-columns: var(--sidebar-width) minmax(0, 1fr);
  min-height: 100vh;
  min-height: 100svh;
  background: var(--surface-0);
  transition: grid-template-columns var(--transition);
}

.app-frame.sidebar-collapsed { grid-template-columns: 72px minmax(0, 1fr); }
.app-frame.fullscreen-route { display: block; }

.app-sidebar {
  position: relative;
  z-index: 40;
  display: flex;
  min-width: 0;
  height: 100vh;
  height: 100svh;
  flex-direction: column;
  overflow: hidden;
  border-right: 1px solid var(--border);
  background: var(--surface-1);
}

.sidebar-header {
  display: flex;
  min-height: 64px;
  align-items: center;
  justify-content: space-between;
  padding: 12px 14px;
  border-bottom: 1px solid var(--border);
}

.brand-lockup {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 10px;
  color: var(--text-primary);
  text-decoration: none;
}

.logo-mark {
  display: grid;
  width: 34px;
  height: 34px;
  flex: 0 0 34px;
  place-items: center;
  border-radius: 7px;
  background: var(--brand);
  color: #071b18;
  font-size: 16px;
  font-weight: 850;
}

.logo-copy { display: grid; min-width: 0; line-height: 1.2; }
.logo-copy strong { font-size: 14px; }
.logo-copy small { margin-top: 3px; color: var(--text-tertiary); font-size: 10px; }

.workspace-status {
  display: flex;
  align-items: center;
  gap: 9px;
  margin: 12px 10px 4px;
  padding: 10px 11px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-0);
}

.workspace-status > span:last-child { display: grid; line-height: 1.2; }
.workspace-status small { color: var(--text-tertiary); font-size: 9px; text-transform: uppercase; }
.workspace-status strong { margin-top: 3px; overflow: hidden; font-size: 11px; text-overflow: ellipsis; white-space: nowrap; }
.status-dot { width: 7px; height: 7px; flex: 0 0 7px; border-radius: 50%; background: var(--success); box-shadow: 0 0 0 3px color-mix(in srgb, var(--success) 15%, transparent); }

.sidebar-nav {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 10px 8px 16px;
}

.nav-group + .nav-group { margin-top: 8px; padding-top: 8px; border-top: 1px solid var(--border-subtle); }
.nav-group-label, .nav-group-toggle {
  min-height: 26px;
  padding: 0 8px;
  color: var(--text-tertiary);
  font-size: 9px;
  font-weight: 750;
  text-transform: uppercase;
}
.nav-group-label { display: flex; align-items: center; }
.nav-group-toggle {
  display: flex;
  width: 100%;
  align-items: center;
  justify-content: space-between;
  border: 0;
  background: transparent;
  cursor: pointer;
}
.nav-group-toggle:hover { color: var(--text-secondary); }
.nav-group-toggle svg { transition: transform var(--transition-fast); }
.nav-group-toggle svg.rotated { transform: rotate(180deg); }
.nav-group-items { display: grid; gap: 2px; }

.nav-item {
  position: relative;
  display: flex;
  min-height: 36px;
  align-items: center;
  gap: 10px;
  padding: 7px 9px;
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  font-size: 12px;
  font-weight: 540;
  text-decoration: none;
  transition: background var(--transition-fast), color var(--transition-fast);
}
.nav-item:hover { background: var(--surface-2); color: var(--text-primary); }
.nav-item.router-link-exact-active { background: var(--selection); color: var(--brand-strong); }
.nav-item.router-link-exact-active::before {
  position: absolute;
  top: 8px;
  bottom: 8px;
  left: -8px;
  width: 2px;
  border-radius: 2px;
  background: var(--brand);
  content: '';
}
.nav-item svg { flex: 0 0 auto; }
.nav-label { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.nav-badge { margin-left: auto; color: var(--brand-strong); font-size: 9px; font-weight: 750; text-transform: uppercase; }

.sidebar-footer {
  display: flex;
  min-height: 52px;
  align-items: center;
  justify-content: space-between;
  padding: 8px;
  border-top: 1px solid var(--border);
}
.sidebar-collapse {
  display: flex;
  min-width: 36px;
  min-height: 34px;
  align-items: center;
  gap: 9px;
  padding: 7px 8px;
  border: 0;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  font: inherit;
  font-size: 11px;
}
.sidebar-collapse:hover { background: var(--surface-2); color: var(--text-primary); }
.version-label { padding-right: 5px; color: var(--text-tertiary); font-family: var(--font-mono); font-size: 9px; }

.sidebar-collapsed .sidebar-header { justify-content: center; padding-inline: 8px; }
.sidebar-collapsed .workspace-status, .sidebar-collapsed .nav-group-label, .sidebar-collapsed .nav-group-toggle { display: none; }
.sidebar-collapsed .nav-group + .nav-group { margin-top: 5px; padding-top: 5px; }
.sidebar-collapsed .nav-item { justify-content: center; padding-inline: 8px; }
.sidebar-collapsed .nav-item.router-link-exact-active::before { left: -8px; }
.sidebar-collapsed .sidebar-footer { justify-content: center; }

.app-workspace { display: grid; min-width: 0; min-height: 100svh; grid-template-rows: auto minmax(0, 1fr); }
.fullscreen-route .app-workspace { display: block; min-height: 100svh; }
.app-topbar {
  position: relative;
  z-index: 30;
  display: flex;
  min-height: 56px;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 8px 18px;
  border-bottom: 1px solid var(--border);
  background: color-mix(in srgb, var(--surface-0) 94%, transparent);
  backdrop-filter: blur(14px);
}
.topbar-context, .topbar-actions { display: flex; align-items: center; gap: 8px; }
.route-context { display: grid; min-width: 0; line-height: 1.2; }
.route-context span { color: var(--text-tertiary); font-size: 9px; font-weight: 700; text-transform: uppercase; }
.route-context strong { margin-top: 2px; overflow: hidden; font-size: 13px; text-overflow: ellipsis; white-space: nowrap; }
.icon-button {
  display: inline-grid;
  width: 34px;
  height: 34px;
  flex: 0 0 34px;
  place-items: center;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-1);
  color: var(--text-secondary);
  cursor: pointer;
}
.icon-button:hover { border-color: var(--border-strong); color: var(--text-primary); }
.locale-control {
  display: flex;
  height: 34px;
  align-items: center;
  gap: 6px;
  padding: 0 8px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-1);
  color: var(--text-secondary);
}
.locale-control select { max-width: 104px; border: 0; outline: 0; background: transparent; color: var(--text-primary); font: inherit; font-size: 11px; }
.locale-control option { background: var(--surface-1); color: var(--text-primary); }
.app-main { min-width: 0; min-height: 0; overflow: auto; }
.mobile-menu, .mobile-close, .mobile-bottom-nav, .mobile-nav-backdrop { display: none; }

.achievement-toast {
  position: fixed;
  top: 70px;
  left: 50%;
  z-index: 200;
  display: flex;
  max-width: min(420px, calc(100vw - 32px));
  align-items: center;
  gap: 9px;
  padding: 11px 14px;
  transform: translateX(-50%);
  border: 1px solid color-mix(in srgb, var(--success) 55%, var(--border));
  border-radius: var(--radius);
  background: var(--surface-2);
  box-shadow: var(--shadow-lg);
  color: var(--text-primary);
  cursor: pointer;
  font: inherit;
  font-size: 12px;
  font-weight: 650;
}
.achievement-toast svg { color: var(--warning); }
.page-enter-active, .page-leave-active { transition: opacity 120ms ease; }
.page-enter-from, .page-leave-to { opacity: 0; }

@media (max-width: 860px) {
  .app-frame, .app-frame.sidebar-collapsed { display: block; }
  .app-sidebar {
    position: fixed;
    inset: 0 auto 0 0;
    width: min(292px, calc(100vw - 48px));
    height: 100svh;
    transform: translateX(-102%);
    box-shadow: var(--shadow-lg);
    transition: transform var(--transition);
  }
  .app-sidebar.mobile-open { transform: translateX(0); }
  .sidebar-collapsed .app-sidebar { width: min(292px, calc(100vw - 48px)); }
  .sidebar-collapsed .logo-copy { display: grid; }
  .sidebar-collapsed .workspace-status { display: flex; }
  .sidebar-collapsed .nav-group-label { display: flex; }
  .sidebar-collapsed .nav-group-toggle { display: flex; }
  .sidebar-collapsed .nav-item { justify-content: flex-start; padding-inline: 9px; }
  .sidebar-collapsed .nav-label, .sidebar-collapsed .nav-badge, .sidebar-collapsed .version-label { display: initial; }
  .sidebar-collapsed .sidebar-footer { justify-content: space-between; }
  .sidebar-collapse { display: none; }
  .mobile-close, .mobile-menu { display: inline-grid; }
  .mobile-nav-backdrop {
    position: fixed;
    inset: 0;
    z-index: 35;
    display: block;
    width: 100%;
    height: 100%;
    border: 0;
    background: rgba(0, 0, 0, 0.55);
  }
  .app-topbar { min-height: 54px; padding: 7px 12px; }
  .locale-control select { width: 38px; color: transparent; }
  .locale-control { width: 34px; padding: 0; justify-content: center; overflow: hidden; }
  .locale-control svg { position: absolute; pointer-events: none; }
  .app-main { padding-bottom: calc(60px + env(safe-area-inset-bottom, 0px)); }
  .mobile-bottom-nav {
    position: fixed;
    right: 0;
    bottom: 0;
    left: 0;
    z-index: 32;
    display: grid;
    height: calc(56px + env(safe-area-inset-bottom, 0px));
    grid-template-columns: repeat(5, minmax(0, 1fr));
    padding: 3px 6px env(safe-area-inset-bottom, 0px);
    border-top: 1px solid var(--border);
    background: color-mix(in srgb, var(--surface-1) 96%, transparent);
    backdrop-filter: blur(16px);
  }
  .mobile-bottom-nav a, .mobile-bottom-nav button {
    display: grid;
    min-width: 0;
    place-items: center;
    align-content: center;
    gap: 2px;
    border: 0;
    border-radius: 5px;
    background: transparent;
    color: var(--text-tertiary);
    text-decoration: none;
    font: inherit;
    font-size: 9px;
  }
  .mobile-bottom-nav a.router-link-exact-active, .mobile-bottom-nav button.active { color: var(--brand-strong); }
}

@media (max-width: 480px) {
  .route-context span { display: none; }
  .route-context strong { font-size: 12px; }
  .topbar-actions { gap: 5px; }
}
</style>
