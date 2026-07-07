<template>
  <Transition name="fade">
    <div v-if="visible" class="shortcuts-overlay" @click.self="$emit('close')">
      <div class="shortcuts-panel">
        <div class="panel-header">
          <span class="eyebrow">Shortcuts</span>
          <h2>Keyboard Shortcuts</h2>
          <button class="close-btn" @click="$emit('close')">Close</button>
        </div>
        <div class="shortcuts-grid">
          <div v-for="group in shortcutGroups" :key="group.name" class="shortcut-group">
            <strong class="group-name">{{ group.name }}</strong>
            <div v-for="item in group.items" :key="item.keys" class="shortcut-row">
              <span class="shortcut-keys">
                <span v-for="(key, ki) in splitKeys(item.keys)" :key="ki" class="kbd">{{ key }}</span>
              </span>
              <span class="shortcut-desc">{{ item.desc }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  visible: boolean
}>()
const emit = defineEmits<{
  (e: 'close'): void
}>()

function splitKeys(keys: string): string[] {
  return keys.split(' + ')
}

const shortcutGroups = [
  {
    name: 'Navigation',
    items: [
      { keys: '?', desc: 'Show keyboard shortcuts' },
      { keys: 'Esc', desc: 'Close modals and panels' },
      { keys: 'G + C', desc: 'Go to AI Chat' },
      { keys: 'G + S', desc: 'Go to Story Mode' },
      { keys: 'G + E', desc: 'Go to Workflow Editor' },
      { keys: 'G + R', desc: 'Go to Characters' },
      { keys: 'G + T', desc: 'Go to Settings' },
    ]
  },
  {
    name: 'Story Mode',
    items: [
      { keys: 'Space', desc: 'Advance dialogue' },
      { keys: 'Enter', desc: 'Select choice' },
      { keys: 'S', desc: 'Quick save' },
      { keys: 'L', desc: 'Quick load' },
    ]
  },
  {
    name: 'General',
    items: [
      { keys: 'Ctrl + S', desc: 'Save settings' },
    ]
  }
]
</script>

<style scoped>
.shortcuts-overlay {
  position: fixed; inset: 0; z-index: 100;
  display: flex; align-items: center; justify-content: center;
  background: rgba(0,0,0,0.7); backdrop-filter: blur(8px);
}
.shortcuts-panel {
  width: min(540px, calc(100vw - 32px));
  max-height: calc(100vh - 80px); overflow-y: auto;
  border: 1px solid var(--border); border-radius: var(--radius-lg);
  background: var(--surface-1); padding: 24px;
}
.panel-header { display: flex; align-items: center; gap: 14px; margin-bottom: 20px; }
.panel-header h2 { flex: 1; color: var(--text-primary); font-size: 20px; margin: 0; }
.eyebrow { display: block; color: var(--text-tertiary); font-size: 11px; font-weight: 800; text-transform: uppercase; }
.close-btn {
  padding: 6px 14px; border: 1px solid var(--border); border-radius: var(--radius-sm);
  background: var(--surface-2); color: var(--text-secondary); cursor: pointer;
  font: inherit; font-weight: 700; font-size: 13px;
}
.close-btn:hover { border-color: var(--brand); color: var(--brand-light); }
.shortcuts-grid { display: grid; gap: 20px; }
.shortcut-group { display: grid; gap: 8px; }
.group-name { color: var(--brand-light); font-size: 13px; padding-bottom: 4px; border-bottom: 1px solid var(--border); }
.shortcut-row { display: flex; gap: 16px; align-items: center; }
.shortcut-keys { display: flex; gap: 4px; min-width: 120px; }
.kbd {
  display: inline-flex; align-items: center; justify-content: center;
  min-width: 28px; height: 26px; padding: 0 6px;
  border: 1px solid var(--border); border-radius: 4px; background: var(--surface-2);
  color: var(--text-primary); font-family: var(--font-mono); font-size: 12px; font-weight: 700;
}
.shortcut-desc { color: var(--text-secondary); font-size: 13px; }
.fade-enter-active, .fade-leave-active { transition: opacity 0.2s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>
