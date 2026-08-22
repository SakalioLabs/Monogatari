<template>
  <Transition name="activity-overlay">
    <section v-if="activity" class="project-activity-overlay" role="status" aria-live="polite" aria-atomic="true">
      <div class="project-activity-panel">
        <div class="activity-icon"><LoaderCircle :size="22" class="spinner" aria-hidden="true" /></div>
        <div class="activity-copy">
          <span>{{ operationLabel }}</span>
          <strong>{{ phaseLabel }}</strong>
          <small>{{ t('projects.activity-elapsed', '{seconds}s elapsed', { seconds: elapsedSeconds }) }}</small>
        </div>
        <div class="activity-progress" aria-hidden="true"><span /></div>
      </div>
    </section>
  </Transition>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { LoaderCircle } from '@lucide/vue'
import { activeProjectActivity } from '../lib/projectActivity'
import { useI18n } from '../lib/i18n'

const { t } = useI18n()
const activity = activeProjectActivity
const elapsedSeconds = ref(0)
let startedAt = 0
let timer: ReturnType<typeof setInterval> | undefined

const operationLabel = computed(() => {
  switch (activity.value?.operation) {
    case 'create': return t('projects.activity-create', 'Creating project')
    case 'import': return t('projects.activity-import', 'Importing project')
    case 'initialize': return t('projects.activity-initialize', 'Preparing project')
    default: return t('projects.activity-open', 'Opening project')
  }
})

const phaseLabel = computed(() => {
  switch (activity.value?.phase) {
    case 'loading_content': return t('projects.activity-loading-content', 'Loading story content and references')
    case 'initializing_ai': return t('projects.activity-initializing-ai', 'Preparing configured AI services')
    case 'authorizing_assets': return t('projects.activity-authorizing-assets', 'Authorizing project assets')
    case 'inspecting_package': return t('projects.activity-inspecting-package', 'Checking the project package')
    case 'extracting_package': return t('projects.activity-extracting-package', 'Unpacking project files')
    case 'validating_import': return t('projects.activity-validating-import', 'Validating imported content')
    case 'finalizing_import': return t('projects.activity-finalizing-import', 'Finishing project setup')
    case 'ready': return t('projects.activity-ready', 'Opening the workspace')
    default: return t('projects.activity-checking-project', 'Checking project files')
  }
})

watch(activity, (next, previous) => {
  if (!next) {
    if (timer) clearInterval(timer)
    timer = undefined
    elapsedSeconds.value = 0
    return
  }
  if (previous?.operation === next.operation) return

  if (timer) clearInterval(timer)
  elapsedSeconds.value = 0
  startedAt = Date.now()
  timer = setInterval(() => {
    elapsedSeconds.value = Math.max(1, Math.floor((Date.now() - startedAt) / 1000))
  }, 1000)
}, { immediate: true })

onBeforeUnmount(() => {
  if (timer) clearInterval(timer)
})
</script>

<style scoped>
.project-activity-overlay {
  position: fixed;
  z-index: 1200;
  inset: 0;
  display: grid;
  place-items: center;
  padding: 20px;
  background: color-mix(in srgb, var(--surface-0) 76%, transparent);
  backdrop-filter: blur(3px);
}

.project-activity-panel {
  display: grid;
  width: min(420px, 100%);
  grid-template-columns: auto minmax(0, 1fr);
  gap: 12px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-sm);
  padding: 16px;
  background: var(--surface-1);
  box-shadow: var(--shadow-lg);
}

.activity-icon {
  display: grid;
  width: 38px;
  height: 38px;
  place-items: center;
  border: 1px solid color-mix(in srgb, var(--brand) 42%, var(--border));
  border-radius: var(--radius-sm);
  background: color-mix(in srgb, var(--brand) 10%, var(--surface-2));
  color: var(--brand-light);
}

.activity-copy { display: grid; min-width: 0; align-content: center; gap: 3px; }
.activity-copy > span { color: var(--text-tertiary); font-size: 9px; font-weight: 750; text-transform: uppercase; }
.activity-copy strong { overflow-wrap: anywhere; color: var(--text-primary); font-size: 13px; line-height: 1.3; }
.activity-copy small { color: var(--text-secondary); font-variant-numeric: tabular-nums; font-size: 10px; }

.activity-progress {
  grid-column: 1 / -1;
  height: 3px;
  overflow: hidden;
  background: var(--surface-3);
}

.activity-progress span {
  display: block;
  width: 42%;
  height: 100%;
  background: var(--brand);
  animation: activity-progress 1.3s ease-in-out infinite;
}

.spinner { animation: spin 900ms linear infinite; }
.activity-overlay-enter-active, .activity-overlay-leave-active { transition: opacity 140ms ease; }
.activity-overlay-enter-from, .activity-overlay-leave-to { opacity: 0; }

@keyframes spin { to { transform: rotate(360deg); } }
@keyframes activity-progress {
  0% { transform: translateX(-125%); }
  60%, 100% { transform: translateX(240%); }
}

@media (max-width: 480px) {
  .project-activity-overlay { padding: 14px; }
  .project-activity-panel { padding: 14px; }
}
</style>
