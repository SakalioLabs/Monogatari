import {
  generateAuthoringApiChat,
  loadAuthoringApiRuntime,
  type AuthoringApiRuntime,
} from './authoringInference'
import type { KnowledgeEntryDefinition } from './knowledgeContent'
import { sanitizeWebNpcReply } from './npcConversation'
import {
  applyBrowserSceneRoleplayTurn,
  buildBrowserRoleplayEvaluatorMessages,
  buildBrowserRoleplayNpcMessages,
  containedBrowserRoleplayEvaluation,
  evaluateBrowserRoleplayFallback,
  parseBrowserRoleplayEvaluation,
  reconcileBrowserRoleplayEvaluation,
  type SceneRoleplaySnapshot,
  type SceneRoleplayTurnResponse,
} from './sceneRoleplay'
import {
  analyzeRoleplayPlayerInput,
  composeRoleplayGenerationRecovery,
  composeRoleplayIntrusionResponse,
  guardRoleplayNpcResponse,
} from './sceneRoleplaySafety'
import type { StoryCharacterInfo } from './storyContent'
import {
  detectWebGpuSupport,
  generateWebGpuChat,
  type WebGpuChatMessage,
  type WebGpuGenerationOptions,
  type WebGpuSupport,
} from './webgpuInference'

type ChatGenerator = (
  messages: WebGpuChatMessage[],
  options?: WebGpuGenerationOptions,
) => Promise<string>

export interface BrowserRoleplayTurnDependencies {
  loadApiRuntime: () => Promise<AuthoringApiRuntime | null>
  generateApiChat: ChatGenerator
  detectWebGpuSupport: () => WebGpuSupport
  generateWebGpuChat: ChatGenerator
}

export interface BrowserRoleplayTurnRequest {
  snapshot: SceneRoleplaySnapshot
  character: StoryCharacterInfo
  locale: string
  knowledgeEntries: KnowledgeEntryDefinition[]
  playerMessage: string
  apiRuntime: AuthoringApiRuntime | null
  onPhase?: (phase: 'npc' | 'evaluation') => void
  onNpcProgress?: (content: string) => void
}

export interface BrowserRoleplayTurnResult {
  response: SceneRoleplayTurnResponse
  apiRuntime: AuthoringApiRuntime | null
}

const defaultDependencies: BrowserRoleplayTurnDependencies = {
  loadApiRuntime: loadAuthoringApiRuntime,
  generateApiChat: generateAuthoringApiChat,
  detectWebGpuSupport,
  generateWebGpuChat,
}

export async function executeBrowserRoleplayTurn(
  request: BrowserRoleplayTurnRequest,
  dependencies: BrowserRoleplayTurnDependencies = defaultDependencies,
): Promise<BrowserRoleplayTurnResult> {
  const {
    snapshot,
    character,
    locale,
    knowledgeEntries,
    playerMessage,
    onNpcProgress,
  } = request
  const currentNode = snapshot.current_node
  const inputSafety = analyzeRoleplayPlayerInput(playerMessage)
  let apiRuntime = request.apiRuntime
  let npcResponse: string
  let evaluation
  let evaluationSource: string

  if (inputSafety.intrusion_detected) {
    npcResponse = composeRoleplayIntrusionResponse(currentNode, playerMessage)
    evaluation = containedBrowserRoleplayEvaluation(currentNode)
    evaluationSource = 'contained_intrusion'
  } else {
    request.onPhase?.('npc')
    apiRuntime ||= await dependencies.loadApiRuntime()
    let generateChat = apiRuntime
      ? dependencies.generateApiChat
      : dependencies.generateWebGpuChat
    if (!apiRuntime) {
      const support = dependencies.detectWebGpuSupport()
      if (!support.available) throw new Error('WebGPU is unavailable in this browser.')
    }

    let rawReply = ''
    const npcMessages = buildBrowserRoleplayNpcMessages(
      snapshot.definition,
      snapshot.session,
      character,
      locale,
      knowledgeEntries,
      playerMessage,
    )
    const npcOptions: WebGpuGenerationOptions = {
      maxNewTokens: snapshot.definition.inference.npc_max_tokens,
      maxContextCharacters: snapshot.definition.inference.max_context_characters,
      recoveryMaxContextCharacters: Math.min(
        3_000,
        snapshot.definition.inference.max_context_characters,
      ),
      onReset() {
        rawReply = ''
      },
      onChunk(chunk) {
        rawReply += chunk
      },
    }
    let npcCandidate: string | null = null
    try {
      const generated = await generateChat(npcMessages, npcOptions)
      npcCandidate = sanitizeWebNpcReply(rawReply || generated)
    } catch {
      rawReply = ''
      if (apiRuntime) {
        apiRuntime = null
        const support = dependencies.detectWebGpuSupport()
        if (support.available) {
          generateChat = dependencies.generateWebGpuChat
          try {
            const generated = await generateChat(npcMessages, npcOptions)
            npcCandidate = sanitizeWebNpcReply(rawReply || generated)
          } catch {
            rawReply = ''
          }
        }
      }
    }

    if (npcCandidate === null) {
      npcResponse = composeRoleplayGenerationRecovery(
        currentNode,
        playerMessage,
        snapshot.session.node_turns + 1,
      )
      evaluation = evaluateBrowserRoleplayFallback(currentNode, playerMessage)
      evaluationSource = 'authored_fallback_npc_inference_error'
    } else {
      const guardedNpc = guardRoleplayNpcResponse(currentNode, inputSafety, npcCandidate, {
        player_message: playerMessage,
        node_turn: snapshot.session.node_turns + 1,
      })
      npcResponse = guardedNpc.response
      if (guardedNpc.state_contained) {
        evaluation = evaluateBrowserRoleplayFallback(currentNode, playerMessage)
        evaluationSource = 'authored_fallback_npc_output'
      } else {
        request.onPhase?.('evaluation')
        evaluationSource = apiRuntime ? 'authoring_api_model' : 'browser_model'
        try {
          const evaluatorOutput = await generateChat(
            buildBrowserRoleplayEvaluatorMessages(
              snapshot.definition,
              snapshot.session,
              playerMessage,
              npcResponse,
              character.id,
            ),
            {
              maxNewTokens: snapshot.definition.inference.evaluator_max_tokens,
              temperature: 0,
              maxContextCharacters: snapshot.definition.inference.max_context_characters,
              recoveryMaxContextCharacters: Math.min(
                3_000,
                snapshot.definition.inference.max_context_characters,
              ),
            },
          )
          evaluation = parseBrowserRoleplayEvaluation(evaluatorOutput)
          const reconciled = reconcileBrowserRoleplayEvaluation(
            currentNode,
            playerMessage,
            evaluation,
          )
          evaluation = reconciled.evaluation
          if (reconciled.changed) {
            evaluationSource = apiRuntime
              ? 'authoring_api_model_reconciled'
              : 'browser_model_reconciled'
          }
        } catch {
          evaluationSource = 'authored_fallback_evaluator_error'
          evaluation = evaluateBrowserRoleplayFallback(currentNode, playerMessage)
        }
      }
    }
  }

  // Only guarded, scene-valid prose may cross the use-case presentation boundary.
  onNpcProgress?.(npcResponse)

  let applied
  try {
    applied = applyBrowserSceneRoleplayTurn(
      snapshot.definition,
      snapshot.session,
      {
        player_message: playerMessage,
        speaker_id: character.id,
        npc_response: npcResponse,
        evaluation,
      },
    )
  } catch {
    evaluationSource = inputSafety.intrusion_detected
      ? 'contained_intrusion'
      : 'authored_fallback_invalid_evaluation'
    evaluation = inputSafety.intrusion_detected
      ? containedBrowserRoleplayEvaluation(currentNode)
      : evaluateBrowserRoleplayFallback(currentNode, playerMessage)
    applied = applyBrowserSceneRoleplayTurn(
      snapshot.definition,
      snapshot.session,
      {
        player_message: playerMessage,
        speaker_id: character.id,
        npc_response: npcResponse,
        evaluation,
      },
    )
  }

  return {
    response: { ...applied.response, evaluation, evaluation_source: evaluationSource },
    apiRuntime,
  }
}
