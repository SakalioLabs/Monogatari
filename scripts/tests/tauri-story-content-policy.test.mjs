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
    dialogueAuthoring: 19,
    knowledgeAuthoring: 33,
    sceneAuthoring: 8,
    endingAuthoring: 13,
    crossRuntime: 7,
  })
  assert.equal(evidence.structuralCheckCount, 2)
})

test('catalog, runtime, authoring surface, and progress drift stays independently actionable', async () => {
  const paths = {
    facade: path.join(tauriSourceDirectory, 'story_events.rs'),
    catalog: path.join(rustDirectory, 'crates', 'authoring', 'src', 'story_events.rs'),
    dialogueValidation: path.join(rustDirectory, 'crates', 'authoring', 'src', 'dialogue_validation.rs'),
    dialogueValidationTests: path.join(rustDirectory, 'crates', 'authoring', 'src', 'dialogue_validation', 'tests.rs'),
    knowledgeDocuments: path.join(rustDirectory, 'crates', 'authoring', 'src', 'knowledge_documents.rs'),
    knowledgeDocumentsTests: path.join(rustDirectory, 'crates', 'authoring', 'src', 'knowledge_documents', 'tests.rs'),
    knowledgeValidation: path.join(rustDirectory, 'crates', 'authoring', 'src', 'knowledge_validation.rs'),
    knowledgeValidationTests: path.join(rustDirectory, 'crates', 'authoring', 'src', 'knowledge_validation', 'tests.rs'),
    runtimeValidation: path.join(rustDirectory, 'crates', 'authoring', 'src', 'runtime_validation.rs'),
    runtimeValidationTests: path.join(rustDirectory, 'crates', 'authoring', 'src', 'runtime_validation', 'tests.rs'),
    gameKnowledgeEntry: path.join(rustDirectory, 'crates', 'game', 'src', 'knowledge', 'knowledge_entry.rs'),
    gameKnowledgeBase: path.join(rustDirectory, 'crates', 'game', 'src', 'knowledge', 'knowledge_base.rs'),
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
      if (resolved === paths.dialogueValidation) {
        return source
          .replaceAll('pub fn normalize_dialogue_script', 'pub fn normalize_desktop_dialogue_script')
          .replaceAll('pub fn validate_dialogue_script', 'pub fn validate_desktop_dialogue_script')
      }
      if (resolved === paths.dialogueValidationTests) {
        return source
          .replaceAll(
            'validation_reports_authoring_limits_beyond_runtime_topology',
            'validation_only_reports_runtime_topology',
          )
          .replaceAll(
            'validation_evidence_is_deterministic_and_bounded',
            'validation_evidence_is_unbounded',
          )
      }
      if (resolved === paths.knowledgeDocuments) {
        return source
          .replaceAll('pub fn load_knowledge_documents', 'pub fn load_desktop_knowledge_documents')
          .replaceAll('knowledge_unknown_field', 'accepted_unknown_knowledge_field')
      }
      if (resolved === paths.knowledgeDocumentsTests) {
        return source
          .replaceAll(
            'loader_supports_single_and_array_documents_with_legacy_relations',
            'loader_ignores_legacy_relations',
          )
          .replaceAll(
            'loader_returns_structured_authoring_validation_evidence',
            'loader_returns_string_only_validation_evidence',
          )
      }
      if (resolved === paths.knowledgeValidation) {
        return source
          .replaceAll('pub fn normalize_knowledge_entry', 'pub fn normalize_desktop_knowledge_entry')
          .replaceAll('pub fn validate_knowledge_catalog', 'pub fn validate_desktop_knowledge_catalog')
      }
      if (resolved === paths.knowledgeValidationTests) {
        return source
          .replaceAll(
            'validation_reports_authoring_rules_beyond_runtime_deserialization',
            'validation_only_reports_deserialization_failures',
          )
          .replaceAll(
            'validation_evidence_is_deterministic_and_bounded',
            'validation_evidence_is_unbounded',
          )
      }
      if (resolved === paths.runtimeValidation) {
        return source
          .replaceAll(
            'validate_dialogue_script(&dialogue, character_ids)',
            'validate_runtime_dialogue_script(&dialogue, character_ids)',
          )
          .replaceAll('dialogue_not_canonical', 'dialogue_normalization_ignored')
          .replaceAll(
            'load_knowledge_documents(project_root, directory)',
            'load_runtime_knowledge_documents(project_root, directory)',
          )
      }
      if (resolved === paths.runtimeValidationTests) {
        return source.replaceAll(
          'rejects_knowledge_authoring_rules_that_runtime_deserialization_accepts',
          'accepts_knowledge_authoring_rules_after_deserialization',
        )
      }
      if (resolved === paths.gameKnowledgeEntry) {
        return source
          .replaceAll('alias = "relatedEntries"', 'alias = "legacyRelationsIgnored"')
          .replaceAll('_ => KnowledgeCategory::Other(normalized)', '_ => KnowledgeCategory::Lore')
      }
      if (resolved === paths.gameKnowledgeBase) {
        return source
          .replaceAll('.total_cmp(&left.0)', '.partial_cmp(&left.0).unwrap()')
          .replaceAll(
            'entries.sort_by(|left, right| left.id.cmp(&right.id))',
            'entries.reverse()',
          )
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
        return `${source
          .replaceAll('dialogue_authoring_catalog_fingerprint', 'dialogue_catalog_revision')
          .replaceAll('ensure_valid_dialogue_script', 'validate_desktop_dialogue_script')}
fn validate_dialogue_script() {}`
      }
      if (resolved === paths.knowledge) {
        return `${source
          .replaceAll('knowledge_catalog_fingerprint', 'knowledge_catalog_revision')
          .replaceAll('ensure_valid_knowledge_catalog', 'validate_desktop_knowledge_catalog')}
fn validate_knowledge_catalog() {}`
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
    'Story content integration must own canonical Dialogue normalization in the headless authoring domain',
    'Story content integration must own Dialogue graph, bounds, character, and relationship validation in the headless authoring domain',
    'Story content integration must test Dialogue authoring limits independently of Tauri',
    'Story content integration must bound and stabilize structured Dialogue validation evidence',
    'Story content integration must apply shared Dialogue authoring rules to Agent runtime acceptance',
    'Story content integration must reject Agent Dialogue files that only pass after in-memory normalization',
    'Story content integration must delegate desktop Dialogue validation to the headless authoring domain',
    'Story content integration must keep Dialogue normalization and validation out of Tauri commands',
    'Story content integration must fingerprint knowledge catalogs for optimistic concurrency',
    'Story content integration must own canonical Knowledge normalization in the headless authoring domain',
    'Story content integration must own Knowledge field, bound, duplicate, and relation validation in the headless authoring domain',
    'Story content integration must test Knowledge authoring limits independently of Tauri',
    'Story content integration must stabilize and bound Knowledge validation evidence',
    'Story content integration must own bounded Knowledge document loading in the headless authoring domain',
    'Story content integration must reject unknown Knowledge entry fields',
    'Story content integration must test Knowledge shapes, legacy relations, and category fidelity independently',
    'Story content integration must test structured Knowledge loader rejection evidence',
    'Story content integration must apply shared Knowledge loading and authoring rules to Agent runtime acceptance',
    'Story content integration must test Agent Knowledge rejection beyond deserialization',
    'Story content integration must preserve legacy Knowledge relation fields in the real runtime model',
    'Story content integration must preserve creator-defined normalized Knowledge categories',
    'Story content integration must keep Knowledge search ordering total and panic-free',
    'Story content integration must return deterministic Knowledge entry order',
    'Story content integration must delegate desktop Knowledge saves to shared catalog validation',
    'Story content integration must keep Knowledge normalization, validation, and loading out of Tauri commands',
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
