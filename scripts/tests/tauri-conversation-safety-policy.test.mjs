import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'

import { collectTauriConversationSafetyEvidence } from '../lib/tauri-packaging/conversation-safety-policy.mjs'

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..')
const rustDirectory = path.join(repositoryRoot, 'rust-engine')
const tauriAppDirectory = path.join(rustDirectory, 'crates', 'tauri-app')
const boundaries = {
  rustDirectory,
  tauriAppDirectory,
}

test('checked-in single and group conversation safety contracts return passing evidence', async () => {
  const evidence = await collectTauriConversationSafetyEvidence(boundaries)
  assert.deepEqual(evidence.issues, [])
  assert.deepEqual(evidence.requirementCounts, {
    chatSafety: 28,
    conversationQuality: 10,
    promptGuard: 18,
    fallbackScoring: 8,
    groupChat: 15,
  })
})

test('shared-model, multilingual, facade, and transcript drift stays independently actionable', async () => {
  const authoringDirectory = path.join(rustDirectory, 'crates', 'authoring', 'src')
  const commandDirectory = path.join(tauriAppDirectory, 'src', 'commands')
  const conversationPath = path.join(authoringDirectory, 'conversation_quality.rs')
  const promptGuardPath = path.join(authoringDirectory, 'prompt_guard.rs')
  const chatPath = path.join(commandDirectory, 'chat.rs')
  const promptGuardFacadePath = path.join(commandDirectory, 'prompt_guard.rs')
  const multiChatPath = path.join(commandDirectory, 'multi_chat.rs')
  const evidence = await collectTauriConversationSafetyEvidence({
    ...boundaries,
    async readTextFile(filePath, encoding) {
      const source = await readFile(filePath, encoding)
      const resolved = path.resolve(filePath)
      if (resolved === conversationPath) {
        return source
          .replaceAll('fallback_conversation_evaluation', 'local_conversation_evaluation')
          .replaceAll('谢谢', 'CHINESE_POSITIVE_SENTINEL_REMOVED')
      }
      if (resolved === promptGuardPath) {
        return source.replaceAll('忽略之前', 'CHINESE_GUARD_SENTINEL_REMOVED')
      }
      if (resolved === chatPath) {
        return `${source.replace(
          'pub use llm_authoring::conversation_quality::{',
          'pub use llm_authoring::drifted_quality::{',
        )}\npub struct ChatSafetyTrace {}\n`
      }
      if (resolved === promptGuardFacadePath) {
        return source.replace(
          'pub use llm_authoring::prompt_guard::*;',
          'pub use llm_authoring::drifted_prompt_guard::*;',
        )
      }
      if (resolved === multiChatPath) {
        return source.replaceAll('TRANSCRIPT_BEGIN', 'UNTRUSTED_DIALOGUE_BEGIN')
      }
      return source
    },
  })

  for (const issue of [
    'Headless conversation quality must centralize provider-independent fallback scoring',
    'Tauri chat commands must reuse the shared headless conversation quality models',
    'Tauri chat commands must not duplicate headless conversation quality models',
    'Prompt guard multilingual coverage must detect Chinese prompt-control instructions',
    'Tauri prompt guard commands must delegate to the shared headless authoring domain',
    'Fallback scoring multilingual coverage must score Chinese positive sentiment in local fallback',
    'Group chat runtime safety tracing must wrap group chat transcripts as untrusted dialogue data',
  ]) {
    assert(evidence.issues.includes(issue), issue)
  }
})

test('conversation safety policy requires both Rust filesystem boundaries', async () => {
  await assert.rejects(
    () => collectTauriConversationSafetyEvidence(),
    /requires rustDirectory/,
  )
  await assert.rejects(
    () => collectTauriConversationSafetyEvidence({ rustDirectory }),
    /requires tauriAppDirectory/,
  )
})
