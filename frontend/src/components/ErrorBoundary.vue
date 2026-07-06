<template>
  <div v-if="error" class="error-boundary">
    <div class="error-content">
      <h2>Something went wrong</h2>
      <p class="error-msg">{{ error.message }}</p>
      <button class="btn btn-primary btn-sm" @click="reset">Try Again</button>
    </div>
  </div>
  <slot v-else />
</template>
<script setup lang="ts">
import { ref, onErrorCaptured } from "vue"
const error = ref<Error | null>(null)
onErrorCaptured((err) => { error.value = err; return false })
function reset() { error.value = null }
</script>
<style scoped>
.error-boundary { display: flex; align-items: center; justify-content: center; min-height: 200px; padding: 40px; }
.error-content { text-align: center; display: flex; flex-direction: column; gap: 12px; align-items: center; }
.error-content h2 { color: var(--danger); font-size: 18px; }
.error-msg { color: var(--text-secondary); font-size: 13px; max-width: 400px; }
</style>