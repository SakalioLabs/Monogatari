<template>
  <div class="editor-page">
    <header class="editor-header">
      <h1>Character Editor</h1>
      <button class="btn btn-primary btn-sm" @click="createNew">+ New Character</button>
    </header>
    <div v-if="editing" class="editor-form">
      <div class="field-grid">
        <div class="field"><label>ID</label><input class="input" v-model="form.id" placeholder="unique_id" /></div>
        <div class="field"><label>Name</label><input class="input" v-model="form.name" placeholder="Character Name" /></div>
        <div class="field full"><label>Description</label><textarea class="input" v-model="form.description" rows="2"></textarea></div>
        <div class="field full"><label>Background</label><textarea class="input" v-model="form.background" rows="3"></textarea></div>
        <div class="field full"><label>Speech Style</label><input class="input" v-model="form.speech_style" placeholder="e.g. cheerful and friendly" /></div>
      </div>
      <div class="personality-section">
        <h3>Personality Traits (Big Five)</h3>
        <div class="trait-grid">
          <div class="trait-row"><label>Openness</label><input type="range" v-model.number="form.openness" min="0" max="1" step="0.1" /><span class="trait-val">{{ form.openness.toFixed(1) }}</span></div>
          <div class="trait-row"><label>Conscientiousness</label><input type="range" v-model.number="form.conscientiousness" min="0" max="1" step="0.1" /><span class="trait-val">{{ form.conscientiousness.toFixed(1) }}</span></div>
          <div class="trait-row"><label>Extraversion</label><input type="range" v-model.number="form.extraversion" min="0" max="1" step="0.1" /><span class="trait-val">{{ form.extraversion.toFixed(1) }}</span></div>
          <div class="trait-row"><label>Agreeableness</label><input type="range" v-model.number="form.agreeableness" min="0" max="1" step="0.1" /><span class="trait-val">{{ form.agreeableness.toFixed(1) }}</span></div>
          <div class="trait-row"><label>Neuroticism</label><input type="range" v-model.number="form.neuroticism" min="0" max="1" step="0.1" /><span class="trait-val">{{ form.neuroticism.toFixed(1) }}</span></div>
        </div>
      </div>
      <div class="actions">
        <button class="btn btn-primary" @click="save">Save Character</button>
        <button class="btn btn-secondary" @click="editing = false">Cancel</button>
      </div>
    </div>
    <div v-else class="char-list">
      <div v-for="cid in charIds" :key="cid" class="char-row" @click="selectChar(cid)">
        <span class="char-id">{{ cid }}</span>
      </div>
      <p v-if="charIds.length === 0" class="muted">No characters found. Click New Character to create one.</p>
    </div>
  </div>
</template>
<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { invokeCommand } from '../lib/tauri'

interface CharForm {
  id: string; name: string; description: string; background: string; speech_style: string;
  openness: number; conscientiousness: number; extraversion: number; agreeableness: number; neuroticism: number;
}

const charIds = ref<string[]>([])
const editing = ref(false)
const form = reactive<CharForm>({
  id: '', name: '', description: '', background: '', speech_style: '',
  openness: 0.5, conscientiousness: 0.5, extraversion: 0.5, agreeableness: 0.5, neuroticism: 0.5
})

const resetForm = () => { Object.assign(form, { id: '', name: '', description: '', background: '', speech_style: '', openness: 0.5, conscientiousness: 0.5, extraversion: 0.5, agreeableness: 0.5, neuroticism: 0.5 }) }

async function loadList() { try { charIds.value = await invokeCommand<string[]>('get_characters', {}, []) } catch {} }
function createNew() { resetForm(); editing.value = true }
async function save() { try { await invokeCommand('create_character', { input: { ...form } }); editing.value = false; await loadList() } catch (e: any) { alert('Save failed: ' + e) } }
async function selectChar(id: string) {
  try {
    const c = await invokeCommand<any>('get_character', { characterId: id })
    if (c) { Object.assign(form, { id: c.id, name: c.name, description: c.description || '', background: c.background || '', speech_style: c.personality?.speech_style || '', openness: c.personality?.openness ?? 0.5, conscientiousness: c.personality?.conscientiousness ?? 0.5, extraversion: c.personality?.extraversion ?? 0.5, agreeableness: c.personality?.agreeableness ?? 0.5, neuroticism: c.personality?.neuroticism ?? 0.5 }); editing.value = true }
  } catch {}
}
onMounted(loadList)
</script>
<style scoped>
.editor-page { padding: 24px; max-width: 800px; margin: 0 auto; }
.editor-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 24px; }
.editor-header h1 { font-size: 22px; font-weight: 700; }
.field-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; }
.field-grid .full { grid-column: 1 / -1; }
.field label { display: block; font-size: 11px; font-weight: 700; color: var(--text-tertiary); text-transform: uppercase; margin-bottom: 6px; }
.personality-section { margin-top: 24px; padding: 20px; background: var(--surface-2); border-radius: var(--radius); }
.personality-section h3 { font-size: 14px; font-weight: 700; margin-bottom: 16px; }
.trait-grid { display: flex; flex-direction: column; gap: 10px; }
.trait-row { display: flex; align-items: center; gap: 12px; }
.trait-row label { width: 140px; font-size: 12px; color: var(--text-secondary); }
.trait-row input[type=range] { flex: 1; }
.trait-val { width: 30px; font-size: 12px; color: var(--text-tertiary); text-align: right; }
.actions { margin-top: 24px; display: flex; gap: 8px; }
.char-list { display: flex; flex-direction: column; gap: 8px; }
.char-row { padding: 12px 16px; background: var(--surface-2); border: 1px solid var(--border); border-radius: var(--radius); cursor: pointer; transition: all 0.15s; }
.char-row:hover { border-color: var(--brand); }
.char-id { font-size: 14px; font-weight: 600; }
.muted { color: var(--text-tertiary); font-size: 13px; }
</style>
