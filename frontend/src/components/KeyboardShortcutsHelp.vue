<template>
  <Transition name="fade">
    <div v-if="visible" class="shortcuts-overlay" @click.self="$emit('close')">
      <section class="shortcuts-panel" role="dialog" aria-modal="true" aria-labelledby="shortcuts-title">
        <header class="panel-header">
          <div>
            <span class="eyebrow"><Command :size="13" />{{ t('shortcuts.eyebrow', 'Command reference') }}</span>
            <h2 id="shortcuts-title">{{ t('shortcuts.title', 'Keyboard shortcuts') }}</h2>
          </div>
          <button class="icon-command" :title="t('common.close', 'Close')" :aria-label="t('common.close', 'Close')" @click="$emit('close')">
            <X :size="17" />
          </button>
        </header>

        <div class="shortcuts-grid">
          <section v-for="group in shortcutGroups" :key="group.name" class="shortcut-group">
            <strong class="group-name">{{ group.name }}</strong>
            <div v-for="item in group.items" :key="item.keys" class="shortcut-row">
              <span class="shortcut-keys">
                <kbd v-for="key in splitKeys(item.keys)" :key="key">{{ key }}</kbd>
              </span>
              <span class="shortcut-desc">{{ item.desc }}</span>
            </div>
          </section>
        </div>
      </section>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Command, X } from '@lucide/vue'
import { useI18n } from '../lib/i18n'

defineProps<{ visible: boolean }>()
defineEmits<{ (event: 'close'): void }>()

const { t } = useI18n()

const shortcutGroups = computed(() => [
  {
    name: t('shortcuts.group-navigation', 'Navigation'),
    items: [
      { keys: '?', desc: t('shortcuts.show-help', 'Show keyboard shortcuts') },
      { keys: 'Esc', desc: t('shortcuts.close-panels', 'Close modals and panels') },
      { keys: 'G + C', desc: t('shortcuts.go-chat', 'Go to Character Test') },
      { keys: 'G + S', desc: t('shortcuts.go-story', 'Go to Playtest') },
      { keys: 'G + E', desc: t('shortcuts.go-workflow', 'Go to Workflow Editor') },
      { keys: 'G + R', desc: t('shortcuts.go-characters', 'Go to Characters') },
      { keys: 'G + T', desc: t('shortcuts.go-settings', 'Go to Settings') },
    ],
  },
  {
    name: t('shortcuts.group-story', 'Playtest'),
    items: [
      { keys: 'Space', desc: t('shortcuts.advance-dialogue', 'Advance dialogue') },
      { keys: 'Enter', desc: t('shortcuts.select-choice', 'Select a choice') },
      { keys: 'S', desc: t('shortcuts.quick-save', 'Quick save') },
      { keys: 'L', desc: t('shortcuts.quick-load', 'Quick load') },
    ],
  },
  {
    name: t('shortcuts.group-general', 'General'),
    items: [
      { keys: 'Ctrl + S', desc: t('shortcuts.save-settings', 'Save settings') },
    ],
  },
])

function splitKeys(keys: string): string[] {
  return keys.split(' + ')
}
</script>

<style scoped>
.shortcuts-overlay { position: fixed; inset: 0; z-index: 100; display: grid; place-items: center; padding: 16px; background: rgba(4, 6, 9, 0.76); backdrop-filter: blur(5px); }
.shortcuts-panel { width: min(560px, 100%); max-height: min(700px, calc(100svh - 32px)); overflow: hidden auto; border: 1px solid var(--border); border-radius: var(--radius); background: var(--surface-1); box-shadow: var(--shadow-lg); }
.panel-header { position: sticky; top: 0; z-index: 1; display: flex; min-height: 72px; align-items: center; justify-content: space-between; gap: 14px; padding: 14px 16px; border-bottom: 1px solid var(--border); background: var(--surface-1); }
.panel-header > div { display: grid; min-width: 0; gap: 5px; }
.panel-header h2 { margin: 0; color: var(--text-primary); font-size: 18px; line-height: 1.2; }
.eyebrow { display: flex; align-items: center; gap: 6px; color: var(--text-tertiary); font-size: 9px; font-weight: 800; text-transform: uppercase; }
.icon-command { display: inline-grid; width: 34px; height: 34px; flex: 0 0 34px; place-items: center; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--surface-2); color: var(--text-secondary); cursor: pointer; }
.icon-command:hover { border-color: var(--border-strong); color: var(--text-primary); }
.shortcuts-grid { display: grid; padding: 4px 16px 14px; }
.shortcut-group { display: grid; gap: 2px; padding: 13px 0; border-bottom: 1px solid var(--border); }
.shortcut-group:last-child { border-bottom: 0; }
.group-name { margin-bottom: 6px; color: var(--text-secondary); font-size: 10px; text-transform: uppercase; }
.shortcut-row { display: grid; min-height: 37px; grid-template-columns: 142px minmax(0, 1fr); align-items: center; gap: 14px; }
.shortcut-keys { display: flex; min-width: 0; align-items: center; gap: 4px; }
kbd { display: inline-flex; min-width: 28px; height: 25px; align-items: center; justify-content: center; padding: 0 6px; border: 1px solid var(--border-strong); border-radius: 4px; background: var(--surface-2); color: var(--text-primary); box-shadow: inset 0 -1px 0 var(--border); font-family: var(--font-mono); font-size: 10px; font-weight: 750; }
.shortcut-desc { min-width: 0; color: var(--text-secondary); font-size: 11px; line-height: 1.35; }
.fade-enter-active, .fade-leave-active { transition: opacity 0.16s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
@media (max-width: 520px) {
  .shortcuts-overlay { align-items: end; padding: 0; }
  .shortcuts-panel { max-height: calc(100svh - 56px); border-right: 0; border-bottom: 0; border-left: 0; border-radius: var(--radius) var(--radius) 0 0; padding-bottom: env(safe-area-inset-bottom); }
  .shortcut-row { grid-template-columns: 118px minmax(0, 1fr); }
}
</style>
