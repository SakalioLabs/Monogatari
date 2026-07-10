<template>
  <div class="home-page">
    <header class="page-lead">
      <div>
        <span class="eyebrow">{{ t('home.workspace-eyebrow', 'Project workspace') }}</span>
        <h1>{{ t('home.workspace-title', 'Build your story') }}</h1>
        <p>{{ t('home.workspace-copy', 'Move from character foundations to a playable, quality-checked route without losing your place.') }}</p>
      </div>
      <div class="lead-actions">
        <button class="btn btn-secondary" @click="refreshStatus">
          <RefreshCw :size="15" :class="{ spinning: loading }" aria-hidden="true" />
          {{ t('common.refresh', 'Refresh') }}
        </button>
        <button class="btn btn-primary" @click="open('/character-editor')">
          <UserRoundPlus :size="15" aria-hidden="true" />
          {{ t('home.new-character', 'New character') }}
        </button>
      </div>
    </header>

    <section class="runtime-banner" :class="{ ready: status?.initialized }">
      <span class="runtime-icon">
        <CircleCheck v-if="status?.initialized" :size="19" aria-hidden="true" />
        <CircleDashed v-else :size="19" aria-hidden="true" />
      </span>
      <div class="runtime-copy">
        <strong>{{ runtimeTitle }}</strong>
        <span>{{ runtimeDescription }}</span>
      </div>
      <div class="runtime-actions">
        <button class="text-action" @click="open('/settings')">{{ t('home.configure-project', 'Configure project') }}</button>
        <button class="text-action primary" @click="open('/game')">
          <Play :size="13" aria-hidden="true" />
          {{ t('home.preview-story', 'Preview story') }}
        </button>
      </div>
    </section>

    <section class="metric-strip" :aria-label="t('home.project-metrics', 'Project metrics')">
      <div v-for="metric in metrics" :key="metric.label" class="metric-cell">
        <component :is="metric.icon" :size="16" aria-hidden="true" />
        <span>
          <small>{{ metric.label }}</small>
          <strong>{{ loading ? '...' : metric.value }}</strong>
        </span>
      </div>
    </section>

    <div class="dashboard-columns">
      <section class="workflow-column">
        <section class="section-block">
          <div class="section-heading">
            <div>
              <span class="eyebrow">{{ t('home.creation-path', 'Creation path') }}</span>
              <h2>{{ t('home.creation-title', 'From idea to playable route') }}</h2>
            </div>
            <span class="section-note">{{ t('home.creation-note', 'Four focused stages') }}</span>
          </div>

          <div class="stage-list">
            <article v-for="(stage, index) in creationStages" :key="stage.id" class="stage-row">
              <span class="stage-index">{{ index + 1 }}</span>
              <div class="stage-copy">
                <strong>{{ stage.title }}</strong>
                <p>{{ stage.description }}</p>
              </div>
              <div class="stage-tools">
                <button v-for="tool in stage.tools" :key="tool.path" @click="open(tool.path)">
                  <component :is="tool.icon" :size="15" aria-hidden="true" />
                  <span>{{ tool.label }}</span>
                  <ChevronRight :size="14" aria-hidden="true" />
                </button>
              </div>
            </article>
          </div>
        </section>

        <section class="section-block recent-section">
          <div class="section-heading">
            <div>
              <span class="eyebrow">{{ t('home.recent-eyebrow', 'Continue') }}</span>
              <h2>{{ t('home.recent-title', 'Recent work') }}</h2>
            </div>
          </div>

          <div v-if="recentItems.length" class="recent-list">
            <button v-for="item in recentItems.slice(0, 5)" :key="`${item.type}-${item.name}`" @click="open(item.path)">
              <History :size="15" aria-hidden="true" />
              <span><strong>{{ item.name }}</strong><small>{{ item.type }}</small></span>
              <ChevronRight :size="14" aria-hidden="true" />
            </button>
          </div>
          <div v-else class="empty-recent">
            <History :size="20" aria-hidden="true" />
            <span>
              <strong>{{ t('home.no-recent-title', 'No recent work yet') }}</strong>
              <p>{{ t('home.no-recent-copy', 'Open an editor and your latest items will appear here.') }}</p>
            </span>
            <button class="btn btn-secondary btn-sm" @click="open('/character-editor')">{{ t('home.open-editor', 'Open editor') }}</button>
          </div>
        </section>
      </section>

      <aside class="status-column">
        <section class="status-section">
          <div class="section-heading compact">
            <div>
              <span class="eyebrow">{{ t('home.readiness-eyebrow', 'Readiness') }}</span>
              <h2>{{ t('home.readiness-title', 'Project checklist') }}</h2>
            </div>
            <span class="readiness-count">{{ completedReadiness }}/{{ readinessItems.length }}</span>
          </div>
          <div class="readiness-list">
            <button v-for="item in readinessItems" :key="item.label" @click="open(item.path)">
              <CircleCheck v-if="item.done" :size="16" class="done-icon" aria-hidden="true" />
              <Circle v-else :size="16" aria-hidden="true" />
              <span><strong>{{ item.label }}</strong><small>{{ item.detail }}</small></span>
              <ChevronRight :size="14" aria-hidden="true" />
            </button>
          </div>
        </section>

        <section class="status-section">
          <div class="section-heading compact">
            <div>
              <span class="eyebrow">{{ t('home.quick-tools-eyebrow', 'Shortcuts') }}</span>
              <h2>{{ t('home.quick-tools-title', 'Test and diagnose') }}</h2>
            </div>
          </div>
          <div class="quick-tool-list">
            <button @click="open('/chat')"><BotMessageSquare :size="16" aria-hidden="true" /><span>{{ t('nav.chat', 'Character Test') }}</span><ChevronRight :size="14" aria-hidden="true" /></button>
            <button @click="open('/quality')"><ShieldCheck :size="16" aria-hidden="true" /><span>{{ t('nav.quality', 'Quality') }}</span><ChevronRight :size="14" aria-hidden="true" /></button>
            <button @click="open('/analytics')"><ChartNoAxesCombined :size="16" aria-hidden="true" /><span>{{ t('nav.analytics', 'Analytics') }}</span><ChevronRight :size="14" aria-hidden="true" /></button>
          </div>
        </section>

        <section class="build-meta">
          <span>{{ t('home.engine-version', 'Engine version') }}</span>
          <code>0.9.5</code>
          <span>{{ t('home.active-backend', 'Active backend') }}</span>
          <strong>{{ status?.active_ai_engine || t('common.not-configured', 'Not configured') }}</strong>
        </section>
      </aside>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, type Component } from 'vue'
import { useRouter } from 'vue-router'
import {
  BotMessageSquare,
  ChartNoAxesCombined,
  ChevronRight,
  Circle,
  CircleCheck,
  CircleDashed,
  Clapperboard,
  History,
  Library,
  MessageSquareText,
  Play,
  RefreshCw,
  ShieldCheck,
  Sparkles,
  UserRoundPlus,
  Users,
  Workflow,
  Zap,
} from '@lucide/vue'
import { invokeCommand } from '../lib/tauri'
import { useI18n } from '../lib/i18n'

interface EngineStatus {
  initialized: boolean
  character_count: number
  dialogue_count: number
  knowledge_count: number
  ai_engines: string[]
  active_ai_engine: string | null
}

interface RecentItem {
  icon?: string
  name: string
  type: string
  path: string
}

interface StageTool {
  label: string
  path: string
  icon: Component
}

const router = useRouter()
const { t } = useI18n()
const status = ref<EngineStatus | null>(null)
const sceneCount = ref(0)
const recentItems = ref<RecentItem[]>([])
const loading = ref(true)

const runtimeTitle = computed(() => status.value?.initialized
  ? t('home.runtime-ready', 'Runtime ready')
  : t('home.runtime-setup', 'Project setup required'))
const runtimeDescription = computed(() => status.value?.initialized
  ? t('home.runtime-ready-copy', 'Content managers are loaded and the project is ready for authoring and preview.')
  : t('home.runtime-setup-copy', 'Initialize the project and connect an AI backend before testing live story paths.'))

const metrics = computed(() => [
  { label: t('home.stats.characters', 'Characters'), value: status.value?.character_count ?? 0, icon: Users },
  { label: t('home.stats.dialogues', 'Dialogues'), value: status.value?.dialogue_count ?? 0, icon: MessageSquareText },
  { label: t('home.stats.knowledge', 'Knowledge'), value: status.value?.knowledge_count ?? 0, icon: Library },
  { label: t('home.stats.scenes', 'Scenes'), value: sceneCount.value, icon: Clapperboard },
  { label: t('home.stats.backends', 'AI backends'), value: status.value?.ai_engines?.length ?? 0, icon: Sparkles },
])

const creationStages = computed(() => [
  {
    id: 'foundation',
    title: t('home.stage.foundation-title', 'Define the cast and world'),
    description: t('home.stage.foundation-copy', 'Create character intent, personality, and the knowledge that keeps generation consistent.'),
    tools: [
      { label: t('nav.editor', 'Character Editor'), path: '/character-editor', icon: UserRoundPlus },
      { label: t('nav.knowledge', 'Knowledge'), path: '/knowledge', icon: Library },
    ] satisfies StageTool[],
  },
  {
    id: 'narrative',
    title: t('home.stage.narrative-title', 'Author scenes and dialogue'),
    description: t('home.stage.narrative-copy', 'Build the visible stage and the dialogue graphs players will move through.'),
    tools: [
      { label: t('nav.scenes', 'Scenes'), path: '/scene-editor', icon: Clapperboard },
      { label: t('nav.dialogues', 'Dialogues'), path: '/dialogue-editor', icon: MessageSquareText },
    ] satisfies StageTool[],
  },
  {
    id: 'logic',
    title: t('home.stage.logic-title', 'Connect story logic'),
    description: t('home.stage.logic-copy', 'Wire progression, score gates, unlocks, and reusable workflow behavior.'),
    tools: [
      { label: t('nav.workflow', 'Workflow'), path: '/editor', icon: Workflow },
      { label: t('nav.events', 'Story Events'), path: '/story-events', icon: Zap },
    ] satisfies StageTool[],
  },
  {
    id: 'test',
    title: t('home.stage.test-title', 'Preview and verify'),
    description: t('home.stage.test-copy', 'Play the route, inspect AI behavior, and run quality gates before packaging.'),
    tools: [
      { label: t('home.preview-story', 'Preview story'), path: '/game', icon: Play },
      { label: t('nav.quality', 'Quality'), path: '/quality', icon: ShieldCheck },
    ] satisfies StageTool[],
  },
])

const readinessItems = computed(() => [
  {
    label: t('home.readiness.runtime', 'Project initialized'),
    detail: status.value?.initialized ? t('common.ready', 'Ready') : t('home.readiness.runtime-next', 'Open project settings'),
    done: Boolean(status.value?.initialized),
    path: '/settings',
  },
  {
    label: t('home.readiness.cast', 'Cast available'),
    detail: t('home.readiness.count', '{count} authored', { count: status.value?.character_count ?? 0 }),
    done: (status.value?.character_count ?? 0) > 0,
    path: '/character-editor',
  },
  {
    label: t('home.readiness.dialogue', 'Dialogue available'),
    detail: t('home.readiness.count', '{count} authored', { count: status.value?.dialogue_count ?? 0 }),
    done: (status.value?.dialogue_count ?? 0) > 0,
    path: '/dialogue-editor',
  },
  {
    label: t('home.readiness.ai', 'AI backend connected'),
    detail: status.value?.active_ai_engine || t('home.readiness.ai-next', 'Connect in Settings'),
    done: Boolean(status.value?.active_ai_engine),
    path: '/settings',
  },
])
const completedReadiness = computed(() => readinessItems.value.filter((item) => item.done).length)

function open(path: string) {
  void router.push(path)
}

async function refreshStatus() {
  loading.value = true
  try {
    status.value = await invokeCommand<EngineStatus>('get_engine_status', undefined, {
      initialized: false,
      character_count: 0,
      dialogue_count: 0,
      knowledge_count: 0,
      ai_engines: [],
      active_ai_engine: null,
    })
    const scenes = await invokeCommand<any[]>('list_scene_assets', undefined, []).catch(() => [])
    sceneCount.value = scenes.length
  } catch {
    status.value = {
      initialized: false,
      character_count: 0,
      dialogue_count: 0,
      knowledge_count: 0,
      ai_engines: [],
      active_ai_engine: null,
    }
    sceneCount.value = 0
  } finally {
    loading.value = false
  }
}

function loadRecent() {
  try {
    const stored = localStorage.getItem('monogatari-recent')
    recentItems.value = stored ? JSON.parse(stored) : []
  } catch {
    recentItems.value = []
  }
}

onMounted(() => {
  void refreshStatus()
  loadRecent()
})
</script>

<style scoped>
.home-page { width: min(1320px, 100%); margin: 0 auto; padding: 28px 32px 48px; }
.page-lead { display: flex; align-items: flex-start; justify-content: space-between; gap: 24px; margin-bottom: 20px; }
.eyebrow { display: block; color: var(--text-tertiary); font-size: 9px; font-weight: 750; text-transform: uppercase; }
.page-lead h1 { margin-top: 5px; font-size: 27px; line-height: 1.15; }
.page-lead p { max-width: 660px; margin-top: 7px; color: var(--text-secondary); font-size: 12px; }
.lead-actions { display: flex; flex: 0 0 auto; gap: 7px; }
.lead-actions .btn { min-height: 34px; }
.spinning { animation: spin 900ms linear infinite; }

.runtime-banner { display: grid; grid-template-columns: 36px minmax(0, 1fr) auto; align-items: center; gap: 11px; min-height: 62px; padding: 10px 13px; border: 1px solid var(--border); border-left: 3px solid var(--warning); border-radius: var(--radius); background: var(--surface-1); }
.runtime-banner.ready { border-left-color: var(--success); }
.runtime-icon { display: grid; width: 34px; height: 34px; place-items: center; border-radius: var(--radius-sm); background: var(--surface-2); color: var(--warning); }
.runtime-banner.ready .runtime-icon { color: var(--success); }
.runtime-copy { display: grid; min-width: 0; }
.runtime-copy strong { font-size: 12px; }
.runtime-copy span { margin-top: 3px; color: var(--text-tertiary); font-size: 10px; }
.runtime-actions { display: flex; align-items: center; gap: 5px; }
.text-action { display: inline-flex; min-height: 30px; align-items: center; gap: 5px; padding: 5px 8px; border: 0; border-radius: var(--radius-sm); background: transparent; color: var(--text-secondary); cursor: pointer; font: inherit; font-size: 10px; font-weight: 650; }
.text-action:hover { background: var(--surface-2); color: var(--text-primary); }
.text-action.primary { color: var(--brand-strong); }

.metric-strip { display: grid; grid-template-columns: repeat(5, minmax(0, 1fr)); margin: 14px 0 22px; border-block: 1px solid var(--border); }
.metric-cell { display: grid; min-width: 0; min-height: 67px; grid-template-columns: 24px minmax(0, 1fr); align-items: center; gap: 7px; padding: 10px 12px; }
.metric-cell + .metric-cell { border-left: 1px solid var(--border); }
.metric-cell > svg { color: var(--text-tertiary); }
.metric-cell span { display: grid; min-width: 0; }
.metric-cell small { overflow: hidden; color: var(--text-tertiary); font-size: 9px; text-overflow: ellipsis; text-transform: uppercase; white-space: nowrap; }
.metric-cell strong { margin-top: 2px; overflow: hidden; font-size: 17px; text-overflow: ellipsis; white-space: nowrap; }

.dashboard-columns { display: grid; grid-template-columns: minmax(0, 1.75fr) minmax(270px, 0.75fr); gap: 26px; align-items: start; }
.workflow-column, .status-column { display: grid; gap: 24px; }
.section-block, .status-section { min-width: 0; }
.section-heading { display: flex; min-height: 42px; align-items: flex-start; justify-content: space-between; gap: 12px; padding-bottom: 10px; border-bottom: 1px solid var(--border); }
.section-heading.compact { min-height: 38px; }
.section-heading h2 { margin-top: 3px; font-size: 14px; }
.section-note, .readiness-count { color: var(--text-tertiary); font-family: var(--font-mono); font-size: 9px; }

.stage-list { display: grid; }
.stage-row { display: grid; min-width: 0; grid-template-columns: 28px minmax(180px, 1fr) minmax(260px, 0.95fr); align-items: center; gap: 12px; padding: 14px 4px; border-bottom: 1px solid var(--border-subtle); }
.stage-index { display: grid; width: 24px; height: 24px; place-items: center; border: 1px solid var(--border-strong); border-radius: 50%; color: var(--text-tertiary); font: 9px var(--font-mono); }
.stage-copy { min-width: 0; }
.stage-copy strong { font-size: 12px; }
.stage-copy p { margin-top: 3px; color: var(--text-tertiary); font-size: 10px; line-height: 1.45; }
.stage-tools { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 6px; }
.stage-tools button, .recent-list button, .readiness-list button, .quick-tool-list button { display: grid; width: 100%; min-width: 0; align-items: center; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--surface-1); color: var(--text-secondary); cursor: pointer; text-align: left; }
.stage-tools button { min-height: 36px; grid-template-columns: 20px minmax(0, 1fr) 14px; gap: 6px; padding: 7px 8px; font: inherit; font-size: 10px; }
.stage-tools button:hover, .recent-list button:hover, .readiness-list button:hover, .quick-tool-list button:hover { border-color: var(--border-strong); background: var(--surface-2); color: var(--text-primary); }
.stage-tools button span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

.recent-list { display: grid; margin-top: 8px; }
.recent-list button { grid-template-columns: 22px minmax(0, 1fr) 14px; gap: 7px; padding: 9px 8px; border-color: transparent; background: transparent; font: inherit; }
.recent-list span { display: grid; min-width: 0; }
.recent-list strong, .recent-list small { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.recent-list strong { font-size: 11px; }
.recent-list small { margin-top: 2px; color: var(--text-tertiary); font-size: 9px; }
.empty-recent { display: grid; min-height: 100px; grid-template-columns: 28px minmax(0, 1fr) auto; align-items: center; gap: 10px; padding: 16px 6px; color: var(--text-tertiary); }
.empty-recent span { min-width: 0; }
.empty-recent strong { color: var(--text-secondary); font-size: 11px; }
.empty-recent p { margin-top: 3px; font-size: 10px; }

.status-column { padding-left: 4px; }
.readiness-list, .quick-tool-list { display: grid; margin-top: 8px; }
.readiness-list button { grid-template-columns: 22px minmax(0, 1fr) 14px; gap: 7px; padding: 9px 7px; border-color: transparent; background: transparent; font: inherit; }
.readiness-list button > svg:first-child { color: var(--text-tertiary); }
.readiness-list .done-icon { color: var(--success) !important; }
.readiness-list span { display: grid; min-width: 0; }
.readiness-list strong { font-size: 10px; }
.readiness-list small { margin-top: 2px; overflow: hidden; color: var(--text-tertiary); font-size: 9px; text-overflow: ellipsis; white-space: nowrap; }
.quick-tool-list button { grid-template-columns: 22px minmax(0, 1fr) 14px; gap: 7px; padding: 9px 7px; border-color: transparent; background: transparent; font: inherit; font-size: 10px; }
.build-meta { display: grid; grid-template-columns: minmax(0, 1fr) auto; gap: 8px 12px; padding: 12px 10px; border-top: 1px solid var(--border); color: var(--text-tertiary); font-size: 9px; }
.build-meta code, .build-meta strong { max-width: 150px; overflow: hidden; color: var(--text-secondary); font-size: 9px; text-overflow: ellipsis; white-space: nowrap; }

@media (max-width: 1080px) {
  .dashboard-columns { grid-template-columns: 1fr; }
  .status-column { grid-template-columns: repeat(2, minmax(0, 1fr)); padding-left: 0; }
  .build-meta { grid-column: 1 / -1; }
}

@media (max-width: 760px) {
  .home-page { padding: 20px 16px 36px; }
  .page-lead { display: grid; }
  .lead-actions { width: 100%; }
  .lead-actions .btn { flex: 1; justify-content: center; }
  .runtime-banner { grid-template-columns: 36px minmax(0, 1fr); }
  .runtime-actions { grid-column: 1 / -1; justify-content: flex-end; border-top: 1px solid var(--border-subtle); padding-top: 7px; }
  .metric-strip { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  .metric-cell + .metric-cell { border-left: 0; }
  .metric-cell:nth-child(even) { border-left: 1px solid var(--border); }
  .metric-cell:nth-child(n + 3) { border-top: 1px solid var(--border); }
  .stage-row { grid-template-columns: 28px minmax(0, 1fr); }
  .stage-tools { grid-column: 2; }
  .status-column { grid-template-columns: 1fr; }
  .build-meta { grid-column: auto; }
}

@media (max-width: 430px) {
  .page-lead h1 { font-size: 23px; }
  .stage-tools { grid-template-columns: 1fr; }
  .empty-recent { grid-template-columns: 26px minmax(0, 1fr); }
  .empty-recent .btn { grid-column: 2; justify-self: start; }
}
</style>
