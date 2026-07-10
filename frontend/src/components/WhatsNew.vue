<template>
  <Transition name="fade">
    <div v-if="show" class="wn-overlay" @click.self="dismiss">
      <section class="wn-panel" role="dialog" aria-modal="true" aria-labelledby="whats-new-title">
        <header class="wn-header">
          <div class="wn-heading">
            <span class="eyebrow"><Sparkles :size="13" />{{ t('whats-new.eyebrow', 'Release notes') }}</span>
            <div><h2 id="whats-new-title">{{ t('whats-new.title', "What's new") }}</h2><code>v{{ currentVersion }}</code></div>
          </div>
          <button class="icon-command" :title="t('common.close', 'Close')" :aria-label="t('common.close', 'Close')" @click="dismiss">
            <X :size="17" />
          </button>
        </header>

        <div class="wn-body">
          <section v-for="release in releases" :key="release.version" class="wn-release">
            <header class="wn-release-header">
              <strong>{{ release.version }}</strong>
              <time :datetime="release.date">{{ release.date }}</time>
            </header>
            <ul>
              <li v-for="item in release.items" :key="item">{{ item }}</li>
            </ul>
          </section>
        </div>

        <footer class="wn-footer">
          <span><Check :size="14" />{{ t('whats-new.up-to-date', 'This workspace is up to date.') }}</span>
          <button class="btn btn-primary btn-sm" @click="dismiss"><Check :size="14" />{{ t('whats-new.acknowledge', 'Got it') }}</button>
        </footer>
      </section>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { Check, Sparkles, X } from '@lucide/vue'
import { useI18n } from '../lib/i18n'

const { t } = useI18n()
const show = ref(false)
const currentVersion = '0.9.5'

const releases = computed(() => [
  {
    version: '0.9.5',
    date: '2026-07-08',
    items: [
      t('whats-new.release-095-1', 'Tauri desktop build restored with locked Rust dependencies'),
      t('whats-new.release-095-2', 'Locale catalogs repaired for English, Chinese, Japanese, and Korean'),
      t('whats-new.release-095-3', 'The i18n loader now supports desktop and Web/PWA locale files'),
      t('whats-new.release-095-4', 'TTS configuration now supports system, Azure, and ElevenLabs providers'),
      t('whats-new.release-095-5', 'Project export now inventories distributable content'),
    ],
  },
  {
    version: '0.9.2',
    date: '2026-07-07',
    items: [
      t('whats-new.release-092-1', 'Nori joins the cast with a branching dialogue and town history'),
      t('whats-new.release-092-3', 'Playtest includes pause, save, load, transcript, and title actions'),
      t('whats-new.release-092-4', 'Quick save and quick load are available during gameplay'),
      t('whats-new.release-092-5', 'Keyboard shortcut help is available from the question mark key'),
      t('whats-new.release-092-6', 'Sora joins the cast with observatory dialogue and lore'),
      t('whats-new.release-092-7', 'Core views support English, Chinese, Japanese, and Korean'),
    ],
  },
  {
    version: '0.9.1',
    date: '2026-07-07',
    items: [
      t('whats-new.release-091-2', 'Hana joins the cast with tea shop dialogue and lore'),
      t('whats-new.release-091-3', 'Kai joins the cast with musician dialogue and songs'),
    ],
  },
  {
    version: '0.9.0',
    date: '2026-07-07',
    items: [
      t('whats-new.release-090-1', 'Title preview, Visual Review, and Transcript are available'),
      t('whats-new.release-090-2', 'Mio joins the cast with a branching festival dialogue'),
      t('whats-new.release-090-3', 'Playtest supports autosave and project scene backgrounds'),
      t('whats-new.release-090-4', 'App metadata, social previews, and the favicon were refreshed'),
    ],
  },
])

function checkVersion() {
  if (localStorage.getItem('monogatari-version-seen') !== currentVersion) show.value = true
}

function dismiss() {
  show.value = false
  localStorage.setItem('monogatari-version-seen', currentVersion)
}

onMounted(checkVersion)
</script>

<style scoped>
.wn-overlay { position: fixed; inset: 0; z-index: 110; display: grid; place-items: center; padding: 16px; background: rgba(4, 6, 9, 0.76); backdrop-filter: blur(5px); }
.wn-panel { display: grid; width: min(650px, 100%); max-height: min(760px, calc(100svh - 32px)); grid-template-rows: auto minmax(0, 1fr) auto; overflow: hidden; border: 1px solid var(--border); border-radius: var(--radius); background: var(--surface-1); box-shadow: var(--shadow-lg); }
.wn-header { display: flex; min-height: 72px; align-items: center; justify-content: space-between; gap: 14px; padding: 14px 16px; border-bottom: 1px solid var(--border); }
.wn-heading { display: grid; min-width: 0; gap: 5px; }
.eyebrow { display: flex; align-items: center; gap: 6px; color: var(--text-tertiary); font-size: 9px; font-weight: 800; text-transform: uppercase; }
.wn-heading > div { display: flex; flex-wrap: wrap; align-items: baseline; gap: 9px; }
.wn-heading h2 { margin: 0; color: var(--text-primary); font-size: 18px; line-height: 1.2; }
.wn-heading code { color: var(--brand-light); font-size: 10px; }
.icon-command { display: inline-grid; width: 34px; height: 34px; flex: 0 0 34px; place-items: center; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--surface-2); color: var(--text-secondary); cursor: pointer; }
.icon-command:hover { border-color: var(--border-strong); color: var(--text-primary); }
.wn-body { display: grid; overflow-y: auto; padding: 4px 16px; }
.wn-release { display: grid; gap: 9px; padding: 15px 2px; border-bottom: 1px solid var(--border); }
.wn-release:last-child { border-bottom: 0; }
.wn-release-header { display: flex; align-items: baseline; justify-content: space-between; gap: 12px; }
.wn-release-header strong { color: var(--brand-light); font-family: var(--font-mono); font-size: 11px; }
.wn-release-header time { color: var(--text-tertiary); font-family: var(--font-mono); font-size: 9px; }
.wn-release ul { display: grid; gap: 6px; margin: 0; padding-left: 18px; }
.wn-release li { padding-left: 2px; color: var(--text-secondary); font-size: 11px; line-height: 1.45; }
.wn-release li::marker { color: var(--border-strong); }
.wn-footer { display: flex; min-height: 58px; align-items: center; justify-content: space-between; gap: 12px; padding: 11px 16px; border-top: 1px solid var(--border); background: var(--surface-2); }
.wn-footer > span { display: flex; min-width: 0; align-items: center; gap: 6px; color: var(--text-tertiary); font-size: 9px; }
.wn-footer > span svg { flex: 0 0 auto; color: var(--success); }
.wn-footer .btn { display: inline-flex; align-items: center; gap: 6px; }
.fade-enter-active, .fade-leave-active { transition: opacity 0.16s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
@media (max-width: 520px) {
  .wn-overlay { align-items: end; padding: 0; }
  .wn-panel { max-height: calc(100svh - 56px); border-right: 0; border-bottom: 0; border-left: 0; border-radius: var(--radius) var(--radius) 0 0; }
  .wn-footer > span { display: none; }
  .wn-footer { justify-content: flex-end; padding-bottom: calc(11px + env(safe-area-inset-bottom)); }
}
</style>
