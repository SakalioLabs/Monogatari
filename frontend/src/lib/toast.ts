import { ref } from "vue"
export interface ToastItem { id: number; message: string; type: string }
export const toasts = ref<ToastItem[]>([])
let nextId = 0
export function showToast(message: string, type = "info", duration = 3000) {
  const id = nextId++
  toasts.value.push({ id, message, type })
  setTimeout(() => { toasts.value = toasts.value.filter(t => t.id !== id) }, duration)
}