<template>
  <section v-if="error" class="error-boundary" role="alert">
    <CircleAlert :size="24" />
    <div class="error-copy">
      <span class="eyebrow">{{ t('error-boundary.eyebrow', 'View recovery') }}</span>
      <h2>{{ t('error-boundary.title', 'This view could not be rendered') }}</h2>
      <p>{{ error.message }}</p>
    </div>
    <button class="btn btn-primary btn-sm" @click="reset">
      <RotateCcw :size="14" />{{ t('error-boundary.retry', 'Try again') }}
    </button>
  </section>
  <slot v-else />
</template>

<script setup lang="ts">
import { onErrorCaptured, ref } from 'vue'
import { CircleAlert, RotateCcw } from '@lucide/vue'
import { useI18n } from '../lib/i18n'

const { t } = useI18n()
const error = ref<Error | null>(null)

onErrorCaptured((capturedError) => {
  error.value = capturedError
  return false
})

function reset() {
  error.value = null
}
</script>

<style scoped>
.error-boundary { display: grid; width: min(680px, calc(100% - 32px)); min-height: 112px; grid-template-columns: 32px minmax(0, 1fr) auto; align-items: center; gap: 14px; margin: 28px auto; padding: 16px; border: 1px solid color-mix(in srgb, var(--danger) 42%, var(--border)); border-radius: var(--radius); background: var(--surface-1); }
.error-boundary > svg { color: var(--danger); }
.error-copy { display: grid; min-width: 0; gap: 3px; }
.eyebrow { color: var(--text-tertiary); font-size: 9px; font-weight: 800; text-transform: uppercase; }
.error-copy h2 { margin: 0; color: var(--text-primary); font-size: 14px; line-height: 1.3; }
.error-copy p { overflow: hidden; margin: 0; color: var(--text-tertiary); font-family: var(--font-mono); font-size: 9px; line-height: 1.4; text-overflow: ellipsis; white-space: nowrap; }
.error-boundary .btn { display: inline-flex; align-items: center; gap: 6px; }
@media (max-width: 620px) {
  .error-boundary { grid-template-columns: 28px minmax(0, 1fr); }
  .error-boundary .btn { grid-column: 1 / -1; width: 100%; }
}
</style>
