<template>
  <div class="audio-page">
    <header class="page-header">
      <div>
        <span class="eyebrow">{{ t('audio.eyebrow', 'Project media') }}</span>
        <h1>{{ t('audio.title', 'Audio Manager') }}</h1>
        <p>{{ t('audio.subtitle', 'Manage background music, ambience, and sound effects used by your scenes.') }}</p>
      </div>
      <div class="header-actions">
        <button class="btn btn-secondary btn-sm" @click="refreshTracks"><RefreshCw :size="14" />{{ t('common.refresh', 'Refresh') }}</button>
        <button class="btn btn-primary btn-sm" @click="showAddTrack = true"><Plus :size="14" />{{ t('audio.add-track', 'Add track') }}</button>
      </div>
    </header>

    <section class="audio-grid">
      <div class="audio-section">
        <div class="section-head">
          <div><span class="eyebrow">BGM</span><strong>{{ t('audio.music-bed', 'Music and ambience') }}</strong></div>
          <span>{{ t('audio.track-count', '{count} tracks', { count: bgmTracks.length }) }}</span>
        </div>
        <div v-for="track in bgmTracks" :key="track.id" class="track-card" :class="{ playing: isTrackPlaying(track), warning: trackErrors[track.id] }">
          <div class="track-info">
            <span class="track-icon"><component :is="trackTypeIcon(track.type)" :size="16" /></span>
            <div class="track-meta">
              <strong>{{ track.name }}</strong>
              <span>{{ track.file_path || t('audio.no-file', 'No file') }}</span>
              <small v-if="trackErrors[track.id]">{{ trackErrors[track.id] }}</small>
            </div>
          </div>
          <div class="track-controls">
            <button class="ctrl-btn" :title="musicButtonLabel(track)" :aria-label="musicButtonLabel(track)" @click="togglePlay(track)">
              <Pause v-if="isTrackPlaying(track)" :size="15" />
              <Play v-else :size="15" />
            </button>
            <input type="range" :value="track.volume" min="0" max="100" @input="setVolume(track, $event)" class="vol-slider" />
            <span class="vol-label">{{ track.volume }}%</span>
            <button class="ctrl-btn danger" :title="t('audio.delete-track', 'Delete track')" :aria-label="t('audio.delete-track', 'Delete track')" @click="deleteTrack(track)"><Trash2 :size="15" /></button>
          </div>
        </div>
        <div v-if="bgmTracks.length === 0" class="empty-section">{{ t('audio.empty-music', 'No music or ambience tracks configured.') }}</div>
      </div>

      <div class="audio-section">
        <div class="section-head">
          <div><span class="eyebrow">SFX</span><strong>{{ t('audio.sound-effects', 'Sound effects') }}</strong></div>
          <span>{{ t('audio.effect-count', '{count} effects', { count: sfxTracks.length }) }}</span>
        </div>
        <div v-for="track in sfxTracks" :key="track.id" class="track-card" :class="{ playing: isSfxPreviewing(track.id), warning: trackErrors[track.id] }">
          <div class="track-info">
            <span class="track-icon sfx"><Zap :size="16" /></span>
            <div class="track-meta">
              <strong>{{ track.name }}</strong>
              <span>{{ track.file_path || t('audio.no-file', 'No file') }}</span>
              <small v-if="trackErrors[track.id]">{{ trackErrors[track.id] }}</small>
            </div>
          </div>
          <div class="track-controls">
            <button class="ctrl-btn" :title="t('common.preview', 'Preview')" :aria-label="t('common.preview', 'Preview')" @click="playSfx(track)"><Play :size="15" /></button>
            <input type="range" :value="track.volume" min="0" max="100" @input="setVolume(track, $event)" class="vol-slider" />
            <span class="vol-label">{{ track.volume }}%</span>
            <button class="ctrl-btn danger" :title="t('audio.delete-track', 'Delete track')" :aria-label="t('audio.delete-track', 'Delete track')" @click="deleteTrack(track)"><Trash2 :size="15" /></button>
          </div>
        </div>
        <div v-if="sfxTracks.length === 0" class="empty-section">{{ t('audio.empty-effects', 'No sound effects configured.') }}</div>
      </div>
    </section>

    <section class="mixer-panel">
      <div class="section-head">
        <div><span class="eyebrow">{{ t('audio.master-mix', 'Master mix') }}</span><strong>{{ t('audio.volume-control', 'Volume control') }}</strong></div>
        <Volume2 :size="17" />
      </div>
      <div class="mixer-grid">
        <label class="mixer-channel"><span>{{ t('audio.master', 'Master') }}</span><input type="range" v-model.number="masterVolume" min="0" max="100" /><b>{{ masterVolume }}%</b></label>
        <label class="mixer-channel"><span>BGM</span><input type="range" v-model.number="bgmVolume" min="0" max="100" /><b>{{ bgmVolume }}%</b></label>
        <label class="mixer-channel"><span>{{ t('audio.ambient', 'Ambient') }}</span><input type="range" v-model.number="ambientVolume" min="0" max="100" /><b>{{ ambientVolume }}%</b></label>
        <label class="mixer-channel"><span>SFX</span><input type="range" v-model.number="sfxVolume" min="0" max="100" /><b>{{ sfxVolume }}%</b></label>
        <label class="mixer-channel"><span>{{ t('audio.voice', 'Voice') }}</span><input type="range" v-model.number="voiceVolume" min="0" max="100" /><b>{{ voiceVolume }}%</b></label>
      </div>
      <div class="transport-row">
        <span>{{ playingMusicLabel }}</span>
        <button class="btn btn-secondary btn-sm" :disabled="!playingMusicId" @click="stopMusic"><Square :size="13" />{{ t('audio.stop', 'Stop') }}</button>
      </div>
      <div v-if="lastAudioError" class="audio-error">
        {{ lastAudioError }}
      </div>
    </section>

    <Transition name="fade">
      <div v-if="showAddTrack" class="modal-overlay" @click.self="showAddTrack = false">
        <div class="modal">
          <div class="modal-head">
            <div><span class="eyebrow">{{ t('audio.media-library', 'Media library') }}</span><strong>{{ t('audio.add-track', 'Add track') }}</strong></div>
            <button class="close-btn" :title="t('common.close', 'Close')" :aria-label="t('common.close', 'Close')" @click="showAddTrack = false"><X :size="16" /></button>
          </div>
          <div class="form-stack">
            <label class="form-field"><span>{{ t('audio.track-name', 'Track name') }}</span><input v-model="newTrackName" class="input" :placeholder="t('audio.track-name-placeholder', 'My track')" /></label>
            <label class="form-field"><span>{{ t('common.type', 'Type') }}</span><select v-model="newTrackType" class="input"><option value="bgm">{{ t('audio.bgm', 'Background Music') }}</option><option value="sfx">{{ t('audio.sfx', 'Sound Effects') }}</option><option value="ambient">{{ t('audio.ambient', 'Ambient') }}</option></select></label>
            <label class="form-field"><span>{{ t('audio.file-path', 'File path') }}</span><input v-model="newTrackPath" class="input" placeholder="assets/audio/track.mp3" /></label>
            <button class="btn btn-primary" :disabled="!newTrackName.trim()" @click="addTrack"><Plus :size="15" />{{ t('audio.add-track', 'Add track') }}</button>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from '../lib/i18n'
import type { Component } from 'vue'
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { Music2, Pause, Play, Plus, RefreshCw, Square, Trash2, Volume2, Waves, X, Zap } from '@lucide/vue'
import { resolveAssetUrl } from '../lib/assets'

type AudioTrackType = 'bgm' | 'sfx' | 'ambient'

interface AudioTrack {
  id: string
  name: string
  type: AudioTrackType
  file_path: string
  volume: number
  loop: boolean
}

interface AudioManagerState {
  tracks: AudioTrack[]
  masterVolume: number
  bgmVolume: number
  ambientVolume: number
  sfxVolume: number
  voiceVolume: number
}

const audioStateKey = 'monogatari.audio-manager.v1'
const defaultTracks: AudioTrack[] = [
  { id: 'bgm_main', name: 'Main Theme', type: 'bgm', file_path: 'assets/audio/main_theme.mp3', volume: 80, loop: true },
  { id: 'bgm_calm', name: 'Calm Evening', type: 'bgm', file_path: 'assets/audio/calm.mp3', volume: 70, loop: true },
  { id: 'amb_springtown', name: 'Springtown Night', type: 'ambient', file_path: 'assets/audio/springtown_night.ogg', volume: 55, loop: true },
  { id: 'sfx_click', name: 'UI Click', type: 'sfx', file_path: 'assets/audio/click.wav', volume: 60, loop: false },
  { id: 'sfx_door', name: 'Door Open', type: 'sfx', file_path: 'assets/audio/door.wav', volume: 70, loop: false },
]

const { t } = useI18n()
const tracks = ref<AudioTrack[]>(defaultTracks.map(cloneTrack))
const playingMusicId = ref<string | null>(null)
const sfxPreviewIds = ref<Set<string>>(new Set())
const trackErrors = ref<Record<string, string>>({})
const lastAudioError = ref('')
const showAddTrack = ref(false)
const newTrackName = ref('')
const newTrackType = ref<AudioTrackType>('bgm')
const newTrackPath = ref('')
const masterVolume = ref(80)
const bgmVolume = ref(80)
const ambientVolume = ref(80)
const sfxVolume = ref(80)
const voiceVolume = ref(80)
const activeMusicAudio = ref<HTMLAudioElement | null>(null)
const sfxAudioElements = new Map<string, HTMLAudioElement>()
let sfxSerial = 0

const bgmTracks = computed(() => tracks.value.filter(t => t.type === 'bgm' || t.type === 'ambient'))
const sfxTracks = computed(() => tracks.value.filter(t => t.type === 'sfx'))
const playingMusicLabel = computed(() => {
  const track = tracks.value.find(t => t.id === playingMusicId.value)
  return track
    ? t('audio.playing-track', 'Playing {name}', { name: track.name })
    : t('audio.no-music-playing', 'No music playing')
})

onMounted(() => {
  refreshTracks()
  updateActiveAudioVolumes()
})

onBeforeUnmount(() => {
  stopMusic()
  stopAllSfx()
})

watch([masterVolume, bgmVolume, ambientVolume, sfxVolume, voiceVolume], () => {
  clampMixerState()
  updateActiveAudioVolumes()
  persistAudioState()
})

watch(tracks, () => {
  updateActiveAudioVolumes()
  persistAudioState()
}, { deep: true })

function cloneTrack(track: AudioTrack): AudioTrack {
  return {
    ...track,
    type: normalizeTrackType(track.type),
    volume: clampVolume(track.volume, 70),
    loop: track.type === 'sfx' ? false : track.loop !== false,
  }
}

function normalizeTrackType(type: unknown): AudioTrackType {
  return type === 'sfx' || type === 'ambient' ? type : 'bgm'
}

function clampVolume(value: unknown, fallback = 80): number {
  const numeric = Number(value)
  if (!Number.isFinite(numeric)) return fallback
  return Math.max(0, Math.min(100, Math.round(numeric)))
}

function storageAvailable(): boolean {
  return typeof window !== 'undefined' && typeof window.localStorage !== 'undefined'
}

function loadAudioState(): AudioManagerState | null {
  if (!storageAvailable()) return null
  try {
    const raw = window.localStorage.getItem(audioStateKey)
    if (!raw) return null
    const parsed = JSON.parse(raw) as Partial<AudioManagerState>
    if (!Array.isArray(parsed.tracks)) return null
    return {
      tracks: parsed.tracks.map(track => cloneTrack(track as AudioTrack)).filter(track => track.id && track.name),
      masterVolume: clampVolume(parsed.masterVolume, 80),
      bgmVolume: clampVolume(parsed.bgmVolume, 80),
      ambientVolume: clampVolume(parsed.ambientVolume, 80),
      sfxVolume: clampVolume(parsed.sfxVolume, 80),
      voiceVolume: clampVolume(parsed.voiceVolume, 80),
    }
  } catch (error) {
    console.warn('Audio state could not be loaded', error)
    return null
  }
}

function persistAudioState() {
  if (!storageAvailable()) return
  const state: AudioManagerState = {
    tracks: tracks.value.map(cloneTrack),
    masterVolume: masterVolume.value,
    bgmVolume: bgmVolume.value,
    ambientVolume: ambientVolume.value,
    sfxVolume: sfxVolume.value,
    voiceVolume: voiceVolume.value,
  }
  try {
    window.localStorage.setItem(audioStateKey, JSON.stringify(state))
  } catch (error) {
    console.warn('Audio state could not be saved', error)
  }
}

function refreshTracks() {
  const restored = loadAudioState()
  if (restored && restored.tracks.length > 0) {
    tracks.value = restored.tracks
    masterVolume.value = restored.masterVolume
    bgmVolume.value = restored.bgmVolume
    ambientVolume.value = restored.ambientVolume
    sfxVolume.value = restored.sfxVolume
    voiceVolume.value = restored.voiceVolume
  } else if (tracks.value.length === 0) {
    tracks.value = defaultTracks.map(cloneTrack)
  }
  lastAudioError.value = ''
  trackErrors.value = {}
}

function clampMixerState() {
  masterVolume.value = clampVolume(masterVolume.value)
  bgmVolume.value = clampVolume(bgmVolume.value)
  ambientVolume.value = clampVolume(ambientVolume.value)
  sfxVolume.value = clampVolume(sfxVolume.value)
  voiceVolume.value = clampVolume(voiceVolume.value)
}

function trackTypeIcon(type: AudioTrackType): Component {
  if (type === 'ambient') return Waves
  if (type === 'sfx') return Zap
  return Music2
}

function isTrackPlaying(track: AudioTrack) {
  return playingMusicId.value === track.id
}

function isSfxPreviewing(trackId: string) {
  return sfxPreviewIds.value.has(trackId)
}

function musicButtonLabel(track: AudioTrack) {
  return isTrackPlaying(track) ? t('audio.pause', 'Pause') : t('audio.play', 'Play')
}

function channelVolume(track: AudioTrack): number {
  if (track.type === 'sfx') return sfxVolume.value
  if (track.type === 'ambient') return ambientVolume.value
  return bgmVolume.value
}

function effectiveTrackVolume(track: AudioTrack): number {
  return (clampVolume(track.volume) / 100) * (masterVolume.value / 100) * (channelVolume(track) / 100)
}

function applyAudioElementVolume(audio: HTMLAudioElement, track: AudioTrack) {
  audio.volume = Math.max(0, Math.min(1, effectiveTrackVolume(track)))
  audio.loop = track.type !== 'sfx' && track.loop
}

function updateActiveAudioVolumes() {
  const activeTrack = tracks.value.find(track => track.id === playingMusicId.value)
  if (activeTrack && activeMusicAudio.value) {
    applyAudioElementVolume(activeMusicAudio.value, activeTrack)
  }
  for (const [key, audio] of sfxAudioElements) {
    const trackId = key.split(':')[0]
    const track = tracks.value.find(item => item.id === trackId)
    if (track) applyAudioElementVolume(audio, track)
  }
}

function resolvedTrackUrl(track: AudioTrack): string | null {
  return resolveAssetUrl(track.file_path)
}

function createAudioElement(track: AudioTrack): HTMLAudioElement | null {
  const resolvedUrl = resolvedTrackUrl(track)
  if (!resolvedUrl) {
    setTrackError(track.id, t('audio.error.missing-path', 'Missing audio path'))
    return null
  }
  const audio = new Audio(resolvedUrl)
  audio.preload = 'auto'
  applyAudioElementVolume(audio, track)
  audio.onerror = () => {
    setTrackError(track.id, t('audio.error.file-unavailable', 'Audio file unavailable'))
    lastAudioError.value = t('audio.error.track-unavailable', '{name}: audio file unavailable', { name: track.name })
  }
  return audio
}

function setTrackError(trackId: string, message: string | null) {
  const next = { ...trackErrors.value }
  if (message) next[trackId] = message
  else delete next[trackId]
  trackErrors.value = next
}

async function togglePlay(track: AudioTrack) {
  if (playingMusicId.value === track.id) {
    stopMusic()
    return
  }
  await playMusic(track)
}

async function playMusic(track: AudioTrack) {
  stopMusic()
  setTrackError(track.id, null)
  lastAudioError.value = ''
  const audio = createAudioElement(track)
  if (!audio) return
  activeMusicAudio.value = audio
  playingMusicId.value = track.id
  try {
    await audio.play()
  } catch (error) {
    stopMusic()
    setTrackError(track.id, t('audio.error.playback-blocked', 'Playback blocked'))
    lastAudioError.value = t('audio.error.playback-detail', '{name}: playback blocked or unsupported', { name: track.name })
    console.warn('Music playback failed', error)
  }
}

function stopMusic() {
  if (activeMusicAudio.value) {
    activeMusicAudio.value.pause()
    activeMusicAudio.value.removeAttribute('src')
    activeMusicAudio.value.load()
  }
  activeMusicAudio.value = null
  playingMusicId.value = null
}

async function playSfx(track: AudioTrack) {
  setTrackError(track.id, null)
  lastAudioError.value = ''
  const audio = createAudioElement({ ...track, loop: false })
  if (!audio) return
  const key = `${track.id}:${++sfxSerial}`
  markSfxPreview(track.id, true)
  sfxAudioElements.set(key, audio)
  audio.onended = () => finishSfxPreview(key, track.id)
  audio.onerror = () => {
    setTrackError(track.id, t('audio.error.file-unavailable', 'Audio file unavailable'))
    lastAudioError.value = t('audio.error.track-unavailable', '{name}: audio file unavailable', { name: track.name })
    finishSfxPreview(key, track.id)
  }
  try {
    await audio.play()
  } catch (error) {
    setTrackError(track.id, t('audio.error.preview-blocked', 'Preview blocked'))
    lastAudioError.value = t('audio.error.preview-detail', '{name}: preview blocked or unsupported', { name: track.name })
    finishSfxPreview(key, track.id)
    console.warn('SFX preview failed', error)
  }
}

function finishSfxPreview(key: string, trackId: string) {
  const audio = sfxAudioElements.get(key)
  if (audio) {
    audio.pause()
    audio.removeAttribute('src')
    audio.load()
  }
  sfxAudioElements.delete(key)
  const stillPlaying = [...sfxAudioElements.keys()].some(item => item.startsWith(`${trackId}:`))
  if (!stillPlaying) markSfxPreview(trackId, false)
}

function markSfxPreview(trackId: string, active: boolean) {
  const next = new Set(sfxPreviewIds.value)
  if (active) next.add(trackId)
  else next.delete(trackId)
  sfxPreviewIds.value = next
}

function stopAllSfx() {
  for (const [key, audio] of sfxAudioElements) {
    audio.pause()
    audio.removeAttribute('src')
    audio.load()
    sfxAudioElements.delete(key)
  }
  sfxPreviewIds.value = new Set()
}

function setVolume(track: AudioTrack, event: Event) {
  track.volume = clampVolume((event.target as HTMLInputElement).value, track.volume)
  updateActiveAudioVolumes()
}

function addTrack() {
  if (!newTrackName.value.trim()) return
  const type = normalizeTrackType(newTrackType.value)
  const safeName = newTrackName.value.trim()
  const slug = safeName.toLowerCase().replace(/[^a-z0-9]+/g, '_').replace(/^_+|_+$/g, '') || 'track'
  tracks.value.push({
    id: `track_${slug}_${Date.now()}`,
    name: safeName,
    type,
    file_path: newTrackPath.value.trim(),
    volume: 70,
    loop: type !== 'sfx',
  })
  showAddTrack.value = false
  newTrackName.value = ''
  newTrackPath.value = ''
}

function deleteTrack(track: AudioTrack) {
  if (!window.confirm(t('audio.delete-confirm', 'Delete track "{name}"?', { name: track.name }))) return
  if (playingMusicId.value === track.id) stopMusic()
  stopAllSfx()
  tracks.value = tracks.value.filter(item => item.id !== track.id)
  setTrackError(track.id, null)
}
</script>

<style scoped>
.audio-page { max-width: 1180px; margin: 0 auto; padding: 32px 36px 48px; }
.page-header { display: flex; justify-content: space-between; gap: 18px; align-items: flex-start; margin-bottom: 24px; }
.page-header h1 { margin: 3px 0 0; color: var(--text-primary); font-size: 28px; line-height: 1.15; }
.page-header p { color: var(--text-tertiary); font-size: 13px; margin-top: 6px; }
.eyebrow { display: block; color: var(--text-tertiary); font-size: 11px; font-weight: 800; text-transform: uppercase; }
.header-actions { display: flex; gap: 8px; flex-shrink: 0; }
.btn { display: inline-flex; align-items: center; justify-content: center; gap: 7px; }
.audio-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 24px; margin-bottom: 20px; }
.audio-section { min-width: 0; }
.section-head { display: flex; justify-content: space-between; align-items: center; gap: 12px; min-height: 46px; margin-bottom: 10px; padding-bottom: 10px; border-bottom: 1px solid var(--border); }
.section-head strong { display: block; margin-top: 3px; color: var(--text-primary); font-size: 14px; }
.section-head > span, .section-head > svg { color: var(--text-tertiary); font-size: 10px; font-weight: 700; }
.track-card { display: flex; justify-content: space-between; align-items: center; gap: 12px; padding: 12px; margin-bottom: 8px; border: 1px solid var(--border); border-radius: var(--radius); background: var(--surface-2); transition: border-color 0.2s; }
.track-card.playing { border-color: var(--brand); background: color-mix(in srgb, var(--brand) 5%, var(--surface-1)); }
.track-card.warning { border-color: rgba(239,68,68,0.35); }
.track-info { display: flex; gap: 10px; align-items: center; min-width: 0; }
.track-icon { width: 34px; height: 34px; flex-shrink: 0; display: grid; place-items: center; border-radius: var(--radius-sm); background: var(--surface-3); color: var(--brand-light); }
.track-icon.sfx { color: var(--warning); }
.track-meta { min-width: 0; }
.track-meta strong { display: block; color: var(--text-primary); font-size: 13px; }
.track-meta span, .track-meta small { display: block; max-width: 320px; overflow: hidden; color: var(--text-tertiary); font-size: 11px; text-overflow: ellipsis; white-space: nowrap; }
.track-meta small { color: var(--danger); }
.track-controls { display: flex; gap: 8px; align-items: center; }
.ctrl-btn { display: grid; place-items: center; width: 32px; height: 32px; padding: 0; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--surface-3); color: var(--text-primary); cursor: pointer; }
.ctrl-btn:hover { border-color: var(--brand); color: var(--brand-light); }
.ctrl-btn.danger:hover { border-color: var(--danger); color: var(--danger); }
.vol-slider { width: 72px; accent-color: var(--brand); }
.vol-label { color: var(--text-tertiary); font-size: 11px; min-width: 32px; }
.empty-section { color: var(--text-tertiary); font-size: 13px; text-align: center; padding: 24px; }
.mixer-panel { border: 1px solid var(--border); border-radius: var(--radius); background: var(--surface-1); padding: 18px; }
.mixer-grid { display: grid; grid-template-columns: repeat(5, minmax(0, 1fr)); gap: 14px; margin-top: 14px; }
.mixer-channel { display: grid; gap: 8px; text-align: center; padding: 8px 10px; }
.mixer-channel span { color: var(--text-secondary); font-size: 12px; font-weight: 800; }
.mixer-channel input { accent-color: var(--brand); }
.mixer-channel b { color: var(--brand-light); font-size: 13px; }
.transport-row { display: flex; justify-content: space-between; align-items: center; gap: 12px; margin-top: 14px; padding-top: 12px; border-top: 1px solid var(--border); }
.transport-row span { min-width: 0; overflow: hidden; color: var(--text-secondary); font-size: 12px; font-weight: 800; text-overflow: ellipsis; white-space: nowrap; }
.audio-error { margin-top: 10px; padding: 9px 10px; border: 1px solid rgba(239,68,68,0.28); border-radius: var(--radius-sm); background: rgba(239,68,68,0.1); color: var(--danger); font-size: 12px; font-weight: 700; }
.modal-overlay { position: fixed; inset: 0; z-index: 40; display: grid; place-items: center; background: rgba(0,0,0,0.66); backdrop-filter: blur(5px); }
.modal { width: min(460px, calc(100vw - 32px)); border: 1px solid var(--border); border-radius: var(--radius-lg); background: var(--surface-1); box-shadow: var(--shadow-lg); padding: 20px; }
.modal-head { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
.modal-head strong { display: block; margin-top: 3px; color: var(--text-primary); font-size: 15px; }
.close-btn { display: grid; place-items: center; width: 34px; height: 34px; padding: 0; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--surface-1); color: var(--text-secondary); cursor: pointer; }
.close-btn:hover { border-color: var(--brand); color: var(--brand-light); }
.form-stack { display: grid; gap: 14px; }
.form-field { display: grid; gap: 6px; }
.form-field span { color: var(--text-secondary); font-size: 12px; font-weight: 800; }
.fade-enter-active, .fade-leave-active { transition: opacity 0.2s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
@media (max-width: 980px) { .mixer-grid { grid-template-columns: repeat(3, minmax(0, 1fr)); } }
@media (max-width: 720px) { .audio-page { padding: 22px 16px 96px; } .page-header { flex-direction: column; } .header-actions { width: 100%; } .header-actions .btn { flex: 1; } .audio-grid { grid-template-columns: 1fr; } .track-card { align-items: stretch; flex-direction: column; } .track-controls { justify-content: space-between; } .vol-slider { flex: 1; width: auto; } .mixer-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); } }
</style>
