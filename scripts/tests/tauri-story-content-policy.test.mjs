import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'

import { collectTauriStoryContentEvidence } from '../lib/tauri-packaging/story-content-policy.mjs'

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..')
const rustDirectory = path.join(repositoryRoot, 'rust-engine')
const tauriAppDirectory = path.join(rustDirectory, 'crates', 'tauri-app')
const tauriSourceDirectory = path.join(tauriAppDirectory, 'src')
const commandDirectory = path.join(tauriSourceDirectory, 'commands')
const boundaries = { rustDirectory, tauriAppDirectory }

test('checked-in Story Content authoring and runtime contracts return passing evidence', async () => {
  const evidence = await collectTauriStoryContentEvidence(boundaries)
  assert.deepEqual(evidence.issues, [])
  assert.deepEqual(evidence.requirementCounts, {
    storyCatalog: 21,
    eventRuntime: 20,
    dialogueAuthoring: 12,
    knowledgeAuthoring: 12,
    sceneAuthoring: 8,
    endingAuthoring: 13,
    crossRuntime: 7,
  })
})

test('catalog, runtime, authoring surface, and progress drift stays independently actionable', async () => {
  const paths = {
    facade: path.join(tauriSourceDirectory, 'story_events.rs'),
    catalog: path.join(rustDirectory, 'crates', 'authoring', 'src', 'story_events.rs'),
    progress: path.join(tauriSourceDirectory, 'story_progress.rs'),
    access: path.join(tauriSourceDirectory, 'story_access.rs'),
    storyCommands: path.join(commandDirectory, 'story_events.rs'),
    chat: path.join(commandDirectory, 'chat.rs'),
    workflow: path.join(commandDirectory, 'workflow.rs'),
    quality: path.join(commandDirectory, 'quality_suite.rs'),
    dialogue: path.join(commandDirectory, 'dialogue.rs'),
    knowledge: path.join(commandDirectory, 'knowledge.rs'),
    scenes: path.join(commandDirectory, 'scenes.rs'),
    endings: path.join(commandDirectory, 'endings.rs'),
  }
  const evidence = await collectTauriStoryContentEvidence({
    ...boundaries,
    async readTextFile(filePath, encoding) {
      const source = await readFile(filePath, encoding)
      const resolved = path.resolve(filePath)
      if (resolved === paths.facade) {
        return source.replaceAll(
          'pub use llm_authoring::story_events::*',
          'pub use llm_authoring::drifted_events::*',
        )
      }
      if (resolved === paths.catalog) {
        return source.replaceAll('monogatari-story-event-catalog/v1', 'story-event-catalog-drifted')
      }
      if (resolved === paths.storyCommands) {
        return source.replaceAll('save_story_event_catalog', 'store_story_event_catalog')
      }
      if (resolved === paths.chat) {
        return source.replaceAll('apply_triggered_event_decisions', 'discard_triggered_event_decisions')
      }
      if (resolved === paths.workflow) {
        return source.replaceAll('validate_workflow_with_catalog', 'validate_workflow_without_catalog')
      }
      if (resolved === paths.access) {
        return source.replaceAll('ensure_story_content_access', 'allow_story_content_access')
      }
      if (resolved === paths.dialogue) {
        return source.replaceAll('dialogue_authoring_catalog_fingerprint', 'dialogue_catalog_revision')
      }
      if (resolved === paths.knowledge) {
        return source.replaceAll('knowledge_catalog_fingerprint', 'knowledge_catalog_revision')
      }
      if (resolved === paths.scenes) {
        return source.replaceAll('scene_authoring_catalog_fingerprint', 'scene_catalog_revision')
      }
      if (resolved === paths.endings) {
        return source.replaceAll('story_ending_catalog_fingerprint', 'ending_catalog_revision')
      }
      if (resolved === paths.quality) {
        return source.replaceAll('event_catalog: &StoryEventCatalog', 'event_catalog: &DriftedCatalog')
      }
      if (resolved === paths.progress) {
        return source.replaceAll('monogatari-story-progress/v1', 'story-progress-drifted')
      }
      return source
    },
  })

  for (const issue of [
    'Story content integration must keep Tauri as a thin Story Event compatibility facade',
    'Story content integration must version project story event catalogs',
    'Story content integration must support validated optimistic-concurrency catalog saves',
    'Story content integration must apply triggered chat events through the shared executor',
    'Story content integration must validate workflow event references against the active catalog',
    'Story content integration must centralize scene, dialogue, and ending access enforcement',
    'Story content integration must fingerprint complete dialogue catalogs for optimistic concurrency',
    'Story content integration must fingerprint knowledge catalogs for optimistic concurrency',
    'Story content integration must fingerprint authored and inferred scenes for optimistic concurrency',
    'Story content integration must fingerprint ending catalogs for optimistic concurrency',
    'Story content integration must run quality scenarios against project event rules',
    'Story content integration must version persistent story progress',
  ]) {
    assert(evidence.issues.includes(issue), issue)
  }
})

test('Story Content policy requires both Rust filesystem boundaries', async () => {
  await assert.rejects(
    () => collectTauriStoryContentEvidence(),
    /requires rustDirectory/,
  )
  await assert.rejects(
    () => collectTauriStoryContentEvidence({ rustDirectory }),
    /requires tauriAppDirectory/,
  )
})
