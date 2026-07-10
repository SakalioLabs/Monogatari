<template>
  <div class="achievements-page">
    <header class="page-header">
      <div>
        <span class="eyebrow">{{ t('achievements.eyebrow', 'Player progress') }}</span>
        <h1>{{ t('achievements.title', 'Achievements') }}</h1>
        <p>{{ t('achievements.subtitle', 'Review player milestones, relationship progress, and story unlocks.') }}</p>
      </div>
      <button class="btn btn-secondary btn-sm" @click="refreshAchievements">
        <RefreshCw :size="14" />
        {{ t('common.refresh', 'Refresh') }}
      </button>
    </header>

    <section class="stats-strip" :aria-label="t('achievements.metrics', 'Achievement metrics')">
      <div class="stat-card">
        <span class="stat-value">{{ unlockedCount }}</span>
        <span class="stat-label">{{ t('achievements.unlocked', 'Unlocked') }}</span>
      </div>
      <div class="stat-card">
        <span class="stat-value">{{ totalCount }}</span>
        <span class="stat-label">{{ t('achievements.total', 'Total') }}</span>
      </div>
      <div class="stat-card">
        <span class="stat-value">{{ completionPercent }}%</span>
        <span class="stat-label">{{ t('achievements.complete', 'Complete') }}</span>
      </div>
      <div class="stat-card">
        <span class="stat-value">{{ inProgressCount }}</span>
        <span class="stat-label">{{ t('achievements.in-progress', 'In progress') }}</span>
      </div>
    </section>

    <nav class="filter-row" :aria-label="t('achievements.category-filter', 'Achievement categories')">
      <button
        v-for="category in categories"
        :key="category"
        class="filter-btn"
        :class="{ active: activeCategory === category }"
        @click="activeCategory = category"
      >
        <span>{{ categoryLabel(category) }}</span>
        <small>{{ categoryCount(category) }}</small>
      </button>
    </nav>

    <main class="achievements-grid">
      <article
        v-for="achievement in filteredAchievements"
        :key="achievement.id"
        class="achievement-row"
        :class="{ unlocked: achievement.unlocked, locked: !achievement.unlocked }"
      >
        <div class="achievement-icon" :style="achievementIconStyle(achievement)">
          <component :is="achievement.icon" :size="20" />
        </div>
        <div class="achievement-info">
          <strong>{{ achievement.unlocked ? t(achievement.nameKey) : '???' }}</strong>
          <span class="achievement-description">
            {{ achievement.unlocked ? t(achievement.descriptionKey) : t('achievements.locked-description', 'Continue playing to reveal this achievement.') }}
          </span>
          <div v-if="achievement.unlocked" class="achievement-meta">
            <time v-if="achievement.unlockedAt">{{ formatUnlockDate(achievement.unlockedAt) }}</time>
            <span>{{ categoryLabel(achievement.category) }}</span>
          </div>
        </div>
        <div v-if="achievement.progress !== undefined" class="achievement-progress">
          <div class="progress-track" aria-hidden="true">
            <div class="progress-fill" :style="{ width: `${achievement.progress}%` }"></div>
          </div>
          <span>{{ achievement.progressText || `${achievement.progress}%` }}</span>
        </div>
      </article>

      <div v-if="filteredAchievements.length === 0" class="empty-state">
        <Trophy :size="30" />
        <p>{{ t('achievements.empty', 'No achievements are available in this category.') }}</p>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import type { Component } from 'vue'
import { computed, onMounted, ref } from 'vue'
import {
  BookOpen,
  Clapperboard,
  HeartHandshake,
  History,
  Images,
  MessageCircle,
  MessagesSquare,
  PackageOpen,
  PanelsTopLeft,
  PartyPopper,
  RefreshCw,
  Save,
  Star,
  Trophy,
  Users,
  Workflow,
} from '@lucide/vue'
import { useI18n } from '../lib/i18n'

type AchievementCategory = 'social' | 'relationships' | 'gameplay' | 'creation'
type CategoryFilter = 'all' | AchievementCategory

interface Achievement {
  id: string
  nameKey: string
  descriptionKey: string
  icon: Component
  color: string
  category: AchievementCategory
  unlocked: boolean
  unlockedAt: string | null
  progress?: number
  progressText?: string
}

const { locale, t } = useI18n()
const achievements = ref<Achievement[]>([])
const activeCategory = ref<CategoryFilter>('all')

const categoryOrder: AchievementCategory[] = ['social', 'relationships', 'gameplay', 'creation']
const categories = computed<CategoryFilter[]>(() => [
  'all',
  ...categoryOrder.filter(category => achievements.value.some(item => item.category === category)),
])
const filteredAchievements = computed(() => activeCategory.value === 'all'
  ? achievements.value
  : achievements.value.filter(item => item.category === activeCategory.value))
const unlockedCount = computed(() => achievements.value.filter(item => item.unlocked).length)
const totalCount = computed(() => achievements.value.length)
const completionPercent = computed(() => totalCount.value
  ? Math.round((unlockedCount.value / totalCount.value) * 100)
  : 0)
const inProgressCount = computed(() => achievements.value.filter(item => (
  !item.unlocked && item.progress !== undefined && item.progress > 0
)).length)

const defaultAchievements: Achievement[] = [
  achievement('first_chat', MessageCircle, '#2dd4bf', 'social', true),
  achievement('chat_10', MessagesSquare, '#60a5fa', 'social', true),
  achievement('chat_50', MessagesSquare, '#818cf8', 'social', true),
  achievement('high_relationship', HeartHandshake, '#fb7185', 'relationships'),
  achievement('all_characters', Users, '#c084fc', 'social', true),
  achievement('first_save', Save, '#4ade80', 'gameplay'),
  achievement('first_workflow', Workflow, '#fb923c', 'creation'),
  achievement('knowledge_10', BookOpen, '#f472b6', 'creation', true),
  achievement('eval_high', Star, '#facc15', 'gameplay'),
  achievement('group_chat', PartyPopper, '#38bdf8', 'social'),
  achievement('story_mode', Clapperboard, '#a78bfa', 'gameplay'),
  achievement('export_project', PackageOpen, '#94a3b8', 'creation'),
  achievement('title_screen', PanelsTopLeft, '#f43f5e', 'gameplay'),
  achievement('backlog_reader', History, '#22c55e', 'gameplay'),
  achievement('cg_collector', Images, '#a855f7', 'gameplay'),
]

function achievement(
  id: string,
  icon: Component,
  color: string,
  category: AchievementCategory,
  tracked = false,
): Achievement {
  return {
    id,
    nameKey: `achievements.item.${id}.name`,
    descriptionKey: `achievements.item.${id}.description`,
    icon,
    color,
    category,
    unlocked: false,
    unlockedAt: null,
    ...(tracked ? { progress: 0, progressText: id === 'first_chat' ? '0/1' : '0/10' } : {}),
  }
}

function categoryLabel(category: CategoryFilter): string {
  return t(`achievements.category.${category}`, category === 'all' ? 'All' : category)
}

function categoryCount(category: CategoryFilter): number {
  return category === 'all'
    ? achievements.value.length
    : achievements.value.filter(item => item.category === category).length
}

function achievementIconStyle(item: Achievement) {
  return { color: item.color, backgroundColor: `${item.color}1f` }
}

function formatUnlockDate(value: string): string {
  const date = new Date(value)
  return Number.isNaN(date.getTime()) ? value : date.toLocaleDateString(locale.value)
}

function checkAchievements() {
  const stored = localStorage.getItem('monogatari-achievements')
  if (stored) {
    try {
      const unlocked = JSON.parse(stored) as Record<string, string>
      for (const item of achievements.value) {
        if (unlocked[item.id]) {
          item.unlocked = true
          item.unlockedAt = unlocked[item.id]
        }
      }
    } catch {
      localStorage.removeItem('monogatari-achievements')
    }
  }

  let totalMessages = 0
  for (let index = 0; index < localStorage.length; index += 1) {
    const key = localStorage.key(index)
    if (key?.startsWith('monogatari-chat-count-')) {
      totalMessages += Number.parseInt(localStorage.getItem(key) || '0', 10) || 0
    }
  }

  updateTrackedAchievement('first_chat', totalMessages, 1)
  updateTrackedAchievement('chat_10', totalMessages, 10)
  updateTrackedAchievement('chat_50', totalMessages, 50)
}

function updateTrackedAchievement(id: string, current: number, target: number) {
  const item = achievements.value.find(achievementItem => achievementItem.id === id)
  if (!item) return
  const boundedCurrent = Math.min(current, target)
  item.progress = Math.round((boundedCurrent / target) * 100)
  item.progressText = `${boundedCurrent}/${target}`
  if (current >= target && !item.unlocked) unlockAchievement(id)
}

function unlockAchievement(id: string) {
  const item = achievements.value.find(achievementItem => achievementItem.id === id)
  if (!item || item.unlocked) return

  item.unlocked = true
  item.unlockedAt = new Date().toISOString()
  const stored = localStorage.getItem('monogatari-achievements')
  let unlocked: Record<string, string> = {}
  try {
    unlocked = stored ? JSON.parse(stored) as Record<string, string> : {}
  } catch {
    unlocked = {}
  }
  unlocked[id] = item.unlockedAt
  localStorage.setItem('monogatari-achievements', JSON.stringify(unlocked))
  window.dispatchEvent(new CustomEvent('achievement-unlock', { detail: { id } }))
}

function refreshAchievements() {
  achievements.value = defaultAchievements.map(item => ({ ...item }))
  checkAchievements()
}

;(window as Window & { __monogatari_unlock?: (id: string) => void }).__monogatari_unlock = unlockAchievement

onMounted(refreshAchievements)
</script>

<style scoped>
.achievements-page {
  max-width: 1180px;
  margin: 0 auto;
  padding: 32px 36px 48px;
}

.page-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 18px;
  margin-bottom: 22px;
}

.page-header h1 {
  margin: 3px 0 0;
  color: var(--text-primary);
  font-size: 28px;
  line-height: 1.15;
}

.page-header p {
  max-width: 640px;
  margin: 7px 0 0;
  color: var(--text-secondary);
  font-size: 13px;
  line-height: 1.55;
}

.eyebrow {
  display: block;
  color: var(--text-tertiary);
  font-size: 11px;
  font-weight: 800;
  text-transform: uppercase;
}

.stats-strip {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 10px;
  margin-bottom: 18px;
}

.stat-card {
  display: grid;
  align-content: center;
  min-height: 76px;
  padding: 14px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-1);
}

.stat-value {
  color: var(--text-primary);
  font-size: 23px;
  font-weight: 850;
}

.stat-label {
  color: var(--text-tertiary);
  font-size: 11px;
  font-weight: 700;
}

.filter-row {
  display: flex;
  gap: 4px;
  overflow-x: auto;
  margin-bottom: 16px;
  border-bottom: 1px solid var(--border);
}

.filter-btn {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  min-height: 38px;
  padding: 7px 12px;
  border: 0;
  border-bottom: 2px solid transparent;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  font: inherit;
  font-size: 12px;
  font-weight: 700;
  white-space: nowrap;
}

.filter-btn small {
  color: var(--text-tertiary);
  font-size: 10px;
}

.filter-btn:hover { color: var(--text-primary); }
.filter-btn.active { border-bottom-color: var(--brand); color: var(--brand-light); }

.achievements-grid {
  display: grid;
  gap: 8px;
}

.achievement-row {
  display: grid;
  grid-template-columns: 44px minmax(0, 1fr) minmax(110px, 170px);
  gap: 13px;
  align-items: center;
  min-height: 78px;
  padding: 13px 14px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-1);
}

.achievement-row.unlocked { border-color: rgba(45, 212, 191, 0.25); }
.achievement-row.locked { opacity: 0.58; }

.achievement-icon {
  display: grid;
  place-items: center;
  width: 42px;
  height: 42px;
  border-radius: var(--radius);
}

.achievement-info {
  display: grid;
  gap: 3px;
  min-width: 0;
}

.achievement-info strong {
  color: var(--text-primary);
  font-size: 13px;
}

.achievement-description {
  color: var(--text-tertiary);
  font-size: 12px;
  line-height: 1.45;
}

.achievement-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  color: var(--text-tertiary);
  font-size: 10px;
}

.achievement-progress {
  display: grid;
  gap: 5px;
}

.achievement-progress > span {
  color: var(--text-tertiary);
  font-family: var(--font-mono);
  font-size: 10px;
  text-align: right;
}

.progress-track {
  height: 5px;
  overflow: hidden;
  border-radius: 999px;
  background: var(--surface-3);
}

.progress-fill {
  height: 100%;
  border-radius: inherit;
  background: var(--brand);
}

.empty-state {
  display: grid;
  place-items: center;
  gap: 8px;
  min-height: 220px;
  color: var(--text-tertiary);
  text-align: center;
}

.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 7px;
  min-height: 34px;
  padding: 6px 13px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
  color: var(--text-secondary);
  cursor: pointer;
  font: inherit;
  font-size: 12px;
  font-weight: 700;
}

.btn:hover { border-color: var(--brand); color: var(--brand-light); }

@media (max-width: 720px) {
  .achievements-page { padding: 22px 16px 96px; }
  .page-header { align-items: flex-start; }
  .page-header p { display: none; }
  .stats-strip { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  .achievement-row { grid-template-columns: 40px minmax(0, 1fr); }
  .achievement-progress { grid-column: 1 / -1; }
}

@media (max-width: 420px) {
  .page-header { flex-direction: column; }
  .page-header .btn { width: 100%; }
}
</style>
