<template>
  <div class="achievements-page">
    <header class="page-header">
      <div>
        <span class="eyebrow">Gamification</span>
        <h1>Achievements</h1>
        <p>Track player milestones, relationship progress, and story unlocks across all playthroughs.</p>
      </div>
      <div class="header-actions">
        <button class="btn btn-secondary btn-sm" @click="refreshAchievements">Refresh</button>
      </div>
    </header>

    <section class="stats-strip">
      <div class="stat-card">
        <span class="stat-value">{{ unlockedCount }}</span>
        <span class="stat-label">Unlocked</span>
      </div>
      <div class="stat-card">
        <span class="stat-value">{{ totalCount }}</span>
        <span class="stat-label">Total</span>
      </div>
      <div class="stat-card">
        <span class="stat-value">{{ completionPercent }}%</span>
        <span class="stat-label">Complete</span>
      </div>
      <div class="stat-card">
        <span class="stat-value">{{ playtimeStr }}</span>
        <span class="stat-label">Playtime</span>
      </div>
    </section>

    <section class="filter-row">
      <button v-for="cat in categories" :key="cat" class="filter-btn" :class="{ active: activeCategory === cat }" @click="activeCategory = cat">
        {{ cat }}
      </button>
    </section>

    <main class="achievements-grid">
      <div v-for="ach in filteredAchievements" :key="ach.id" class="ach-card" :class="{ unlocked: ach.unlocked, locked: !ach.unlocked }">
        <div class="ach-icon" :style="{ background: ach.unlocked ? ach.color : 'var(--surface-3)' }">
          {{ ach.icon }}
        </div>
        <div class="ach-info">
          <strong>{{ ach.unlocked ? ach.name : '???' }}</strong>
          <span class="ach-desc">{{ ach.unlocked ? ach.description : 'Keep playing to unlock this achievement.' }}</span>
          <div class="ach-meta" v-if="ach.unlocked">
            <span class="ach-date">{{ ach.unlockedAt }}</span>
            <span class="ach-category">{{ ach.category }}</span>
          </div>
        </div>
        <div v-if="ach.progress !== undefined" class="ach-progress">
          <div class="progress-track"><div class="progress-fill" :style="{ width: ach.progress + '%' }"></div></div>
          <span class="progress-text">{{ ach.progressText || ach.progress + '%' }}</span>
        </div>
      </div>

      <div v-if="filteredAchievements.length === 0" class="empty-state">
        <span class="empty-icon">&#127942;</span>
        <p>No achievements in this category yet.</p>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { invokeCommand } from '../lib/tauri'

interface Achievement {
  id: string
  name: string
  description: string
  icon: string
  color: string
  category: string
  unlocked: boolean
  unlockedAt: string | null
  progress?: number
  progressText?: string
}

const achievements = ref<Achievement[]>([])
const activeCategory = ref('All')

const categories = computed(() => {
  const cats = new Set(achievements.value.map(a => a.category))
  return ['All', ...Array.from(cats)]
})

const filteredAchievements = computed(() => {
  if (activeCategory.value === 'All') return achievements.value
  return achievements.value.filter(a => a.category === activeCategory.value)
})

const unlockedCount = computed(() => achievements.value.filter(a => a.unlocked).length)
const totalCount = computed(() => achievements.value.length)
const completionPercent = computed(() => totalCount.value ? Math.round((unlockedCount.value / totalCount.value) * 100) : 0)
const playtimeStr = ref('0h')

const defaultAchievements: Achievement[] = [
  { id: 'first_chat', name: 'First Words', description: 'Send your first message to an AI character.', icon: '💬', color: 'rgba(45,212,191,0.3)', category: 'Social', unlocked: false, unlockedAt: null, progress: 0, progressText: '0/1' },
  { id: 'chat_10', name: 'Getting Chatty', description: 'Send 10 messages in a single conversation.', icon: '🗣️', color: 'rgba(96,165,250,0.3)', category: 'Social', unlocked: false, unlockedAt: null, progress: 0, progressText: '0/10' },
  { id: 'chat_50', name: 'Conversationalist', description: 'Send 50 messages across all conversations.', icon: '📝', color: 'rgba(96,165,250,0.3)', category: 'Social', unlocked: false, unlockedAt: null, progress: 0, progressText: '0/50' },
  { id: 'high_relationship', name: 'Best Friends', description: 'Reach a relationship score of 0.8 with any character.', icon: '❤️', color: 'rgba(244,63,94,0.3)', category: 'Relationships', unlocked: false, unlockedAt: null },
  { id: 'all_characters', name: 'Meet Everyone', description: 'Chat with every available character at least once.', icon: '👥', color: 'rgba(168,85,247,0.3)', category: 'Social', unlocked: false, unlockedAt: null, progress: 0, progressText: '0/10' },
  { id: 'first_save', name: 'Save Point', description: 'Save your game for the first time.', icon: '💾', color: 'rgba(34,197,94,0.3)', category: 'Gameplay', unlocked: false, unlockedAt: null },
  { id: 'first_workflow', name: 'Story Architect', description: 'Create and save your first workflow.', icon: '🏗️', color: 'rgba(251,146,60,0.3)', category: 'Creation', unlocked: false, unlockedAt: null },
  { id: 'knowledge_10', name: 'Lore Keeper', description: 'Add 10 entries to the knowledge base.', icon: '📚', color: 'rgba(244,114,182,0.3)', category: 'Creation', unlocked: false, unlockedAt: null, progress: 0, progressText: '0/10' },
  { id: 'eval_high', name: 'Star Performer', description: 'Get an overall evaluation score above 80%.', icon: '⭐', color: 'rgba(251,191,36,0.3)', category: 'Gameplay', unlocked: false, unlockedAt: null },
  { id: 'group_chat', name: 'Party Organizer', description: 'Start your first group chat with multiple characters.', icon: '🎉', color: 'rgba(56,189,248,0.3)', category: 'Social', unlocked: false, unlockedAt: null },
  { id: 'story_mode', name: 'Visual Novel Player', description: 'Start the Story Mode runtime for the first time.', icon: '🎮', color: 'rgba(129,140,248,0.3)', category: 'Gameplay', unlocked: false, unlockedAt: null },
  { id: 'export_project', name: 'Publisher', description: 'Export your project for distribution.', icon: '📦', color: 'rgba(148,163,184,0.3)', category: 'Creation', unlocked: false, unlockedAt: null },
  { id: 'title_screen', name: 'First Impressions', description: 'Visit the title screen.', icon: '🎬', color: 'rgba(244,63,94,0.3)', category: 'Gameplay', unlocked: false, unlockedAt: null },
  { id: 'backlog_reader', name: 'Historian', description: 'View the conversation backlog.', icon: '📖', color: 'rgba(34,197,94,0.3)', category: 'Gameplay', unlocked: false, unlockedAt: null },
  { id: 'cg_collector', name: 'Scene Collector', description: 'View the CG Gallery.', icon: '🖼️', color: 'rgba(168,85,247,0.3)', category: 'Gameplay', unlocked: false, unlockedAt: null },
]

function checkAchievements() {
  // Check local storage for unlocked achievements
  const stored = localStorage.getItem('monogatari-achievements')
  if (stored) {
    try {
      const unlocked = JSON.parse(stored)
      for (const ach of achievements.value) {
        if (unlocked[ach.id]) {
          ach.unlocked = true
          ach.unlockedAt = unlocked[ach.id]
        }
      }
    } catch {}
  }
  
  // Check message count for progress
  let totalMessages = 0
  for (let i = 0; i < localStorage.length; i++) {
    const key = localStorage.key(i)
    if (key?.startsWith('monogatari-chat-count-')) {
      totalMessages += parseInt(localStorage.getItem(key) || '0')
    }
  }
  
  const msgAch = achievements.value.find(a => a.id === 'chat_10')
  if (msgAch) {
    msgAch.progress = Math.min(100, Math.round((Math.min(totalMessages, 10) / 10) * 100))
    msgAch.progressText = Math.min(totalMessages, 10) + '/10'
    if (totalMessages >= 10 && !msgAch.unlocked) unlockAchievement('chat_10')
  }
  
  const msg50Ach = achievements.value.find(a => a.id === 'chat_50')
  if (msg50Ach) {
    msg50Ach.progress = Math.min(100, Math.round((Math.min(totalMessages, 50) / 50) * 100))
    msg50Ach.progressText = Math.min(totalMessages, 50) + '/50'
    if (totalMessages >= 50 && !msg50Ach.unlocked) unlockAchievement('chat_50')
  }
  
  const firstChatAch = achievements.value.find(a => a.id === 'first_chat')
  if (firstChatAch && totalMessages > 0 && !firstChatAch.unlocked) {
    firstChatAch.progress = 100
    firstChatAch.progressText = '1/1'
    unlockAchievement('first_chat')
  }
}

function unlockAchievement(id: string) {
  // Dispatch event for toast notification
  if (typeof window !== 'undefined') {
    window.dispatchEvent(new CustomEvent('achievement-unlock', { detail: { id } }))
  }
  const ach = achievements.value.find(a => a.id === id)
  if (!ach || ach.unlocked) return
  ach.unlocked = true
  ach.unlockedAt = new Date().toLocaleDateString()
  
  const stored = localStorage.getItem('monogatari-achievements')
  const unlocked = stored ? JSON.parse(stored) : {}
  unlocked[id] = ach.unlockedAt
  localStorage.setItem('monogatari-achievements', JSON.stringify(unlocked))
}

function refreshAchievements() {
  achievements.value = [...defaultAchievements]
  checkAchievements()
}

// Export for other views to use
(window as any).__monogatari_unlock = unlockAchievement

onMounted(refreshAchievements)
</script>

<style scoped>
.achievements-page { max-width: 1280px; margin: 0 auto; padding: 34px 40px; }
.page-header { display: flex; justify-content: space-between; gap: 18px; align-items: flex-start; margin-bottom: 22px; }
.page-header h1 { color: var(--text-primary); font-size: 28px; line-height: 1.15; margin-top: 3px; }
.page-header p { max-width: 620px; color: var(--text-secondary); font-size: 13px; margin-top: 7px; }
.eyebrow { display: block; color: var(--text-tertiary); font-size: 11px; font-weight: 800; text-transform: uppercase; }
.header-actions { display: flex; gap: 8px; flex-shrink: 0; }
.stats-strip { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 12px; margin-bottom: 18px; }
.stat-card { display: grid; align-content: center; gap: 4px; min-height: 78px; padding: 15px; border: 1px solid var(--border); border-radius: var(--radius); background: var(--surface-1); }
.stat-value { color: var(--brand-light); font-size: 24px; font-weight: 850; }
.stat-label { color: var(--text-tertiary); font-size: 11px; font-weight: 700; text-transform: uppercase; }
.filter-row { display: flex; gap: 4px; margin-bottom: 18px; border-bottom: 1px solid var(--border); padding-bottom: 4px; }
.filter-btn { padding: 8px 16px; border: none; background: none; color: var(--text-secondary); cursor: pointer; font: inherit; font-weight: 600; font-size: 13px; border-bottom: 2px solid transparent; margin-bottom: -4px; transition: all 0.2s; }
.filter-btn:hover { color: var(--text-primary); }
.filter-btn.active { color: var(--brand-light); border-bottom-color: var(--brand); }
.achievements-grid { display: grid; gap: 10px; }
.ach-card { display: grid; grid-template-columns: 48px 1fr auto; gap: 14px; align-items: center; padding: 16px; border: 1px solid var(--border); border-radius: var(--radius); background: var(--surface-1); transition: all 0.2s; }
.ach-card.locked { opacity: 0.55; }
.ach-card.unlocked { border-color: rgba(45,212,191,0.3); }
.ach-card.unlocked:hover { border-color: var(--brand); }
.ach-icon { width: 48px; height: 48px; border-radius: var(--radius); display: flex; align-items: center; justify-content: center; font-size: 22px; }
.ach-info { min-width: 0; display: grid; gap: 3px; }
.ach-info strong { color: var(--text-primary); font-size: 14px; }
.ach-desc { color: var(--text-tertiary); font-size: 12px; line-height: 1.4; }
.ach-meta { display: flex; gap: 10px; margin-top: 4px; }
.ach-date, .ach-category { font-size: 11px; color: var(--text-tertiary); }
.ach-category { padding: 1px 8px; border: 1px solid var(--border); border-radius: 100px; }
.ach-progress { display: grid; gap: 4px; min-width: 120px; }
.progress-track { height: 6px; overflow: hidden; border-radius: 999px; background: var(--surface-3); }
.progress-fill { height: 100%; border-radius: inherit; background: var(--brand); transition: width 0.3s; }
.progress-text { font-size: 11px; color: var(--text-tertiary); text-align: right; }
.empty-state { text-align: center; padding: 60px; color: var(--text-tertiary); }
.empty-icon { font-size: 48px; display: block; margin-bottom: 12px; }
.btn { min-height: 34px; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--surface-2); color: var(--text-secondary); cursor: pointer; font: inherit; font-weight: 700; font-size: 13px; padding: 6px 14px; transition: all 0.15s; }
.btn:hover { border-color: var(--brand); color: var(--brand-light); }
.btn-secondary { background: var(--surface-2); }
.btn-sm { min-height: 30px; padding: 4px 12px; font-size: 12px; }
@media (max-width: 640px) { .achievements-page { padding: 22px; } .stats-strip { grid-template-columns: repeat(2, minmax(0, 1fr)); } .ach-card { grid-template-columns: 40px 1fr; } .ach-progress { grid-column: 1 / -1; } }
</style>
