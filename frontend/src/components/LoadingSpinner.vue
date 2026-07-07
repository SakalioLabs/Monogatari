<template>
  <div class="loading-wrap" :class="{ inline }">
    <div class="loading-spinner" :style="{ width: size + 'px', height: size + 'px', borderWidth: thickness + 'px' }"></div>
    <span v-if="text" class="loading-text">{{ text }}</span>
  </div>
</template>

<script setup lang="ts">
withDefaults(defineProps<{
  text?: string
  size?: number
  thickness?: number
  inline?: boolean
}>(), {
  text: '',
  size: 28,
  thickness: 3,
  inline: false,
})
</script>

<style scoped>
.loading-wrap {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 40px;
}
.loading-wrap.inline {
  display: inline-flex;
  flex-direction: row;
  padding: 0;
  gap: 8px;
}
.loading-spinner {
  border-radius: 50%;
  border-style: solid;
  border-color: var(--surface-3);
  border-top-color: var(--brand);
  animation: spin 1s linear infinite;
}
.loading-text {
  color: var(--text-tertiary);
  font-size: 13px;
  font-weight: 600;
}
@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
