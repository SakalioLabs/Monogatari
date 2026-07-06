<template>
  <div class="toast-container">
    <div v-for="toast in toasts" :key="toast.id" class="toast" :class="toast.type">
      <span class="toast-msg">{{ toast.message }}</span>
    </div>
  </div>
</template>
<script setup lang="ts">
import { ref } from 'vue'
export interface ToastItem { id: number; message: string; type: string }
const toasts = ref<ToastItem[]>([])
let nextId = 0
export function showToast(message: string, type = 'info', duration = 3000) {
  const id = nextId++
  toasts.value.push({ id, message, type })
  setTimeout(() => { toasts.value = toasts.value.filter(t => t.id !== id) }, duration)
}
defineExpose({ showToast })
</script>
<style scoped>
.toast-container { position: fixed; top: 20px; right: 20px; z-index: 9999; display: flex; flex-direction: column; gap: 8px; }
.toast { padding: 10px 18px; border-radius: 8px; font-size: 13px; font-weight: 600; animation: slideIn 0.3s ease; }
.toast.info { background: var(--info); color: white; }
.toast.success { background: var(--success); color: white; }
.toast.warning { background: var(--warning); color: var(--surface-0); }
.toast.error { background: var(--danger); color: white; }
@keyframes slideIn { from { opacity: 0; transform: translateX(20px); } to { opacity: 1; transform: translateX(0); } }
</style>