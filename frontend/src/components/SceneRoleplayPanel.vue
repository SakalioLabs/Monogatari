<template>
  <section
    class="roleplay-shell"
    data-testid="scene-roleplay"
    :data-roleplay-status="snapshot.session.status"
    :data-evaluation-source="lastEvaluationSource || undefined"
    :data-evaluation-deltas="lastEvaluationDeltaCount"
    :data-evaluation-evidence="lastEvaluationEvidenceCount"
    :data-current-relationship="currentRelationship.toFixed(3)"
  >
    <header class="roleplay-head">
      <div class="node-copy">
        <span class="node-kicker">{{ snapshot.definition.title }}</span>
        <strong>{{ currentSceneName }}</strong>
        <small>{{ currentNode.player_goal }}</small>
      </div>
      <div class="turn-count">
        <span>{{ t('roleplay.turn', 'Turn') }}</span>
        <strong>{{ snapshot.session.node_turns }} / {{ currentNode.max_turns }}</strong>
      </div>
    </header>

    <div class="score-strip" :aria-label="t('roleplay.scores', 'Story scores')">
      <div v-for="dimension in snapshot.definition.score_dimensions" :key="dimension.id" class="score-item">
        <div class="score-label">
          <span>{{ dimension.label }}</span>
          <strong>{{ formatScore(snapshot.session.scores[dimension.id] || 0) }}</strong>
        </div>
        <div class="score-track" aria-hidden="true">
          <span :style="{ width: `${scorePercent(dimension)}%` }"></span>
        </div>
      </div>
      <div v-if="currentNode.relationship_rule" class="score-item relationship-item">
        <div class="score-label">
          <span>{{ currentCharacter?.name || currentNode.character_id }} · {{ t('roleplay.affinity', 'Affinity') }}</span>
          <strong>{{ formatScore(currentRelationship) }}</strong>
        </div>
        <div class="score-track relationship-track" aria-hidden="true">
          <span :style="{ width: `${relationshipPercent}%` }"></span>
        </div>
      </div>
    </div>

    <div ref="transcriptElement" class="roleplay-transcript" aria-live="polite">
      <template v-for="entry in transcriptEntries" :key="entry.key">
        <div v-if="entry.kind === 'narration'" class="narration-entry">
          <span>{{ entry.scene }}</span>
          <p>{{ entry.content }}</p>
        </div>
        <article v-else class="turn-entry" :class="entry.role">
          <div class="turn-speaker">{{ entry.speaker }}</div>
          <p>{{ entry.content }}</p>
        </article>
      </template>

      <article v-if="pendingPlayer" class="turn-entry player pending">
        <div class="turn-speaker">{{ t('roleplay.you', 'You') }}</div>
        <p>{{ pendingPlayer }}</p>
      </article>
      <article v-if="isGenerating" class="turn-entry character pending">
        <div class="turn-speaker">{{ currentCharacter?.name || currentNode.character_id }}</div>
        <p v-if="streamingReply">{{ streamingReply }}</p>
        <div v-else class="thinking-line"><LoaderCircle :size="16" />{{ t('roleplay.responding', 'Responding') }}</div>
      </article>
    </div>

    <footer v-if="snapshot.session.status === 'active'" class="roleplay-composer">
      <div
        v-if="lastEvaluationSource.startsWith('authored_fallback')"
        class="roleplay-degraded"
        data-testid="roleplay-degraded"
        role="status"
      >
        {{ t('roleplay.evaluation-fallback', 'This turn used an authored recovery because live model generation was unavailable.') }}
      </div>
      <div v-if="errorMessage" class="roleplay-error">{{ errorMessage }}</div>
      <div class="composer-row">
        <textarea
          ref="inputElement"
          v-model="inputText"
          :placeholder="t('roleplay.message-placeholder', 'Say or do something in this scene...')"
          :aria-label="t('roleplay.message-placeholder', 'Say or do something in this scene...')"
          :disabled="isGenerating"
          maxlength="4000"
          rows="2"
          @keydown.enter.exact.prevent="sendTurn"
        ></textarea>
        <button
          class="send-button"
          :disabled="!canSend"
          :title="t('roleplay.send', 'Send')"
          :aria-label="t('roleplay.send', 'Send')"
          @click="sendTurn"
        >
          <LoaderCircle v-if="isGenerating" :size="18" />
          <Send v-else :size="18" />
        </button>
      </div>
    </footer>

    <footer v-else class="roleplay-ending" data-testid="roleplay-ending">
      <span>{{ t('roleplay.ending', 'Ending') }}</span>
      <strong>{{ activeEnding?.title || snapshot.session.ending_id }}</strong>
      <p v-if="activeEnding?.description">{{ activeEnding.description }}</p>
      <div class="ending-actions">
        <button v-if="canContinue" class="continue-button" data-testid="roleplay-continue" @click="emit('continue')">
          {{ t('roleplay.continue', 'Continue') }}<ArrowRight :size="16" />
        </button>
        <button class="restart-button" @click="emit('restart')"><RotateCcw :size="16" />{{ t('roleplay.restart', 'Restart') }}</button>
      </div>
    </footer>
  </section>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { ArrowRight, LoaderCircle, RotateCcw, Send } from '@lucide/vue'

import { useI18n } from '../lib/i18n'
import { loadKnowledgeAuthoringCatalog, type KnowledgeEntryDefinition } from '../lib/knowledgeContent'
import { sanitizeWebNpcReply } from '../lib/npcConversation'
import {
  applyBrowserSceneRoleplayTurn,
  buildBrowserRoleplayEvaluatorMessages,
  buildBrowserRoleplayNpcMessages,
  containedBrowserRoleplayEvaluation,
  evaluateBrowserRoleplayFallback,
  parseBrowserRoleplayEvaluation,
  reconcileBrowserRoleplayEvaluation,
  type RoleplayScoreDimension,
  type SceneRoleplayNode,
  type SceneRoleplaySnapshot,
  type SceneRoleplayTurnResponse,
} from '../lib/sceneRoleplay'
import {
  analyzeRoleplayPlayerInput,
  composeRoleplayGenerationRecovery,
  composeRoleplayIntrusionResponse,
  guardRoleplayNpcResponse,
} from '../lib/sceneRoleplaySafety'
import type { StoryCharacterInfo, StoryEndingInfo } from '../lib/storyContent'
import { invokeCommand } from '../lib/tauri'
import { generateAuthoringApiChat, loadAuthoringApiRuntime } from '../lib/authoringInference'
import { detectWebGpuSupport, generateWebGpuChat } from '../lib/webgpuInference'

const props = defineProps<{
  snapshot: SceneRoleplaySnapshot
  desktopRuntime: boolean
  characters: StoryCharacterInfo[]
  endings: StoryEndingInfo[]
  locale: string
  sceneName: string | null
  canContinue?: boolean
}>()

const emit = defineEmits<{
  update: [snapshot: SceneRoleplaySnapshot]
  nodeChange: [node: SceneRoleplayNode]
  emotion: [emotion: string]
  ending: [endingId: string]
  continue: []
  restart: []
}>()

const { t } = useI18n()
const inputText = ref('')
const pendingPlayer = ref('')
const streamingReply = ref('')
const errorMessage = ref<string | null>(null)
const isGenerating = ref(false)
const lastEvaluationSource = ref('')
const lastEvaluationDeltaCount = ref(0)
const lastEvaluationEvidenceCount = ref(0)
const inputElement = ref<HTMLTextAreaElement>()
const transcriptElement = ref<HTMLElement>()
let knowledgeEntries: KnowledgeEntryDefinition[] = []
let authoringApiAvailable = false
let generationSequence = 0

const currentNode = computed(() => props.snapshot.current_node)
const currentCharacter = computed(() => props.characters.find(character => character.id === currentNode.value.character_id) || null)
const currentRelationship = computed(() => props.snapshot.session.relationships?.[currentNode.value.character_id] || 0)
const relationshipPercent = computed(() => (currentRelationship.value + 1) * 50)
const currentSceneName = computed(() => props.sceneName || currentNode.value.scene_id)
const activeEnding = computed(() => props.endings.find(ending => ending.id === props.snapshot.session.ending_id) || null)
const canSend = computed(() => Boolean(inputText.value.trim() && !isGenerating.value && currentCharacter.value))

type TranscriptEntry =
  | { key: string; kind: 'narration'; scene: string; content: string }
  | { key: string; kind: 'message'; role: 'player' | 'character'; speaker: string; content: string }

const transcriptEntries = computed<TranscriptEntry[]>(() => {
  const entries: TranscriptEntry[] = []
  let previousNode = ''
  for (const turn of props.snapshot.session.transcript) {
    const node = props.snapshot.definition.nodes.find(candidate => candidate.id === turn.node_id)
    if (node && node.id !== previousNode) {
      entries.push({
        key: `narration-${node.id}-${turn.turn}`,
        kind: 'narration',
        scene: node.scene_id,
        content: node.opening_narration,
      })
      previousNode = node.id
    }
    entries.push({
      key: `player-${turn.turn}`,
      kind: 'message',
      role: 'player',
      speaker: t('roleplay.you', 'You'),
      content: turn.player_message,
    })
    const speaker = props.characters.find(character => character.id === node?.character_id)?.name || node?.character_id || ''
    entries.push({
      key: `character-${turn.turn}`,
      kind: 'message',
      role: 'character',
      speaker,
      content: turn.npc_response,
    })
  }
  if (previousNode !== currentNode.value.id && props.snapshot.session.status === 'active') {
    entries.push({
      key: `narration-current-${currentNode.value.id}`,
      kind: 'narration',
      scene: currentNode.value.scene_id,
      content: currentNode.value.opening_narration,
    })
  }
  return entries
})

watch(() => props.snapshot.session.total_turns, scrollToBottom)
watch(() => currentNode.value.id, () => {
  emit('nodeChange', currentNode.value)
  scrollToBottom()
})

onMounted(async () => {
  const [knowledgeResult, authoringRuntime] = await Promise.all([
    loadKnowledgeAuthoringCatalog().catch(() => ({ entries: [] })),
    props.desktopRuntime ? Promise.resolve(null) : loadAuthoringApiRuntime(),
  ])
  knowledgeEntries = knowledgeResult.entries
  authoringApiAvailable = Boolean(authoringRuntime)
  emit('nodeChange', currentNode.value)
  scrollToBottom()
  nextTick(() => inputElement.value?.focus())
})

async function sendTurn() {
  const playerMessage = inputText.value.trim()
  const character = currentCharacter.value
  if (!playerMessage || !character || !canSend.value) return
  const requestId = ++generationSequence
  inputText.value = ''
  pendingPlayer.value = playerMessage
  streamingReply.value = ''
  errorMessage.value = null
  isGenerating.value = true
  scrollToBottom()

  try {
    let response: SceneRoleplayTurnResponse
    if (props.desktopRuntime) {
      response = await invokeCommand<SceneRoleplayTurnResponse>('send_scene_roleplay_turn', {
        roleplayId: props.snapshot.definition.id,
        message: playerMessage,
      })
    } else {
      const inputSafety = analyzeRoleplayPlayerInput(playerMessage)
      let npcResponse: string
      let evaluation
      let evaluationSource: string
      if (inputSafety.intrusion_detected) {
        npcResponse = composeRoleplayIntrusionResponse(currentNode.value, playerMessage)
        evaluation = containedBrowserRoleplayEvaluation(currentNode.value)
        evaluationSource = 'contained_intrusion'
      } else {
        const generateChat = authoringApiAvailable ? generateAuthoringApiChat : generateWebGpuChat
        if (!authoringApiAvailable) {
          const support = detectWebGpuSupport()
          if (!support.available) throw new Error('WebGPU is unavailable in this browser.')
        }
        let rawReply = ''
        let npcCandidate: string | null = null
        try {
          const generated = await generateChat(
            buildBrowserRoleplayNpcMessages(
              props.snapshot.definition,
              props.snapshot.session,
              character,
              props.locale,
              knowledgeEntries,
              playerMessage,
            ),
            {
              maxNewTokens: props.snapshot.definition.inference.npc_max_tokens,
              maxContextCharacters: props.snapshot.definition.inference.max_context_characters,
              recoveryMaxContextCharacters: Math.min(3_000, props.snapshot.definition.inference.max_context_characters),
              onReset() {
                rawReply = ''
              },
              onChunk(chunk) {
                rawReply += chunk
              },
            },
          )
          npcCandidate = sanitizeWebNpcReply(rawReply || generated)
        } catch {
          rawReply = ''
        }
        if (npcCandidate === null) {
          npcResponse = composeRoleplayGenerationRecovery(
            currentNode.value,
            playerMessage,
            props.snapshot.session.node_turns + 1,
          )
          evaluation = evaluateBrowserRoleplayFallback(currentNode.value, playerMessage)
          evaluationSource = 'authored_fallback_npc_inference_error'
        } else {
          const guardedNpc = guardRoleplayNpcResponse(currentNode.value, inputSafety, npcCandidate, {
            player_message: playerMessage,
            node_turn: props.snapshot.session.node_turns + 1,
          })
          npcResponse = guardedNpc.response
          if (guardedNpc.state_contained) {
            evaluation = evaluateBrowserRoleplayFallback(currentNode.value, playerMessage)
            evaluationSource = 'authored_fallback_npc_output'
          } else {
            evaluationSource = authoringApiAvailable ? 'authoring_api_model' : 'browser_model'
            try {
              const evaluatorOutput = await generateChat(
                buildBrowserRoleplayEvaluatorMessages(
                  props.snapshot.definition,
                  props.snapshot.session,
                  playerMessage,
                  npcResponse,
                ),
                {
                  maxNewTokens: props.snapshot.definition.inference.evaluator_max_tokens,
                  temperature: 0,
                  maxContextCharacters: props.snapshot.definition.inference.max_context_characters,
                  recoveryMaxContextCharacters: Math.min(3_000, props.snapshot.definition.inference.max_context_characters),
                },
              )
              evaluation = parseBrowserRoleplayEvaluation(evaluatorOutput)
              const reconciled = reconcileBrowserRoleplayEvaluation(
                currentNode.value,
                playerMessage,
                evaluation,
              )
              evaluation = reconciled.evaluation
              if (reconciled.changed) {
                evaluationSource = authoringApiAvailable
                  ? 'authoring_api_model_reconciled'
                  : 'browser_model_reconciled'
              }
            } catch {
              evaluationSource = 'authored_fallback_evaluator_error'
              evaluation = evaluateBrowserRoleplayFallback(currentNode.value, playerMessage)
            }
          }
        }
      }

      let applied
      try {
        applied = applyBrowserSceneRoleplayTurn(
          props.snapshot.definition,
          props.snapshot.session,
          { player_message: playerMessage, npc_response: npcResponse, evaluation },
        )
      } catch (error) {
        evaluationSource = inputSafety.intrusion_detected
          ? 'contained_intrusion'
          : 'authored_fallback_invalid_evaluation'
        evaluation = inputSafety.intrusion_detected
          ? containedBrowserRoleplayEvaluation(currentNode.value)
          : evaluateBrowserRoleplayFallback(currentNode.value, playerMessage)
        applied = applyBrowserSceneRoleplayTurn(
          props.snapshot.definition,
          props.snapshot.session,
          { player_message: playerMessage, npc_response: npcResponse, evaluation },
        )
      }
      response = { ...applied.response, evaluation, evaluation_source: evaluationSource }
    }

    if (requestId !== generationSequence) return
    const nextSnapshot: SceneRoleplaySnapshot = {
      schema: 'monogatari-scene-roleplay-snapshot/v1',
      definition: props.snapshot.definition,
      session: response.session,
      current_node: response.current_node,
    }
    lastEvaluationSource.value = response.evaluation_source
    lastEvaluationDeltaCount.value = response.evaluation.score_deltas.length
    lastEvaluationEvidenceCount.value = response.evaluation.evidence.length
    emit('update', nextSnapshot)
    if (response.evaluation.npc_emotion) emit('emotion', response.evaluation.npc_emotion)
    if (response.outcome.ending_id) emit('ending', response.outcome.ending_id)
  } catch (error) {
    if (requestId === generationSequence) errorMessage.value = String(error)
  } finally {
    if (requestId === generationSequence) {
      pendingPlayer.value = ''
      streamingReply.value = ''
      isGenerating.value = false
      scrollToBottom()
      nextTick(() => inputElement.value?.focus())
    }
  }
}

function scorePercent(dimension: RoleplayScoreDimension): number {
  const value = props.snapshot.session.scores[dimension.id] || 0
  return Math.max(0, Math.min(100, (value - dimension.min) / (dimension.max - dimension.min) * 100))
}

function formatScore(value: number): string {
  return `${value > 0 ? '+' : ''}${value.toFixed(1)}`
}

function scrollToBottom() {
  nextTick(() => {
    const element = transcriptElement.value
    if (element) element.scrollTop = element.scrollHeight
  })
}
</script>

<style scoped>
.roleplay-shell {
  display: grid;
  grid-template-rows: auto auto minmax(0, 1fr) auto;
  min-height: 0;
  height: 100%;
  overflow: hidden;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 6px;
  background: rgba(16, 19, 20, 0.9);
  backdrop-filter: blur(18px);
  box-shadow: 0 18px 48px rgba(0, 0, 0, 0.38);
}

.roleplay-head {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 16px;
  align-items: start;
  padding: 16px 18px 12px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.node-copy { min-width: 0; display: grid; gap: 3px; }
.node-copy strong { overflow: hidden; color: #f4f3ed; font-size: 15px; text-overflow: ellipsis; white-space: nowrap; }
.node-copy small { color: #b9bfbb; font-size: 12px; line-height: 1.45; }
.node-kicker { color: #d8b969; font-size: 10px; font-weight: 800; text-transform: uppercase; }
.turn-count { display: grid; gap: 2px; color: #9da39f; font-size: 10px; text-align: right; }
.turn-count strong { color: #f0d78e; font-size: 14px; }

.score-strip {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
  gap: 12px;
  padding: 10px 18px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(34, 40, 41, 0.5);
}

.score-item { min-width: 0; display: grid; gap: 5px; }
.score-label { display: flex; gap: 8px; justify-content: space-between; color: #b9bfbb; font-size: 10px; }
.score-label span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.score-label strong { color: #f4f3ed; font-variant-numeric: tabular-nums; }
.score-track { height: 3px; overflow: hidden; background: #414b4c; }
.score-track span { display: block; height: 100%; background: #d8b969; transition: width 220ms ease; }
.relationship-track span { background: #78b7a4; }

.roleplay-transcript {
  min-height: 0;
  overflow-y: auto;
  overscroll-behavior: contain;
  padding: 16px 18px 24px;
  scrollbar-color: #414b4c transparent;
}

.roleplay-degraded {
  padding: 7px 10px;
  border-left: 2px solid #d8b969;
  background: rgba(216, 185, 105, 0.1);
  color: #d9d5c5;
  font-size: 11px;
  line-height: 1.4;
}

.narration-entry {
  margin: 6px 0 18px;
  padding-left: 12px;
  border-left: 2px solid #6f8580;
}
.narration-entry span { color: #9da39f; font-size: 10px; font-weight: 800; text-transform: uppercase; }
.narration-entry p { margin: 4px 0 0; color: #c9ccc7; font-size: 12px; line-height: 1.65; }

.turn-entry { width: min(84%, 680px); margin: 0 0 14px; }
.turn-entry.player { margin-left: auto; padding: 10px 12px; border-radius: 6px; background: #303839; }
.turn-entry.character { margin-right: auto; }
.turn-entry.pending { opacity: 0.78; }
.turn-speaker { margin-bottom: 4px; color: #d8b969; font-size: 10px; font-weight: 800; }
.player .turn-speaker { color: #a8c6bc; }
.turn-entry p { margin: 0; color: #f4f3ed; font-size: 14px; line-height: 1.65; white-space: pre-wrap; overflow-wrap: anywhere; }
.turn-entry small { display: block; margin-top: 5px; color: #7f8984; font-size: 9px; line-height: 1.4; }
.thinking-line { display: flex; gap: 7px; align-items: center; color: #9da39f; font-size: 12px; }
.thinking-line svg, .send-button svg:first-child { animation: spin 1s linear infinite; }

.roleplay-composer {
  padding: 10px 12px 12px;
  border-top: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(23, 27, 28, 0.96);
}
.composer-row { display: grid; grid-template-columns: minmax(0, 1fr) 42px; gap: 8px; align-items: stretch; }
.composer-row textarea {
  width: 100%;
  min-height: 54px;
  max-height: 130px;
  resize: vertical;
  border: 1px solid rgba(235, 231, 214, 0.2);
  border-radius: 5px;
  outline: 0;
  background: #222829;
  color: #f4f3ed;
  font: inherit;
  font-size: 13px;
  line-height: 1.5;
  padding: 9px 10px;
}
.composer-row textarea:focus { border-color: #d8b969; }
.composer-row textarea::placeholder { color: #7f8984; }
.send-button {
  display: grid;
  place-items: center;
  min-width: 42px;
  border: 1px solid #d8b969;
  border-radius: 5px;
  background: #d8b969;
  color: #101314;
  cursor: pointer;
}
.send-button:disabled { cursor: not-allowed; opacity: 0.45; }
.roleplay-error { margin-bottom: 8px; font-size: 10px; line-height: 1.4; }
.roleplay-error { color: #f1a7a7; }

.roleplay-ending {
  display: grid;
  gap: 6px;
  padding: 18px;
  border-top: 1px solid rgba(255, 255, 255, 0.1);
  background: #222829;
}
.roleplay-ending span { color: #9da39f; font-size: 10px; font-weight: 800; text-transform: uppercase; }
.roleplay-ending strong { color: #f0d78e; font-size: 18px; }
.roleplay-ending p { margin: 0; color: #c9ccc7; font-size: 12px; line-height: 1.55; }
.ending-actions { display: flex; justify-content: center; gap: 8px; }
.continue-button { display: inline-flex; align-items: center; gap: 7px; min-height: 34px; border: 1px solid #d8b969; background: #d8b969; color: #151716; font: inherit; font-weight: 800; cursor: pointer; }
.restart-button { justify-self: start; display: inline-flex; gap: 7px; align-items: center; margin-top: 6px; border: 1px solid #6f8580; border-radius: 5px; background: transparent; color: #f4f3ed; cursor: pointer; padding: 7px 10px; }

@keyframes spin { to { transform: rotate(360deg); } }

@media (max-width: 720px) {
  .roleplay-head { padding: 12px; }
  .node-copy small { display: -webkit-box; overflow: hidden; -webkit-box-orient: vertical; -webkit-line-clamp: 2; }
  .score-strip { gap: 8px; padding: 9px 12px; }
  .score-label { display: grid; gap: 1px; }
  .roleplay-transcript { padding: 12px; }
  .turn-entry { width: 92%; }
}
</style>
