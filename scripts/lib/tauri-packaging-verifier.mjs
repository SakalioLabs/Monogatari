import { readFile as readFileFromDisk } from 'node:fs/promises'
import path from 'node:path'

import { collectTauriBuildToolchainEvidence } from './tauri-packaging/build-toolchain-policy.mjs'
import { collectTauriCommandRegistrationEvidence } from './tauri-packaging/command-registration-policy.mjs'
import { collectTauriConversationSafetyEvidence } from './tauri-packaging/conversation-safety-policy.mjs'
import { collectTauriInstallationPolicyEvidence } from './tauri-packaging/installation-policy.mjs'
import { collectTauriPackagePolicyEvidence } from './tauri-packaging/package-policy.mjs'
import { collectTauriProjectPackageEvidence } from './tauri-packaging/project-package-policy.mjs'
import { collectTauriProjectRuntimeEvidence } from './tauri-packaging/project-runtime-policy.mjs'
import { collectTauriQualityWorkflowEvidence } from './tauri-packaging/quality-workflow-policy.mjs'

export function createTauriPackagingVerifier(options = {}) {
  const boundaries = resolveBoundaries(options)
  const logger = options.logger ?? console
  return async function verifyTauriPackagingConfig() {
    const evidence = await collectTauriPackagingEvidence({
      ...options,
      ...boundaries,
    })
    if (evidence.issues.length > 0) {
      throw new Error(`Tauri packaging config verification failed:\n${evidence.issues.join('\n')}`)
    }
    logger.log(
      `[release] Tauri packaging config OK (${evidence.targets.join(', ')} target(s), ${evidence.iconCount} icon(s))`,
    )
    return evidence
  }
}

export async function collectTauriPackagingEvidence(options = {}) {
  const {
    repositoryRoot: root,
    frontendDirectory: frontendDir,
    rustDirectory: rustDir,
    tauriAppDirectory: tauriAppDir,
  } = resolveBoundaries(options)
  const readFile = options.readTextFile ?? readFileFromDisk
  const packagePolicyEvidence = await collectTauriPackagePolicyEvidence({
    ...options,
    repositoryRoot: root,
    frontendDirectory: frontendDir,
    rustDirectory: rustDir,
    tauriAppDirectory: tauriAppDir,
  })
  const installationPolicyEvidence = await collectTauriInstallationPolicyEvidence({
    ...options,
    repositoryRoot: root,
    tauriAppDirectory: tauriAppDir,
  })
  const commandRegistrationEvidence = await collectTauriCommandRegistrationEvidence({
    ...options,
    tauriAppDirectory: tauriAppDir,
  })
  const conversationSafetyEvidence = await collectTauriConversationSafetyEvidence({
    ...options,
    rustDirectory: rustDir,
    tauriAppDirectory: tauriAppDir,
  })
  const qualityWorkflowEvidence = await collectTauriQualityWorkflowEvidence({
    ...options,
    rustDirectory: rustDir,
    tauriAppDirectory: tauriAppDir,
  })
  const buildToolchainEvidence = await collectTauriBuildToolchainEvidence({
    ...options,
    repositoryRoot: root,
    rustDirectory: rustDir,
    tauriAppDirectory: tauriAppDir,
  })
  const projectPackageEvidence = await collectTauriProjectPackageEvidence({
    ...options,
    rustDirectory: rustDir,
    tauriAppDirectory: tauriAppDir,
  })
  const projectRuntimeEvidence = await collectTauriProjectRuntimeEvidence({
    ...options,
    rustDirectory: rustDir,
    tauriAppDirectory: tauriAppDir,
  })
  const issues = [
    ...packagePolicyEvidence.issues,
    ...installationPolicyEvidence.issues,
    ...commandRegistrationEvidence.issues,
    ...conversationSafetyEvidence.issues,
    ...qualityWorkflowEvidence.issues,
    ...buildToolchainEvidence.issues,
    ...projectPackageEvidence.issues,
    ...projectRuntimeEvidence.issues,
  ]
  const tauriStoryEventsSource = await readFile(path.join(tauriAppDir, 'src', 'story_events.rs'), 'utf8')
  const authoringStoryEventsSource = await readFile(path.join(rustDir, 'crates', 'authoring', 'src', 'story_events.rs'), 'utf8')
  const tauriStoryProgressSource = await readFile(path.join(tauriAppDir, 'src', 'story_progress.rs'), 'utf8')
  const tauriStoryAccessSource = await readFile(path.join(tauriAppDir, 'src', 'story_access.rs'), 'utf8')
  const tauriContentReferencesSource = await readFile(path.join(tauriAppDir, 'src', 'content_references.rs'), 'utf8')
  const tauriStoryEventCommandsSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'story_events.rs'), 'utf8')
  const tauriEndingCommandsSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'endings.rs'), 'utf8')
  const tauriDialogueCommandsSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'dialogue.rs'), 'utf8')
  const tauriKnowledgeCommandsSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'knowledge.rs'), 'utf8')
  const tauriScenesSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'scenes.rs'), 'utf8')
  const tauriChatSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'chat.rs'), 'utf8')
  const tauriQualitySuiteSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'quality_suite.rs'), 'utf8')
  const tauriWorkflowSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'workflow.rs'), 'utf8')

  const storyEventCatalogRequirements = [
    [tauriStoryEventsSource, 'pub use llm_authoring::story_events::*', 'keep Tauri as a thin Story Event compatibility facade'],
    [authoringStoryEventsSource, 'monogatari-story-event-catalog/v1', 'version project story event catalogs'],
    [authoringStoryEventsSource, 'monogatari-event-trigger-rule/v1', 'preserve legacy rule fingerprint compatibility'],
    [authoringStoryEventsSource, 'monogatari-event-trigger-rule/v2', 'fingerprint character scope and repeat behavior'],
    [authoringStoryEventsSource, 'MAX_STORY_EVENT_FILE_BYTES', 'bound individual story event files'],
    [authoringStoryEventsSource, 'MAX_STORY_EVENT_CATALOG_BYTES', 'bound aggregate story event catalogs'],
    [authoringStoryEventsSource, 'metadata.file_type().is_symlink()', 'reject symlinked story event files'],
    [authoringStoryEventsSource, 'normalize_event_directory_reference', 'validate configured story event directories'],
    [authoringStoryEventsSource, 'Story event directory escapes the project root', 'enforce the project root boundary for event directories'],
    [authoringStoryEventsSource, 'Duplicate story event id', 'reject duplicate story event ids'],
    [authoringStoryEventsSource, 'validate_character_references', 'validate character-scoped event references'],
    [authoringStoryEventsSource, 'validate_content_references', 'validate typed Event content targets'],
    [authoringStoryEventsSource, 'pub enum StoryEventAction', 'define typed story event actions'],
    [authoringStoryEventsSource, 'normalize_story_event_actions', 'normalize typed and legacy event effects'],
    [authoringStoryEventsSource, 'MAX_EVENT_ACTIONS', 'bound event action lists'],
    [authoringStoryEventsSource, 'event_trigger_rule_fingerprint', 'centralize trigger rule fingerprints'],
    [authoringStoryEventsSource, 'checked_in_catalog_preserves_pinned_v1_rule_fingerprints', 'test pinned legacy rule fingerprints'],
    [authoringStoryEventsSource, 'checked_in_catalog_preserves_cross_runtime_catalog_fingerprint', 'pin the action-bound catalog fingerprint across Rust and release tooling'],
    [authoringStoryEventsSource, 'project_catalog_supports_character_scope_and_repeatable_rules', 'test creator-defined scope and repeat behavior'],
    [authoringStoryEventsSource, 'configured_event_directory_is_project_relative_and_enforced', 'test configured event directory containment'],
    [authoringStoryEventsSource, 'missing_directory_uses_compatibility_catalog_but_empty_directory_stays_empty', 'preserve old projects without forcing events into intentionally empty catalogs'],
    [tauriStoryEventCommandsSource, 'get_story_event_catalog', 'expose the active catalog to author tooling'],
    [tauriStoryEventCommandsSource, 'get_story_progress', 'expose persistent story progress to runtime tooling'],
    [tauriStoryEventCommandsSource, 'reload_story_event_catalog', 'support atomic author hot reloads'],
    [tauriStoryEventCommandsSource, 'save_story_event_catalog', 'support validated optimistic-concurrency catalog saves'],
    [tauriStoryEventCommandsSource, 'write_staged_event_document', 'stage event catalog replacement before activation'],
    [tauriStoryEventCommandsSource, 'rollback_staged_event_document', 'restore event catalogs after post-write validation failure'],
    [tauriStoryEventCommandsSource, 'visual_save_rejects_multi_document_catalogs', 'reject ambiguous visual flattening of multi-document catalogs'],
    [tauriStoryEventCommandsSource, 'apply_story_event_definition', 'centralize atomic story event effect application'],
    [tauriStoryEventCommandsSource, 'rejected_reload_leaves_active_catalog_unchanged', 'test failed reloads do not replace active rules'],
    [tauriChatSource, 'state.story_event_catalog.read().await.clone()', 'evaluate chat events from the active project catalog'],
    [tauriChatSource, 'apply_triggered_event_decisions', 'apply triggered chat events through the shared executor'],
    [tauriChatSource, 'chat-event-applications', 'emit applied event effects for streaming chat'],
    [tauriWorkflowSource, 'validate_workflow_with_catalog', 'validate workflow event references against the active catalog'],
    [tauriWorkflowSource, 'node_event_unknown', 'report unknown workflow story events'],
    [tauriWorkflowSource, 'event_catalog.decision_for', 'evaluate workflow trigger nodes from project rules'],
    [tauriWorkflowSource, 'ensure_story_content_access', 'enforce story scene gates during real workflow execution'],
    [tauriWorkflowSource, 'workflow_scene_change_enforces_event_unlocks_for_real_runs', 'test workflow scene gate enforcement'],
    [tauriStoryAccessSource, 'monogatari-story-content-access/v1', 'version event-derived content access snapshots'],
    [tauriStoryAccessSource, 'ensure_story_content_access', 'centralize scene, dialogue, and ending access enforcement'],
    [tauriStoryAccessSource, 'content_not_referenced_by_an_unlock_action_is_open', 'preserve access to legacy unreferenced content'],
    [tauriDialogueCommandsSource, 'ensure_dialogue_access', 'enforce dialogue unlocks before playback'],
    [tauriDialogueCommandsSource, 'list_dialogues', 'expose dialogue metadata with access decisions'],
    [tauriDialogueCommandsSource, 'monogatari-dialogue-authoring-catalog/v1', 'version dialogue authoring catalog snapshots'],
    [tauriDialogueCommandsSource, 'dialogue_authoring_catalog_fingerprint', 'fingerprint complete dialogue catalogs for optimistic concurrency'],
    [tauriDialogueCommandsSource, 'validate_dialogue_script', 'validate dialogue graph, character, script, and relationship references'],
    [tauriDialogueCommandsSource, 'stage_json_replacement', 'atomically stage dialogue saves'],
    [tauriDialogueCommandsSource, 'dialogue_references', 'protect event- and ending-referenced dialogues from deletion'],
    [tauriDialogueCommandsSource, 'replace_scripts(runtime_scripts)', 'hot-reload validated dialogue catalogs into runtime state'],
    [tauriDialogueCommandsSource, 'dialogue_save_is_atomic_rejects_stale_graphs_and_hot_reloads_runtime', 'test atomic dialogue save and hot reload behavior'],
    [tauriDialogueCommandsSource, 'dialogue_create_rejects_portable_case_aliases_without_replacing_script', 'test dialogue create cannot replace a Windows path alias'],
    [tauriDialogueCommandsSource, 'dialogue_delete_requires_event_and_ending_references_to_be_removed', 'test dialogue deletion reference protection'],
    [tauriDialogueCommandsSource, 'preview_dialogue', 'support author dialogue preview without player gates'],
    [tauriKnowledgeCommandsSource, 'get_knowledge_authoring_catalog', 'expose editable knowledge catalog snapshots'],
    [tauriKnowledgeCommandsSource, 'save_knowledge_entry_definition', 'support validated optimistic-concurrency knowledge saves'],
    [tauriKnowledgeCommandsSource, 'delete_knowledge_entry_definition', 'support knowledge entry deletion'],
    [tauriKnowledgeCommandsSource, 'knowledge_catalog_fingerprint', 'fingerprint knowledge catalogs for optimistic concurrency'],
    [tauriKnowledgeCommandsSource, 'stage_json_replacement', 'stage atomic knowledge document replacements'],
    [tauriKnowledgeCommandsSource, 'staged.rollback().await?', 'restore rejected knowledge document replacements'],
    [tauriKnowledgeCommandsSource, 'knowledge_references(&project_root', 'protect referenced knowledge entries from deletion'],
    [tauriKnowledgeCommandsSource, 'validate_knowledge_relations', 'validate related knowledge ids before catalog activation'],
    [tauriKnowledgeCommandsSource, 'authoring_loader_supports_single_and_array_documents', 'test single-entry and array knowledge documents'],
    [tauriKnowledgeCommandsSource, 'validation_rejects_non_portable_ids_and_out_of_range_importance', 'test knowledge authoring validation boundaries'],
    [tauriContentReferencesSource, 'pub fn knowledge_references', 'discover character-pinned knowledge references'],
    [tauriContentReferencesSource, 'knowledge_references_find_character_pins', 'test character-pinned knowledge reference discovery'],
    [tauriScenesSource, 'enter_story_scene', 'separate gated Story Mode entry from author scene selection'],
    [tauriScenesSource, 'monogatari-scene-authoring-catalog/v1', 'version scene authoring catalog snapshots'],
    [tauriScenesSource, 'scene_authoring_catalog_fingerprint', 'fingerprint authored and inferred scenes for optimistic concurrency'],
    [tauriScenesSource, 'stage_json_replacement', 'atomically stage scene metadata saves'],
    [tauriScenesSource, 'scene_references', 'protect referenced scene metadata from deletion'],
    [tauriScenesSource, 'scene_save_promotes_inferred_assets_and_rejects_stale_or_invalid_updates', 'test inferred scene promotion and stale-write rejection'],
    [tauriScenesSource, 'scene_create_rejects_portable_case_aliases_without_replacing_metadata', 'test scene create cannot replace a Windows path alias'],
    [tauriScenesSource, 'scene_delete_requires_event_ending_and_workflow_references_to_be_removed', 'test scene deletion reference protection'],
    [tauriEndingCommandsSource, 'monogatari-story-ending/v1', 'version story ending assets'],
    [tauriEndingCommandsSource, 'monogatari-story-ending-catalog/v1', 'version ending authoring catalog snapshots'],
    [tauriEndingCommandsSource, 'story_ending_catalog_fingerprint', 'fingerprint ending catalogs for optimistic concurrency'],
    [tauriEndingCommandsSource, 'validate_story_ending_references', 'cross-check ending scene and dialogue references before save'],
    [tauriEndingCommandsSource, 'stage_json_replacement', 'stage atomic ending replacements through the shared content transaction'],
    [tauriEndingCommandsSource, 'staged.rollback().await?', 'restore rejected ending replacements'],
    [tauriEndingCommandsSource, 'still unlocked by event(s)', 'protect event-referenced endings from deletion'],
    [tauriEndingCommandsSource, 'preview_story_ending_inner', 'support validated author preview without player gates'],
    [tauriEndingCommandsSource, 'start_story_ending_inner', 'validate and launch ending scene/dialogue pairs'],
    [tauriEndingCommandsSource, 'ending_launch_enforces_unlocks_then_starts_scene_and_dialogue', 'test complete ending gate and launch behavior'],
    [tauriEndingCommandsSource, 'ending_save_is_atomic_and_rejects_stale_or_invalid_updates', 'test atomic ending saves and stale-write rejection'],
    [tauriEndingCommandsSource, 'ending_create_rejects_portable_case_aliases_without_replacing_definition', 'test ending create cannot replace a Windows path alias'],
    [tauriEndingCommandsSource, 'ending_delete_requires_event_references_to_be_removed_first', 'test ending deletion reference protection'],
    [tauriWorkflowSource, 'apply_story_event_definition', 'apply real workflow trigger effects through the shared executor'],
    [tauriQualitySuiteSource, 'event_catalog: &StoryEventCatalog', 'run quality scenarios against project event rules'],
    [tauriStoryProgressSource, 'monogatari-story-progress/v1', 'version persistent story progress'],
    [tauriStoryProgressSource, 'monogatari-story-event-application/v1', 'version event application audit reports'],
    [tauriStoryProgressSource, 'validate_and_normalize', 'validate restored story progress before activation'],
    [tauriStoryProgressSource, 'nonrepeatable_event_applies_once_per_character_scope', 'test idempotent nonrepeatable effects'],
    [tauriStoryProgressSource, 'repeatable_event_increments_count_but_unlocks_idempotently', 'test repeatable event accounting'],
  ]
  for (const [source, needle, description] of storyEventCatalogRequirements) {
    if (!source.includes(needle)) {
      issues.push(`Story event catalog integration must ${description}`)
    }
  }

  return {
    issues,
    targets: packagePolicyEvidence.targets,
    iconCount: packagePolicyEvidence.iconCount,
  }
}

function resolveBoundaries(options) {
  const boundaries = {
    repositoryRoot: options.repositoryRoot,
    frontendDirectory: options.frontendDirectory,
    rustDirectory: options.rustDirectory,
    tauriAppDirectory: options.tauriAppDirectory,
  }
  for (const [name, value] of Object.entries(boundaries)) {
    if (typeof value !== 'string' || value.length === 0) {
      throw new Error(`Tauri packaging verifier requires ${name}.`)
    }
  }
  return boundaries
}
