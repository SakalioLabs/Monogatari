<template>
  <Transition name="fade">
    <button v-if="visible" class="back-to-top" @click="scrollToTop" title="Back to top">
      &#8593;
    </button>
  </Transition>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'

const visible = ref(false)
let scrollHandler: (() => void) | null = null

function scrollToTop() {
  window.scrollTo({ top: 0, behavior: 'smooth' })
}

onMounted(() => {
  scrollHandler = () => { visible.value = window.scrollY > 300 }
  window.addEventListener('scroll', scrollHandler, { passive: true })
})

onUnmounted(() => {
  if (scrollHandler) window.removeEventListener('scroll', scrollHandler)
})
</script>

<style scoped>
.back-to-top {
  position: fixed; bottom: 32px; right: 32px; z-index: 50;
  width: 44px; height: 44px;
  border: 1px solid var(--border); border-radius: 50%;
  background: var(--surface-2); color: var(--text-secondary);
  cursor: pointer; font-size: 20px; line-height: 1;
  display: flex; align-items: center; justify-content: center;
  box-shadow: var(--shadow-lg); transition: all 0.15s;
}
.back-to-top:hover { border-color: var(--brand); color: var(--brand-light); background: var(--surface-1); transform: translateY(-2px); }
.fade-enter-active, .fade-leave-active { transition: opacity 0.2s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>
