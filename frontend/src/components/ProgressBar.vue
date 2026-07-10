<template>
  <div class="progress-bar" :class="{ active: isLoading }">
    <div class="progress-fill"></div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useRouter } from 'vue-router'

const router = useRouter()
const isLoading = ref(false)
let timer: number | null = null

router.beforeEach((to, from, next) => {
  isLoading.value = true
  next()
})

router.afterEach(() => {
  if (timer) clearTimeout(timer)
  timer = window.setTimeout(() => { isLoading.value = false }, 300)
})
</script>

<style scoped>
.progress-bar {
  position: fixed; top: 0; left: 0; z-index: 1000;
  width: 100%; height: 3px;
  background: transparent; pointer-events: none;
}
.progress-fill {
  width: 0; height: 100%;
  background: var(--brand);
  transition: width 0.3s ease;
}
.progress-bar.active .progress-fill {
  width: 70%;
  animation: progressBump 2s ease-in-out infinite;
}
@keyframes progressBump {
  0% { width: 10%; }
  50% { width: 60%; }
  100% { width: 20%; }
}
</style>
