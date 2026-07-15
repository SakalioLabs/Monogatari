import { readFile as readFileFromDisk } from 'node:fs/promises'
import path from 'node:path'

const chatSafetyTraceRequirements = [
  ['pub struct ChatSafetyTrace', 'define a serializable chat safety trace'],
  ['safety_trace: ChatSafetyTrace', 'return runtime guard evidence with non-streaming chat responses'],
  ['build_chat_safety_trace', 'centralize runtime chat guard evidence'],
  ['chat-safety-trace', 'emit runtime guard evidence for streaming chat responses'],
  ['response_guard_applied', 'report guarded character response evidence'],
  ['relationship_delta_blocked', 'report relationship side-channel containment evidence'],
  ['ChatSessionAuditReport', 'type restorable chat session audit reports'],
  ['get_chat_session_audit', 'return restorable chat safety and event audit state'],
  ['last_safety_trace', 'persist the latest runtime safety trace in chat sessions'],
  ['build_chat_session_audit_report', 'centralize restorable chat session audit reports'],
  ['input_wrapped_as_untrusted', 'prove player input is wrapped as untrusted dialogue data'],
  ['mind_contract_applied', 'prove the character mind contract was applied'],
  ['knowledge_context_pinned', 'prove creator-pinned knowledge context was applied'],
  ['pinned_knowledge_ref_count', 'report resolved pinned knowledge reference counts'],
  ['pinned_knowledge_ref_ids', 'report resolved pinned knowledge reference ids'],
  ['event_trigger_decisions', 'return explainable story event trigger decisions'],
  ['rule_fingerprint', 'return event rule fingerprints with story event decisions'],
  ['ConversationEvaluationReport', 'type atomic manual scoring reports'],
  ['evaluate_conversation_report', 'return scoring and event decisions through one command'],
  ['triggerable_events', 'return triggerable story events in scoring reports'],
  ['build_event_trigger_decisions', 'centralize explainable story event trigger decisions'],
  ['triggered_events_from_decisions', 'derive triggered story events from the decision audit'],
  ['chat-event-decisions', 'emit story event trigger decisions for streaming chat'],
  ['event_trigger_rule_fingerprints_are_stable_and_rule_bound', 'test event rule fingerprints are stable and rule-bound'],
  ['character_mind_contract_applied', 'emit runtime trace evidence for the character mind contract'],
  ['pinned_knowledge_context_applied', 'emit runtime trace evidence for pinned knowledge context'],
  ['streaming_generation_failed_message', 'replace partial streaming replies with a stable failure bubble'],
  ['streaming_failure_replacement_is_stable_and_generic', 'test streaming failure replacement text stays generic'],
]

const conversationQualityRequirements = [
  ['pub struct ChatMessage', 'own the stable conversation message model'],
  ['pub struct ChatSafetyTrace', 'own serializable runtime guard evidence'],
  ['pub struct ConversationEvaluation', 'own deterministic conversation score reports'],
  ['fallback_conversation_evaluation', 'centralize provider-independent fallback scoring'],
  ['build_chat_safety_trace', 'centralize runtime chat guard evidence'],
  ['build_event_trigger_decisions', 'centralize explainable story event decisions'],
  ['relationship_delta_for_player_message', 'centralize guarded relationship scoring'],
  ['fallback_scoring_is_multilingual_and_ignores_injection_boosts', 'test multilingual fallback and injection containment without Tauri'],
  ['safety_traces_deduplicate_pinned_knowledge_and_report_guards', 'test safety evidence without Tauri'],
  ['event_decisions_use_shared_scores_and_trigger_history', 'test event thresholds and trigger history without Tauri'],
]

const multilingualPromptGuardRequirements = [
  ['normalize_security_text', 'normalize security-sensitive text before guard checks'],
  ['normalize_security_char', 'centralize Unicode security character mapping'],
  ['\\u{FF01}', 'normalize fullwidth ASCII and punctuation before guard checks'],
  ['\\u{200B}', 'remove zero-width obfuscation before guard checks'],
  ['role:system', 'detect role markers after punctuation normalization'],
  ['role_tag_with_boundary', 'detect attributed XML role-control tags without broad substring false positives'],
  ['role_code_fence_payload', 'detect Markdown role-code-fence control blocks'],
  ['prompt_control_block_start', 'omit explicit prompt-control block bodies after detecting their opening marker'],
  ['prompt_control_block_ends', 'resume prompt sanitization only after explicit prompt-control block closers'],
  ['strip_prefix("<!--")', 'strip HTML comment prompt-control prefixes before role-line checks'],
  ["matches!(ch, '>' | '!' | '/' | '-'", 'strip slash/star comment prompt-control prefixes before role-line checks'],
  ['role_heading_matches', 'detect punctuation-free role heading spoofing'],
  ['忽略之前', 'detect Chinese prompt-control instructions'],
  ['以前の指示を無視', 'detect Japanese prompt-control instructions'],
  ['이전 지시를 무시', 'detect Korean prompt-control instructions'],
  ['思维链', 'detect Chinese private-reasoning requests'],
  ['採点基準', 'detect Japanese scoring-rubric leaks'],
  ['채점 기준', 'detect Korean scoring-rubric leaks'],
]

const multilingualFallbackScoringRequirements = [
  ['prompt_guard::normalize_security_text', 'reuse guard normalization before local fallback scoring'],
  ['谢谢', 'score Chinese positive sentiment in local fallback'],
  ['ありがとう', 'score Japanese positive sentiment in local fallback'],
  ['고마워', 'score Korean positive sentiment in local fallback'],
  ['创作', 'score Chinese creative intent in local fallback'],
  ['物語', 'score Japanese creative intent in local fallback'],
  ['이야기', 'score Korean creative intent in local fallback'],
  ['trusted_scoring_texts', 'score only trusted normalized player messages'],
]

const groupChatSafetyTraceRequirements = [
  ['safety_trace: Option<chat::ChatSafetyTrace>', 'attach chat safety traces to group chat messages'],
  ['build_guarded_group_chat_prompt', 'centralize guarded group chat prompt construction'],
  ['group_chat_safety_trace', 'centralize group chat runtime guard evidence'],
  ['normalize_group_character_ids', 'normalize and validate group chat participant ids'],
  ['group_character_ids_are_trimmed_unique_and_minimum_size', 'test group chat participants are unique and sufficient'],
  ['Group chat message cannot be empty.', 'reject empty group chat messages at the command boundary'],
  ['Group chat session is not active.', 'reject inactive group chat sessions at the command boundary'],
  ['group_generation_failed_message', 'surface stable per-character group generation failures'],
  ['.filter(|message| message.role == "player" || message.role == "character")', 'exclude runtime system messages from future group prompts'],
  ['group_prompt_omits_runtime_failure_messages', 'test runtime group failure messages are not replayed as dialogue'],
  ['group_generation_failure_message_is_stable_and_generic', 'test group generation failure copy stays generic'],
  ['response_text.chars().count()', 'log group response length metadata instead of raw dialogue text'],
  ['chat::build_chat_safety_trace', 'reuse the single-character chat safety trace contract'],
  ['chat::relationship_delta_for_player_message', 'reuse relationship side-channel containment evidence'],
  ['TRANSCRIPT_BEGIN', 'wrap group chat transcripts as untrusted dialogue data'],
]

export async function collectTauriConversationSafetyEvidence(options = {}) {
  const {
    rustDirectory,
    tauriAppDirectory,
  } = resolveBoundaries(options)
  const readFile = options.readTextFile ?? readFileFromDisk
  const authoringDirectory = path.join(rustDirectory, 'crates', 'authoring', 'src')
  const commandDirectory = path.join(tauriAppDirectory, 'src', 'commands')
  const authoringConversationQualitySource = await readFile(
    path.join(authoringDirectory, 'conversation_quality.rs'),
    'utf8',
  )
  const authoringPromptGuardSource = await readFile(
    path.join(authoringDirectory, 'prompt_guard.rs'),
    'utf8',
  )
  const tauriChatSource = await readFile(path.join(commandDirectory, 'chat.rs'), 'utf8')
  const tauriPromptGuardFacadeSource = await readFile(
    path.join(commandDirectory, 'prompt_guard.rs'),
    'utf8',
  )
  const tauriMultiChatSource = await readFile(
    path.join(commandDirectory, 'multi_chat.rs'),
    'utf8',
  )
  const issues = []

  appendMissingRequirements(
    authoringConversationQualitySource,
    conversationQualityRequirements,
    'Headless conversation quality',
    issues,
  )
  if (!tauriChatSource.includes('pub use llm_authoring::conversation_quality::{')) {
    issues.push('Tauri chat commands must reuse the shared headless conversation quality models')
  }
  if (/pub\s+struct\s+(?:ChatSafetyTrace|ConversationEvaluation)\s*\{/.test(tauriChatSource)) {
    issues.push('Tauri chat commands must not duplicate headless conversation quality models')
  }

  appendMissingRequirements(
    `${authoringConversationQualitySource}\n${tauriChatSource}`,
    chatSafetyTraceRequirements,
    'Chat runtime safety tracing',
    issues,
  )

  if (!tauriPromptGuardFacadeSource.includes('pub use llm_authoring::prompt_guard::*;')) {
    issues.push('Tauri prompt guard commands must delegate to the shared headless authoring domain')
  }
  appendMissingRequirements(
    authoringPromptGuardSource,
    multilingualPromptGuardRequirements,
    'Prompt guard multilingual coverage',
    issues,
  )
  appendMissingRequirements(
    authoringConversationQualitySource,
    multilingualFallbackScoringRequirements,
    'Fallback scoring multilingual coverage',
    issues,
  )
  appendMissingRequirements(
    tauriMultiChatSource,
    groupChatSafetyTraceRequirements,
    'Group chat runtime safety tracing',
    issues,
  )

  return {
    issues,
    requirementCounts: {
      chatSafety: chatSafetyTraceRequirements.length,
      conversationQuality: conversationQualityRequirements.length,
      promptGuard: multilingualPromptGuardRequirements.length,
      fallbackScoring: multilingualFallbackScoringRequirements.length,
      groupChat: groupChatSafetyTraceRequirements.length,
    },
  }
}

function appendMissingRequirements(source, requirements, label, issues) {
  for (const [needle, description] of requirements) {
    if (!source.includes(needle)) issues.push(`${label} must ${description}`)
  }
}

function resolveBoundaries(options) {
  const boundaries = {
    rustDirectory: options.rustDirectory,
    tauriAppDirectory: options.tauriAppDirectory,
  }
  for (const [name, value] of Object.entries(boundaries)) {
    if (typeof value !== 'string' || value.length === 0) {
      throw new Error(`Tauri conversation safety policy requires ${name}.`)
    }
  }
  return boundaries
}
