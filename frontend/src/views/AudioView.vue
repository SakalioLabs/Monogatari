<template>
  <div class="audio-page">
    <header class="page-header">
      <div>
        <span class="eyebrow">Media</span>
        <h1>{{ t("audio.title", "Audio Manager") }}</h1>
        <p>Manage background music, ambient sounds, and sound effects for your scenes.</p>
      </div>
      <div class="header-actions">
        <button class="btn btn-secondary btn-sm" @click="refreshTracks">Refresh</button>
        <button class="btn btn-primary btn-sm" @click="showAddTrack = true">Add Track</button>
      </div>
    </header>

    <section class="audio-grid">
      <div class="audio-section">
        <div class="section-head">
          <span class="eyebrow">BGM</span>
          <strong>{{ bgmTracks.length }} tracks</strong>
        </div>
        <div v-for="track in bgmTracks" :key="track.id" class="track-card" :class="{ playing: playingId === track.id }">
          <div class="track-info">
            <span class="track-icon">{{ track.type === 'bgm' ? 'B' : 'A' }}</span>
            <div class="track-meta">
              <strong>{{ track.name }}</strong>
              <span>{{ track.file_path || 'No file' }}</span>
            </div>
          </div>
          <div class="track-controls">
            <button class="ctrl-btn" @click="togglePlay(track)">{{ playingId === track.id ? '||' : '>' }}</button>
            <input type="range" :value="track.volume" min="0" max="100" @input="setVolume(track, $event)" class="vol-slider" />
            <span class="vol-label">{{ track.volume }}%</span>
          </div>
        </div>
        <div v-if="bgmTracks.length === 0" class="empty-section">No BGM tracks configured.</div>
      </div>

      <div class="audio-section">
        <div class="section-head">
          <span class="eyebrow">SFX</span>
          <strong>{{ sfxTracks.length }} effects</strong>
        </div>
        <div v-for="track in sfxTracks" :key="track.id" class="track-card">
          <div class="track-info">
            <span class="track-icon sfx">S</span>
            <div class="track-meta">
              <strong>{{ track.name }}</strong>
              <span>{{ track.file_path || 'No file' }}</span>
            </div>
          </div>
          <div class="track-controls">
            <button class="ctrl-btn" @click="playSfx(track)">>|</button>
            <input type="range" :value="track.volume" min="0" max="100" @input="setVolume(track, $event)" class="vol-slider" />
            <span class="vol-label">{{ track.volume }}%</span>
          </div>
        </div>
        <div v-if="sfxTracks.length === 0" class="empty-section">No sound effects configured.</div>
      </div>
    </section>

    <section class="mixer-panel">
      <div class="section-head">
        <span class="eyebrow">Master Mix</span>
        <strong>Volume Control</strong>
      </div>
      <div class="mixer-grid">
        <label class="mixer-channel"><span>Master</span><input type="range" v-model="masterVolume" min="0" max="100" /><b>{{ masterVolume }}%</b></label>
        <label class="mixer-channel"><span>BGM</span><input type="range" v-model="bgmVolume" min="0" max="100" /><b>{{ bgmVolume }}%</b></label>
        <label class="mixer-channel"><span>SFX</span><input type="range" v-model="sfxVolume" min="0" max="100" /><b>{{ sfxVolume }}%</b></label>
        <label class="mixer-channel"><span>Voice</span><input type="range" v-model="voiceVolume" min="0" max="100" /><b>{{ voiceVolume }}%</b></label>
      </div>
    </section>

    <Transition name="fade">
      <div v-if="showAddTrack" class="modal-overlay" @click.self="showAddTrack = false">
        <div class="modal">
          <div class="modal-head">
            <span class="eyebrow">Add Track</span>
            <button class="close-btn" @click="showAddTrack = false">Close</button>
          </div>
          <div class="form-stack">
            <label class="form-field"><span>Track Name</span><input v-model="newTrackName" class="input" placeholder="My Track" /></label>
            <label class="form-field"><span>Type</span><select v-model="newTrackType" class="input"><option value="bgm">Background Music</option><option value="sfx">Sound Effect</option><option value="ambient">Ambient</option></select></label>
            <label class="form-field"><span>File Path</span><input v-model="newTrackPath" class="input" placeholder="assets/audio/track.mp3" /></label>
            <button class="btn btn-primary" :disabled="!newTrackName.trim()" @click="addTrack">Add Track</button>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from '../lib/i18n'
import { ref, computed } from 'vue'

interface AudioTrack {
  id: string
  name: string
  type: 'bgm' | 'sfx' | 'ambient'
  file_path: string
  volume: number
  loop: boolean
}

const tracks = ref<AudioTrack[]>([
  { id: 'bgm_main', name: 'Main Theme', type: 'bgm', file_path: 'assets/audio/main_theme.mp3', volume: 80, loop: true },
  { id: 'bgm_calm', name: 'Calm Evening', type: 'bgm', file_path: 'assets/audio/calm.mp3', volume: 70, loop: true },
  { id: 'sfx_click', name: 'UI Click', type: 'sfx', file_path: 'assets/audio/click.wav', volume: 60, loop: false },
  { id: 'sfx_door', name: 'Door Open', type: 'sfx', file_path: 'assets/audio/door.wav', volume: 70, loop: false },
])

const playingId = ref<string | null>(null)
const { t } = useI18n()
const showAddTrack = ref(false)
const newTrackName = ref('')
const newTrackType = ref('bgm')
const newTrackPath = ref('')
const masterVolume = ref(80)
const bgmVolume = ref(80)
const sfxVolume = ref(80)
const voiceVolume = ref(80)

const bgmTracks = computed(() => tracks.value.filter(t => t.type === 'bgm' || t.type === 'ambient'))
const sfxTracks = computed(() => tracks.value.filter(t => t.type === 'sfx'))

function refreshTracks() {}
function togglePlay(track: AudioTrack) { playingId.value = playingId.value === track.id ? null : track.id }
function playSfx(track: AudioTrack) { playingId.value = track.id; setTimeout(() => { playingId.value = null }, 2000) }
function setVolume(track: AudioTrack, event: Event) { track.volume = Number((event.target as HTMLInputElement).value) }
function addTrack() {
  if (!newTrackName.value.trim()) return
  tracks.value.push({ id: 'track_' + Date.now(), name: newTrackName.value.trim(), type: newTrackType.value as any, file_path: newTrackPath.value.trim(), volume: 70, loop: newTrackType.value === 'bgm' })
  showAddTrack.value = false; newTrackName.value = ''; newTrackPath.value = ''
}
</script>

<style scoped>
.audio-page { max-width: 1180px; margin: 0 auto; padding: 34px 40px; }
.page-header { display: flex; justify-content: space-between; gap: 18px; align-items: flex-start; margin-bottom: 24px; }
.page-header h1 { color: var(--text-primary); font-size: 28px; line-height: 1.15; }
.page-header p { color: var(--text-tertiary); font-size: 13px; margin-top: 6px; }
.eyebrow { display: block; color: var(--text-tertiary); font-size: 11px; font-weight: 800; text-transform: uppercase; }
.header-actions { display: flex; gap: 8px; flex-shrink: 0; }
.audio-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 18px; margin-bottom: 18px; }
.audio-section { border: 1px solid var(--border); border-radius: var(--radius); background: var(--surface-1); padding: 18px; }
.section-head { display: flex; justify-content: space-between; align-items: center; margin-bottom: 14px; }
.section-head strong { color: var(--text-primary); }
.track-card { display: flex; justify-content: space-between; align-items: center; gap: 12px; padding: 12px; margin-bottom: 8px; border: 1px solid var(--border); border-radius: var(--radius); background: var(--surface-2); transition: border-color 0.2s; }
.track-card.playing { border-color: var(--brand); background: rgba(45,212,191,0.06); }
.track-info { display: flex; gap: 10px; align-items: center; min-width: 0; }
.track-icon { width: 34px; height: 34px; flex-shrink: 0; display: flex; align-items: center; justify-content: center; border-radius: var(--radius-sm); background: var(--surface-3); color: var(--brand-light); font-weight: 900; font-size: 13px; }
.track-icon.sfx { color: var(--warning); }
.track-meta { min-width: 0; }
.track-meta strong { display: block; color: var(--text-primary); font-size: 13px; }
.track-meta span { color: var(--text-tertiary); font-size: 11px; }
.track-controls { display: flex; gap: 8px; align-items: center; }
.ctrl-btn { width: 32px; height: 32px; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--surface-3); color: var(--text-primary); cursor: pointer; font-weight: 800; font-size: 12px; }
.ctrl-btn:hover { border-color: var(--brand); color: var(--brand-light); }
.vol-slider { width: 80px; accent-color: var(--brand); }
.vol-label { color: var(--text-tertiary); font-size: 11px; min-width: 32px; }
.empty-section { color: var(--text-tertiary); font-size: 13px; text-align: center; padding: 24px; }
.mixer-panel { border: 1px solid var(--border); border-radius: var(--radius); background: var(--surface-1); padding: 18px; }
.mixer-grid { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 14px; margin-top: 14px; }
.mixer-channel { display: grid; gap: 8px; text-align: center; padding: 14px; border: 1px solid var(--border); border-radius: var(--radius); background: var(--surface-2); }
.mixer-channel span { color: var(--text-secondary); font-size: 12px; font-weight: 800; }
.mixer-channel input { accent-color: var(--brand); }
.mixer-channel b { color: var(--brand-light); font-size: 13px; }
.modal-overlay { position: fixed; inset: 0; z-index: 40; display: grid; place-items: center; background: rgba(0,0,0,0.66); backdrop-filter: blur(5px); }
.modal { width: min(460px, calc(100vw - 32px)); border: 1px solid var(--border); border-radius: var(--radius-lg); background: var(--surface-1); box-shadow: var(--shadow-lg); padding: 20px; }
.modal-head { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
.close-btn { border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--surface-1); color: var(--text-secondary); cursor: pointer; padding: 6px 10px; font-weight: 700; }
.close-btn:hover { border-color: var(--brand); color: var(--brand-light); }
.form-stack { display: grid; gap: 14px; }
.form-field { display: grid; gap: 6px; }
.form-field span { color: var(--text-secondary); font-size: 12px; font-weight: 800; }
.fade-enter-active, .fade-leave-active { transition: opacity 0.2s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
@media (max-width: 720px) { .audio-page { padding: 22px 16px; } .audio-grid { grid-template-columns: 1fr; } .mixer-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); } }
</style>