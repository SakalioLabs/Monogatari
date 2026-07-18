<template>
  <Transition name="npc-drawer">
    <div v-if="open && character" class="npc-layer" @keydown.esc.stop.prevent="emit('close')">
      <div class="npc-scrim" aria-hidden="true" @click="emit('close')"></div>
      <aside
        class="npc-panel"
        data-testid="npc-panel"
        :data-npc-runtime="runtimeKind"
        :aria-label="t('npc.open', 'Talk to {name}', { name: character.name })"
      >
        <header class="npc-header">
          <div class="npc-identity">
            <img v-if="avatarUrl" :src="avatarUrl" :alt="character.name" />
            <div v-else class="npc-avatar-fallback">{{ initials(character.name) }}</div>
            <div class="npc-title-copy">
              <span class="npc-eyebrow"><MessageCircleMore :size="13" aria-hidden="true" />{{ t('npc.title', 'LLM NPC') }}</span>
              <h2>{{ character.name }}</h2>
            </div>
          </div>
          <div class="npc-header-actions">
            <span class="npc-runtime-badge"><MonitorCog :size="13" aria-hidden="true" />{{ runtimeLabel }}</span>
            <button
              class="npc-icon-command"
              data-testid="npc-clear"
              :disabled="messages.length === 0 || isGenerating || isPreparing"
              :title="t('npc.clear', 'Clear conversation')"
              :aria-label="t('npc.clear', 'Clear conversation')"
              @click="clearConversation"
            ><Trash2 :size="15" /></button>
            <button
              class="npc-icon-command"
              data-testid="npc-close"
              :title="t('common.close', 'Close')"
              :aria-label="t('common.close', 'Close')"
              @click="emit('close')"
            ><X :size="16" /></button>
          </div>
        </header>

        <section ref="messageList" class="npc-messages" role="log" aria-live="polite">
          <div v-if="isPreparing" class="npc-status" role="status">
            <LoaderCircle class="npc-spinner" :size="22" aria-hidden="true" />
            <span>{{ t('npc.history-loading', 'Restoring conversation') }}</span>
          </div>
          <div v-else-if="messages.length === 0 && !isGenerating" class="npc-empty">
            <MessageCircleMore :size="28" aria-hidden="true" />
            <strong>{{ t('npc.ready', 'Signal ready') }}</strong>
            <p>{{ character.description }}</p>
          </div>

          <article
            v-for="message in messages"
            :key="message.id"
            class="npc-message"
            :class="message.role"
            :data-npc-message-role="message.role"
          >
            <div class="npc-message-meta">
              <span>{{ message.role === 'player' ? t('npc.player', 'You') : character.name }}</span>
              <small v-if="message.emotion">{{ message.emotion }}</small>
            </div>
            <p>{{ message.content }}</p>
          </article>

          <article v-if="isGenerating" class="npc-message character pending" data-testid="npc-generating">
            <div class="npc-message-meta"><span>{{ character.name }}</span></div>
            <p v-if="streamingReply">{{ streamingReply }}</p>
            <div v-else class="npc-generating"><LoaderCircle class="npc-spinner" :size="16" aria-hidden="true" />{{ t('chat.generating', 'Generating') }}</div>
          </article>
        </section>

        <footer class="npc-composer">
          <div v-if="knowledgeEvidenceCount > 0 || relationshipDelta !== 0 || storyChanged" class="npc-evidence" aria-live="polite">
            <span v-if="knowledgeEvidenceCount > 0"><ShieldCheck :size="13" aria-hidden="true" />{{ t('npc.knowledge-evidence', '{count} pinned knowledge refs', { count: knowledgeEvidenceCount }) }}</span>
            <span v-if="relationshipDelta !== 0">{{ t('npc.relationship-change', 'Relation {value}', { value: signedNumber(relationshipDelta) }) }}</span>
            <span v-if="storyChanged">{{ t('npc.story-updated', 'Story state updated') }}</span>
          </div>
          <p v-if="runtimeIssue" class="npc-error" role="alert">{{ runtimeIssue }}</p>
          <p v-if="errorMessage" class="npc-error" role="alert">{{ errorMessage }}</p>
          <div class="npc-input-row">
            <textarea
              ref="inputElement"
              v-model="inputText"
              data-testid="npc-input"
              rows="2"
              maxlength="4000"
              :disabled="Boolean(runtimeIssue) || isPreparing || isGenerating"
              :placeholder="t('npc.placeholder', 'Message {name}', { name: character.name })"
              @keydown.enter.exact.stop.prevent="sendMessage"
            ></textarea>
            <button
              class="npc-send"
              data-testid="npc-send"
              :disabled="!canSend"
              :title="t('chat.send', 'Send')"
              :aria-label="t('chat.send', 'Send')"
              @click="sendMessage"
            ><Send :size="18" /></button>
          </div>
        </footer>
      </aside>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import {
  LoaderCircle,
  MessageCircleMore,
  MonitorCog,
  Send,
  ShieldCheck,
  Trash2,
  X,
} from '@lucide/vue'
import { resolveAssetUrl } from '../lib/assets'
import { useI18n } from '../lib/i18n'
import { loadKnowledgeAuthoringCatalog, type KnowledgeEntryDefinition } from '../lib/knowledgeContent'
import {
  buildWebNpcChatMessages,
  countResolvedNpcKnowledge,
  createNpcConversationMessage,
  normalizeDesktopNpcReply,
  normalizeNpcHistory,
  sanitizeWebNpcReply,
  stripWebNpcPrivateReasoning,
  type NpcConversationMessage,
  type NpcDesktopChatResponse,
} from '../lib/npcConversation'
import type { StoryCharacterInfo } from '../lib/storyContent'
import { invokeCommand } from '../lib/tauri'
import { detectWebGpuSupport, generateWebGpuChat } from '../lib/webgpuInference'

const props = defineProps<{
  open: boolean
  character: StoryCharacterInfo | null
  desktopRuntime: boolean
  locale: string
}>()

const emit = defineEmits<{
  close: []
  emotion: [emotion: string]
  storyProgress: []
}>()

const { t } = useI18n()
const messages = ref<NpcConversationMessage[]>([])
const inputText = ref('')
const errorMessage = ref<string | null>(null)
const isPreparing = ref(false)
const isGenerating = ref(false)
const streamingReply = ref('')
const knowledgeEvidenceCount = ref(0)
const relationshipDelta = ref(0)
const storyChanged = ref(false)
const messageList = ref<HTMLElement>()
const inputElement = ref<HTMLTextAreaElement>()
const browserSessions = new Map<string, NpcConversationMessage[]>()
let knowledgePromise: Promise<KnowledgeEntryDefinition[]> | null = null
let activeKnowledge: KnowledgeEntryDefinition[] = []
let loadSequence = 0
let messageSequence = 0
let generationSequence = 0

const runtimeKind = computed(() => props.desktopRuntime ? 'tauri' : 'webgpu')
const runtimeLabel = computed(() => props.desktopRuntime
  ? t('chat.desktop-runtime', 'Windows DirectML')
  : t('chat.webgpu-runtime', 'WebGPU runtime'))
const avatarUrl = computed(() => resolveAssetUrl(
  props.character?.portrait_path || props.character?.sprite_path || null,
))
const runtimeIssue = computed(() => {
  if (props.desktopRuntime) return null
  const support = detectWebGpuSupport()
  if (support.available) return null
  if (support.reason === 'insecure-context') {
    return t('npc.webgpu-insecure', 'WebGPU requires a secure context.')
  }
  return t('npc.webgpu-unavailable', 'WebGPU is unavailable in this browser.')
})
const canSend = computed(() => Boolean(
  props.character
  && inputText.value.trim()
  && !runtimeIssue.value
  && !isPreparing.value
  && !isGenerating.value,
))

watch(
  () => [props.open, props.character?.id] as const,
  ([open]) => {
    if (open) void prepareConversation()
  },
  { immediate: true },
)

async function prepareConversation() {
  const character = props.character
  if (!character) return
  const requestId = ++loadSequence
  isPreparing.value = true
  errorMessage.value = null
  relationshipDelta.value = 0
  storyChanged.value = false

  try {
    if (props.desktopRuntime) {
      const history = await invokeCommand<unknown>('get_chat_history', { characterId: character.id })
      if (requestId !== loadSequence) return
      messages.value = normalizeNpcHistory(history)
      knowledgeEvidenceCount.value = uniqueKnowledgeRefs(character).length
    } else {
      activeKnowledge = await browserKnowledgeEntries()
      if (requestId !== loadSequence) return
      messages.value = [...(browserSessions.get(character.id) || [])]
      knowledgeEvidenceCount.value = countResolvedNpcKnowledge(character, activeKnowledge)
    }
  } catch (error) {
    if (requestId !== loadSequence) return
    messages.value = []
    errorMessage.value = withErrorDetail(
      t('npc.history-error', 'Conversation history could not be loaded.'),
      error,
    )
  } finally {
    if (requestId === loadSequence) {
      isPreparing.value = false
      scrollToBottom()
      nextTick(() => inputElement.value?.focus())
    }
  }
}

async function sendMessage() {
  const character = props.character
  const text = inputText.value.trim()
  if (!character || !text || !canSend.value) return

  errorMessage.value = null
  relationshipDelta.value = 0
  storyChanged.value = false
  streamingReply.value = ''
  const timestamp = new Date().toISOString()
  messages.value.push(createNpcConversationMessage('player', text, null, timestamp, ++messageSequence))
  persistBrowserSession(character.id)
  const requestMessages = [...messages.value]
  const requestId = ++generationSequence
  inputText.value = ''
  isGenerating.value = true
  scrollToBottom()

  try {
    if (props.desktopRuntime) {
      const response = await invokeCommand<NpcDesktopChatResponse>('send_chat_message', {
        characterId: character.id,
        message: text,
      })
      const reply = normalizeDesktopNpcReply(response)
      const replyMessage = createNpcConversationMessage(
        'character',
        reply.content,
        reply.emotion,
        new Date().toISOString(),
        ++messageSequence,
      )
      if (props.character?.id === character.id) {
        messages.value.push(replyMessage)
        relationshipDelta.value = reply.relationshipDelta
        storyChanged.value = reply.storyChanged
        knowledgeEvidenceCount.value = reply.safetyTrace?.pinned_knowledge_ref_count
          ?? knowledgeEvidenceCount.value
        emit('emotion', reply.emotion)
      }
      if (reply.storyChanged) emit('storyProgress')
    } else {
      let rawReply = ''
      const generated = await generateWebGpuChat(
        buildWebNpcChatMessages(character, props.locale, messages.value, activeKnowledge),
        {
          onChunk(chunk) {
            rawReply += chunk
            if (props.character?.id === character.id && requestId === generationSequence) {
              streamingReply.value = stripWebNpcPrivateReasoning(rawReply)
              scrollToBottom()
            }
          },
        },
      )
      const content = sanitizeWebNpcReply(rawReply || generated)
      const emotion = character.emotion || 'neutral'
      const replyMessage = createNpcConversationMessage(
        'character',
        content,
        emotion,
        new Date().toISOString(),
        ++messageSequence,
      )
      const session = [...requestMessages, replyMessage]
      browserSessions.set(character.id, session)
      if (props.character?.id === character.id && requestId === generationSequence) {
        messages.value = session
        emit('emotion', emotion)
      }
    }
  } catch (error) {
    if (props.character?.id === character.id && requestId === generationSequence) {
      errorMessage.value = withErrorDetail(t('npc.generation-error', 'Reply generation failed.'), error)
    }
  } finally {
    if (requestId === generationSequence) {
      isGenerating.value = false
      streamingReply.value = ''
      scrollToBottom()
      nextTick(() => inputElement.value?.focus())
    }
  }
}

async function clearConversation() {
  const character = props.character
  if (!character || isGenerating.value || isPreparing.value) return
  errorMessage.value = null
  try {
    if (props.desktopRuntime) {
      await invokeCommand<void>('clear_chat_history', { characterId: character.id })
    } else {
      browserSessions.set(character.id, [])
    }
    messages.value = []
    relationshipDelta.value = 0
    storyChanged.value = false
  } catch (error) {
    errorMessage.value = withErrorDetail(t('npc.clear-error', 'Conversation could not be cleared.'), error)
  }
}

function browserKnowledgeEntries(): Promise<KnowledgeEntryDefinition[]> {
  if (!knowledgePromise) {
    knowledgePromise = loadKnowledgeAuthoringCatalog()
      .then((catalog) => catalog.entries)
      .catch(() => [])
  }
  return knowledgePromise
}

function persistBrowserSession(characterId: string) {
  if (!props.desktopRuntime) browserSessions.set(characterId, [...messages.value])
}

function uniqueKnowledgeRefs(character: StoryCharacterInfo): string[] {
  return [...new Set((character.knowledge_refs || character.knowledge || []).filter(Boolean))]
}

function scrollToBottom() {
  nextTick(() => {
    if (messageList.value) messageList.value.scrollTop = messageList.value.scrollHeight
  })
}

function initials(name: string): string {
  return [...name.trim()].slice(0, 2).join('').toUpperCase() || 'NPC'
}

function signedNumber(value: number): string {
  const rounded = value.toFixed(2)
  return value > 0 ? `+${rounded}` : rounded
}

function withErrorDetail(summary: string, error: unknown): string {
  const detail = error instanceof Error ? error.message : String(error)
  return detail && detail !== '[object Object]' ? `${summary} ${detail}` : summary
}
</script>

<style scoped>
.npc-layer {
  position: fixed;
  z-index: 45;
  inset: 0;
  display: grid;
  justify-items: end;
  overflow: hidden;
}

.npc-scrim {
  position: absolute;
  inset: 0;
  background: rgba(4, 7, 8, 0.46);
}

.npc-panel {
  position: relative;
  display: grid;
  width: min(400px, 100vw);
  height: 100svh;
  min-height: 0;
  grid-template-rows: auto minmax(0, 1fr) auto;
  border-left: 1px solid var(--border, rgba(235, 231, 214, 0.2));
  background: rgba(16, 19, 20, 0.98);
  box-shadow: -18px 0 48px rgba(0, 0, 0, 0.38);
  color: var(--text-primary, #f4f3ed);
}

.npc-header {
  display: grid;
  min-width: 0;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: 10px;
  padding: 12px;
  border-bottom: 1px solid var(--border, rgba(235, 231, 214, 0.2));
  background: var(--surface-1, #171b1c);
}

.npc-identity,
.npc-header-actions,
.npc-eyebrow,
.npc-runtime-badge,
.npc-icon-command,
.npc-evidence,
.npc-evidence span,
.npc-generating,
.npc-input-row,
.npc-send {
  display: flex;
  align-items: center;
}

.npc-identity { min-width: 0; gap: 9px; }
.npc-identity img,
.npc-avatar-fallback {
  width: 38px;
  height: 38px;
  flex: 0 0 38px;
  border: 1px solid var(--border, rgba(235, 231, 214, 0.2));
  border-radius: 6px;
}

.npc-identity img { object-fit: cover; }
.npc-avatar-fallback {
  display: grid;
  place-items: center;
  background: var(--surface-2, #222829);
  color: var(--brand-light, #f0d78e);
  font-size: 10px;
  font-weight: 800;
}

.npc-title-copy { min-width: 0; }
.npc-title-copy h2 {
  margin: 2px 0 0;
  overflow: hidden;
  font-size: 15px;
  line-height: 1.25;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.npc-eyebrow {
  gap: 5px;
  color: var(--text-tertiary, #9da39f);
  font-size: 9px;
  font-weight: 800;
  letter-spacing: 0;
  text-transform: uppercase;
}

.npc-header-actions { justify-content: flex-end; gap: 6px; }
.npc-runtime-badge {
  min-height: 28px;
  gap: 5px;
  padding: 0 7px;
  border: 1px solid rgba(128, 180, 154, 0.45);
  border-radius: 6px;
  background: rgba(128, 180, 154, 0.1);
  color: #a9d2bc;
  font-size: 9px;
  font-weight: 800;
  white-space: nowrap;
}

.npc-icon-command,
.npc-send {
  justify-content: center;
  border: 1px solid var(--border, rgba(235, 231, 214, 0.2));
  border-radius: 6px;
  cursor: pointer;
}

.npc-icon-command {
  width: 30px;
  height: 30px;
  flex: 0 0 30px;
  background: var(--surface-2, #222829);
  color: var(--text-secondary, #c9ccc7);
}

.npc-icon-command:hover:not(:disabled),
.npc-send:hover:not(:disabled) { border-color: var(--brand, #d8b969); color: var(--brand-light, #f0d78e); }
.npc-icon-command:disabled,
.npc-send:disabled { opacity: 0.42; cursor: not-allowed; }

.npc-messages {
  min-height: 0;
  padding: 14px;
  overflow-y: auto;
  scrollbar-width: thin;
}

.npc-status,
.npc-empty {
  display: grid;
  min-height: 100%;
  place-items: center;
  align-content: center;
  gap: 8px;
  color: var(--text-tertiary, #9da39f);
  text-align: center;
}

.npc-status { grid-template-columns: auto auto; font-size: 12px; }
.npc-empty strong { color: var(--text-secondary, #c9ccc7); font-size: 13px; }
.npc-empty p {
  max-width: 30ch;
  margin: 0;
  color: var(--text-tertiary, #9da39f);
  font-size: 11px;
  line-height: 1.55;
}

.npc-message {
  display: grid;
  gap: 5px;
  margin: 0 0 14px;
}

.npc-message.player { margin-left: 36px; }
.npc-message.character { margin-right: 24px; }
.npc-message-meta {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 8px;
  color: var(--text-tertiary, #9da39f);
  font-size: 9px;
  font-weight: 800;
}

.npc-message-meta small { color: #a9d2bc; font-size: 9px; }
.npc-message p {
  margin: 0;
  padding: 9px 10px;
  border: 1px solid var(--border, rgba(235, 231, 214, 0.2));
  border-radius: 7px;
  background: var(--surface-1, #171b1c);
  color: var(--text-primary, #f4f3ed);
  font-size: 12px;
  line-height: 1.6;
  overflow-wrap: anywhere;
  white-space: pre-wrap;
}

.npc-message.player p {
  border-color: rgba(216, 185, 105, 0.42);
  background: rgba(216, 185, 105, 0.09);
}

.npc-message.pending { opacity: 0.86; }
.npc-generating { min-height: 38px; gap: 7px; color: var(--text-tertiary, #9da39f); font-size: 11px; }
.npc-spinner { animation: npc-spin 0.8s linear infinite; }

.npc-composer {
  display: grid;
  gap: 8px;
  padding: 10px 12px 12px;
  border-top: 1px solid var(--border, rgba(235, 231, 214, 0.2));
  background: var(--surface-1, #171b1c);
}

.npc-evidence {
  min-width: 0;
  flex-wrap: wrap;
  gap: 6px 10px;
  color: #a9d2bc;
  font-size: 9px;
  font-weight: 700;
}

.npc-evidence span { gap: 4px; }
.npc-error {
  margin: 0;
  padding: 7px 8px;
  border-left: 2px solid #d98080;
  background: rgba(217, 128, 128, 0.08);
  color: #f0b3b3;
  font-size: 10px;
  line-height: 1.4;
}

.npc-input-row { align-items: stretch; gap: 8px; }
.npc-input-row textarea {
  width: 100%;
  min-width: 0;
  min-height: 58px;
  max-height: 128px;
  resize: vertical;
  border: 1px solid var(--border, rgba(235, 231, 214, 0.2));
  border-radius: 7px;
  outline: 0;
  background: var(--surface-0, #101314);
  color: var(--text-primary, #f4f3ed);
  font: inherit;
  font-size: 12px;
  line-height: 1.45;
  padding: 9px 10px;
}

.npc-input-row textarea:focus { border-color: var(--brand, #d8b969); }
.npc-input-row textarea::placeholder { color: var(--text-tertiary, #9da39f); }
.npc-send {
  width: 42px;
  flex: 0 0 42px;
  background: var(--brand, #d8b969);
  color: #171b1c;
}

.npc-drawer-enter-active,
.npc-drawer-leave-active { transition: opacity 0.18s ease; }
.npc-drawer-enter-active .npc-panel,
.npc-drawer-leave-active .npc-panel { transition: transform 0.18s ease; }
.npc-drawer-enter-from,
.npc-drawer-leave-to { opacity: 0; }
.npc-drawer-enter-from .npc-panel,
.npc-drawer-leave-to .npc-panel { transform: translateX(100%); }

@keyframes npc-spin { to { transform: rotate(360deg); } }

@media (max-width: 720px) {
  .npc-scrim { background: transparent; }
  .npc-panel { width: 100vw; border-left: 0; }
  .npc-header { padding: 10px; }
  .npc-runtime-badge { max-width: 92px; overflow: hidden; text-overflow: ellipsis; }
  .npc-messages { padding: 12px; }
  .npc-composer { padding: 9px 10px max(10px, env(safe-area-inset-bottom)); }
}
</style>
