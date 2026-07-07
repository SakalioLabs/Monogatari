<template>
  <Transition name="fade">
    <div v-if="visible" class="confirm-overlay" @click.self="cancel">
      <div class="confirm-panel">
        <div class="confirm-header">
          <h3>{{ title }}</h3>
        </div>
        <p class="confirm-message">{{ message }}</p>
        <div class="confirm-actions">
          <button class="btn btn-secondary" @click="cancel">{{ cancelText }}</button>
          <button class="btn btn-danger" @click="confirm">{{ confirmText }}</button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
const props = withDefaults(defineProps<{
  visible: boolean
  title?: string
  message?: string
  confirmText?: string
  cancelText?: string
}>(), {
  title: 'Confirm',
  message: 'Are you sure?',
  confirmText: 'Delete',
  cancelText: 'Cancel',
})

const emit = defineEmits<{
  (e: 'confirm'): void
  (e: 'cancel'): void
  (e: 'update:visible', val: boolean): void
}>()

function confirm() { emit('confirm'); emit('update:visible', false) }
function cancel() { emit('cancel'); emit('update:visible', false) }
</script>

<style scoped>
.confirm-overlay {
  position: fixed; inset: 0; z-index: 60;
  display: flex; align-items: center; justify-content: center;
  background: rgba(0,0,0,0.6); backdrop-filter: blur(6px);
}
.confirm-panel {
  width: min(420px, calc(100vw - 32px));
  border: 1px solid var(--border); border-radius: var(--radius);
  background: var(--surface-1); padding: 24px;
}
.confirm-header h3 { color: var(--text-primary); font-size: 18px; margin: 0 0 8px; }
.confirm-message { color: var(--text-secondary); font-size: 14px; line-height: 1.5; margin: 0 0 20px; }
.confirm-actions { display: flex; gap: 8px; justify-content: flex-end; }
.btn { min-height: 36px; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--surface-2); color: var(--text-secondary); cursor: pointer; font: inherit; font-weight: 700; padding: 8px 18px; transition: all 0.15s; }
.btn:hover { border-color: var(--brand); color: var(--brand-light); }
.btn-secondary { background: var(--surface-2); }
.btn-danger { background: rgba(239,68,68,0.15); border-color: rgba(239,68,68,0.4); color: var(--danger); }
.btn-danger:hover { background: rgba(239,68,68,0.25); }
.fade-enter-active, .fade-leave-active { transition: opacity 0.15s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>
