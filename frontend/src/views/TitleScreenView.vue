<template>
  <div class="title-screen">
    <section class="title-hero">
      <img v-if="sceneImageUrl && !backgroundFailed" class="title-background" :src="sceneImageUrl" :alt="activeScene?.name || t('title.preview-scene', 'Preview scene')" @error="backgroundFailed = true" />
      <div class="title-scrim"></div>

      <header class="title-utility">
        <span class="engine-version">v0.9.5</span>
        <label class="language-control">
          <Languages :size="14" />
          <select :value="locale" :aria-label="t('settings.language', 'Language')" @change="changeLocale">
            <option v-for="option in supportedLocales" :key="option.code" :value="option.code">{{ option.label }}</option>
          </select>
        </label>
      </header>

      <div class="title-content">
        <div class="title-brand">
          <span class="brand-mark">M</span>
          <h1>{{ t('title.brand', 'Monogatari') }}</h1>
          <p>{{ t('title.tagline', 'Stories begin here.') }}</p>
        </div>

        <nav class="title-menu" :aria-label="t('title.menu', 'Main menu')">
          <button class="title-command primary" @click="openRoute('/game')"><Play :size="17" /><span>{{ t('title.start', 'Start Game') }}</span><ArrowRight :size="15" /></button>
          <button class="title-command" @click="openRoute('/chat')"><MessageCircle :size="16" /><span>{{ t('nav.chat', 'AI Chat') }}</span><ArrowRight :size="14" /></button>
          <button class="title-command" @click="openRoute('/editor')"><Workflow :size="16" /><span>{{ t('nav.workflow', 'Workflow') }}</span><ArrowRight :size="14" /></button>
          <button class="title-command" @click="openRoute('/characters')"><Images :size="16" /><span>{{ t('title.gallery', 'Gallery') }}</span><ArrowRight :size="14" /></button>
          <button class="title-command" @click="openRoute('/settings')"><Settings :size="16" /><span>{{ t('title.settings', 'Settings') }}</span><ArrowRight :size="14" /></button>
        </nav>
      </div>
    </section>

    <footer class="title-status">
      <div class="scene-status">
        <span class="status-dot"></span>
        <span>{{ t('title.active-scene', 'Active scene') }}</span>
        <strong>{{ activeScene?.name || t('title.preview-scene', 'Preview scene') }}</strong>
      </div>
      <div class="title-meta">
        <span>{{ t('title.credits', 'Credits') }}</span>
        <span>{{ t('title.version', 'Version') }} 0.9.5</span>
        <span>{{ t('title.license', 'MIT License') }}</span>
      </div>
    </footer>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import { ArrowRight, Images, Languages, MessageCircle, Play, Settings, Workflow } from '@lucide/vue'
import { useRouter } from 'vue-router'
import { resolveAssetUrl } from '../lib/assets'
import { useI18n } from '../lib/i18n'
import { invokeCommand } from '../lib/tauri'

interface SceneInfo {
  id: string
  name: string
  background_path: string | null
  absolute_background_path?: string | null
}

const previewScene: SceneInfo = {
  id: 'studio_night',
  name: 'Studio Night',
  background_path: 'assets/backgrounds/studio_night.svg',
  absolute_background_path: null,
}

const router = useRouter()
const { locale, setLocale, supportedLocales, t } = useI18n()
const activeScene = ref<SceneInfo | null>(null)
const backgroundFailed = ref(false)
const sceneImageUrl = computed(() => resolveAssetUrl(activeScene.value?.absolute_background_path || activeScene.value?.background_path || previewScene.background_path))

async function loadScene() {
  try {
    const active = await invokeCommand<{ scene: SceneInfo | null }>('get_current_scene', undefined, { scene: previewScene })
    activeScene.value = active.scene || previewScene
  } catch {
    activeScene.value = previewScene
  }
}

async function changeLocale(event: Event) {
  await setLocale((event.target as HTMLSelectElement).value)
}

function openRoute(path: string) {
  void router.push(path)
}

onMounted(loadScene)
</script>

<style scoped>
.title-screen {
  width: 100%;
  min-height: 100svh;
  overflow: hidden;
  background: #0b1015;
  color: white;
}

.title-hero {
  position: relative;
  display: grid;
  min-height: calc(100svh - 70px);
  align-items: center;
  overflow: hidden;
}

.title-background,
.title-scrim {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
}

.title-background { object-fit: cover; }
.title-scrim { background: rgba(5, 9, 13, 0.62); }

.title-utility {
  position: absolute;
  z-index: 2;
  top: 18px;
  right: 20px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.engine-version,
.language-control {
  min-height: 32px;
  border: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 6px;
  background: rgba(7, 12, 17, 0.72);
  color: rgba(255, 255, 255, 0.76);
}

.engine-version {
  display: inline-flex;
  align-items: center;
  padding: 0 9px;
  font-family: var(--font-mono);
  font-size: 9px;
  font-weight: 800;
}

.language-control {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 7px;
}

.language-control select {
  max-width: 120px;
  border: 0;
  outline: 0;
  background: transparent;
  color: inherit;
  font: inherit;
  font-size: 9px;
  font-weight: 700;
}

.language-control option { background: #111820; color: white; }

.title-content {
  position: relative;
  z-index: 1;
  display: grid;
  width: min(560px, calc(100% - 48px));
  gap: 26px;
  margin-left: clamp(28px, 8vw, 112px);
  padding: 62px 0 34px;
}

.title-brand { display: grid; justify-items: start; gap: 9px; }
.brand-mark {
  display: inline-grid;
  width: 42px;
  height: 42px;
  place-items: center;
  border: 1px solid rgba(255, 255, 255, 0.42);
  border-radius: 6px;
  background: rgba(7, 12, 17, 0.62);
  color: white;
  font-size: 20px;
  font-weight: 900;
}

.title-brand h1 {
  margin: 0;
  color: white;
  font-size: 52px;
  font-weight: 900;
  letter-spacing: 0;
  line-height: 1;
}

.title-brand p {
  margin: 0;
  color: rgba(255, 255, 255, 0.72);
  font-size: 14px;
  font-weight: 600;
}

.title-menu {
  display: grid;
  width: min(440px, 100%);
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 7px;
}

.title-command {
  display: grid;
  min-width: 0;
  min-height: 40px;
  grid-template-columns: 20px minmax(0, 1fr) 16px;
  align-items: center;
  gap: 7px;
  padding: 0 11px;
  border: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 6px;
  background: rgba(7, 12, 17, 0.72);
  color: rgba(255, 255, 255, 0.82);
  font: inherit;
  font-size: 10px;
  font-weight: 800;
  text-align: left;
  cursor: pointer;
}

.title-command span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.title-command > svg:last-child { color: rgba(255, 255, 255, 0.4); }
.title-command:hover { border-color: rgba(255, 255, 255, 0.46); background: rgba(7, 12, 17, 0.9); color: white; }
.title-command.primary { min-height: 46px; grid-column: 1 / -1; border-color: var(--brand); background: color-mix(in srgb, var(--brand) 86%, #071018); color: #071018; }
.title-command.primary > svg:last-child { color: rgba(7, 16, 24, 0.58); }
.title-command.primary:hover { background: var(--brand-light); }

.title-status {
  position: relative;
  z-index: 3;
  display: flex;
  min-height: 70px;
  align-items: center;
  justify-content: space-between;
  gap: 20px;
  padding: 10px clamp(20px, 4vw, 56px);
  border-top: 1px solid rgba(255, 255, 255, 0.13);
  background: #0b1015;
}

.scene-status,
.title-meta {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 9px;
}

.scene-status { color: rgba(255, 255, 255, 0.48); font-size: 9px; text-transform: uppercase; }
.scene-status strong { overflow: hidden; color: rgba(255, 255, 255, 0.84); font-size: 10px; text-overflow: ellipsis; white-space: nowrap; text-transform: none; }
.status-dot { width: 7px; height: 7px; flex: 0 0 7px; border-radius: 50%; background: var(--success); box-shadow: 0 0 0 3px color-mix(in srgb, var(--success) 15%, transparent); }
.title-meta { color: rgba(255, 255, 255, 0.48); font-size: 9px; }
.title-meta span + span { padding-left: 10px; border-left: 1px solid rgba(255, 255, 255, 0.14); }

@media (max-width: 700px) {
  .title-hero { min-height: calc(100svh - 58px); align-items: end; }
  .title-utility { top: 12px; right: 12px; }
  .engine-version { display: none; }
  .title-content { width: calc(100% - 28px); gap: 19px; margin: 0 14px; padding: 76px 0 22px; }
  .brand-mark { width: 36px; height: 36px; font-size: 17px; }
  .title-brand h1 { font-size: 36px; }
  .title-brand p { font-size: 11px; }
  .title-menu { width: 100%; }
  .title-command { min-height: 38px; }
  .title-command.primary { min-height: 44px; }
  .title-status { min-height: 58px; padding: 8px 14px; }
  .title-meta { display: none; }
  .scene-status { width: 100%; }
}

@media (max-height: 620px) {
  .title-content { gap: 16px; padding-top: 52px; }
  .brand-mark { display: none; }
  .title-brand h1 { font-size: 36px; }
  .title-menu { gap: 5px; }
  .title-command, .title-command.primary { min-height: 36px; }
}
</style>
