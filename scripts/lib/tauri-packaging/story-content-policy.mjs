import { readFile as readFileFromDisk } from 'node:fs/promises'
import path from 'node:path'

export async function collectTauriStoryContentEvidence(options = {}) {
  const { rustDirectory, tauriAppDirectory } = resolveBoundaries(options)
  const readFile = options.readTextFile ?? readFileFromDisk
  const authoringDirectory = path.join(rustDirectory, 'crates', 'authoring', 'src')
  const tauriSourceDirectory = path.join(tauriAppDirectory, 'src')
  const commandDirectory = path.join(tauriSourceDirectory, 'commands')
  const tauriStoryEventsSource = await readFile(
    path.join(tauriSourceDirectory, 'story_events.rs'),
    'utf8',
  )
  const authoringStoryEventsSource = await readFile(
    path.join(authoringDirectory, 'story_events.rs'),
    'utf8',
  )
  const authoringDialogueValidationSource = await readFile(
    path.join(authoringDirectory, 'dialogue_validation.rs'),
    'utf8',
  )
  const authoringDialogueValidationTestsSource = await readFile(
    path.join(authoringDirectory, 'dialogue_validation', 'tests.rs'),
    'utf8',
  )
  const authoringRuntimeValidationSource = await readFile(
    path.join(authoringDirectory, 'runtime_validation.rs'),
    'utf8',
  )
  const tauriStoryProgressSource = await readFile(
    path.join(tauriSourceDirectory, 'story_progress.rs'),
    'utf8',
  )
  const tauriStoryAccessSource = await readFile(
    path.join(tauriSourceDirectory, 'story_access.rs'),
    'utf8',
  )
  const tauriContentReferencesSource = await readFile(
    path.join(tauriSourceDirectory, 'content_references.rs'),
    'utf8',
  )
  const tauriStoryEventCommandsSource = await readFile(
    path.join(commandDirectory, 'story_events.rs'),
    'utf8',
  )
  const tauriEndingCommandsSource = await readFile(
    path.join(commandDirectory, 'endings.rs'),
    'utf8',
  )
  const tauriDialogueCommandsSource = await readFile(
    path.join(commandDirectory, 'dialogue.rs'),
    'utf8',
  )
  const tauriKnowledgeCommandsSource = await readFile(
    path.join(commandDirectory, 'knowledge.rs'),
    'utf8',
  )
  const tauriScenesSource = await readFile(path.join(commandDirectory, 'scenes.rs'), 'utf8')
  const tauriChatSource = await readFile(path.join(commandDirectory, 'chat.rs'), 'utf8')
  const tauriQualitySuiteSource = await readFile(
    path.join(commandDirectory, 'quality_suite.rs'),
    'utf8',
  )
  const tauriWorkflowSource = await readFile(
    path.join(commandDirectory, 'workflow.rs'),
    'utf8',
  )
  const issues = []

  const storyCatalogRequirements = [
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
  ]
  const eventRuntimeRequirements = [
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
  ]
  const dialogueAuthoringRequirements = [
    [tauriDialogueCommandsSource, 'ensure_dialogue_access', 'enforce dialogue unlocks before playback'],
    [tauriDialogueCommandsSource, 'list_dialogues', 'expose dialogue metadata with access decisions'],
    [tauriDialogueCommandsSource, 'monogatari-dialogue-authoring-catalog/v1', 'version dialogue authoring catalog snapshots'],
    [tauriDialogueCommandsSource, 'dialogue_authoring_catalog_fingerprint', 'fingerprint complete dialogue catalogs for optimistic concurrency'],
    [authoringDialogueValidationSource, 'pub fn normalize_dialogue_script', 'own canonical Dialogue normalization in the headless authoring domain'],
    [authoringDialogueValidationSource, 'pub fn validate_dialogue_script', 'own Dialogue graph, bounds, character, and relationship validation in the headless authoring domain'],
    [authoringDialogueValidationSource, 'pub struct DialogueValidationResult', 'return structured transport-neutral Dialogue validation evidence'],
    [authoringDialogueValidationTestsSource, 'validation_reports_authoring_limits_beyond_runtime_topology', 'test Dialogue authoring limits independently of Tauri'],
    [authoringDialogueValidationTestsSource, 'validation_evidence_is_deterministic_and_bounded', 'bound and stabilize structured Dialogue validation evidence'],
    [authoringRuntimeValidationSource, 'validate_dialogue_script(&dialogue, character_ids)', 'apply shared Dialogue authoring rules to Agent runtime acceptance'],
    [authoringRuntimeValidationSource, 'dialogue_not_canonical', 'reject Agent Dialogue files that only pass after in-memory normalization'],
    [tauriDialogueCommandsSource, 'ensure_valid_dialogue_script', 'delegate desktop Dialogue validation to the headless authoring domain'],
    [tauriDialogueCommandsSource, 'stage_json_replacement', 'atomically stage dialogue saves'],
    [tauriDialogueCommandsSource, 'dialogue_references', 'protect event- and ending-referenced dialogues from deletion'],
    [tauriDialogueCommandsSource, 'replace_scripts(runtime_scripts)', 'hot-reload validated dialogue catalogs into runtime state'],
    [tauriDialogueCommandsSource, 'dialogue_save_is_atomic_rejects_stale_graphs_and_hot_reloads_runtime', 'test atomic dialogue save and hot reload behavior'],
    [tauriDialogueCommandsSource, 'dialogue_create_rejects_portable_case_aliases_without_replacing_script', 'test dialogue create cannot replace a Windows path alias'],
    [tauriDialogueCommandsSource, 'dialogue_delete_requires_event_and_ending_references_to_be_removed', 'test dialogue deletion reference protection'],
    [tauriDialogueCommandsSource, 'preview_dialogue', 'support author dialogue preview without player gates'],
  ]
  const knowledgeAuthoringRequirements = [
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
  ]
  const sceneAuthoringRequirements = [
    [tauriScenesSource, 'enter_story_scene', 'separate gated Story Mode entry from author scene selection'],
    [tauriScenesSource, 'monogatari-scene-authoring-catalog/v1', 'version scene authoring catalog snapshots'],
    [tauriScenesSource, 'scene_authoring_catalog_fingerprint', 'fingerprint authored and inferred scenes for optimistic concurrency'],
    [tauriScenesSource, 'stage_json_replacement', 'atomically stage scene metadata saves'],
    [tauriScenesSource, 'scene_references', 'protect referenced scene metadata from deletion'],
    [tauriScenesSource, 'scene_save_promotes_inferred_assets_and_rejects_stale_or_invalid_updates', 'test inferred scene promotion and stale-write rejection'],
    [tauriScenesSource, 'scene_create_rejects_portable_case_aliases_without_replacing_metadata', 'test scene create cannot replace a Windows path alias'],
    [tauriScenesSource, 'scene_delete_requires_event_ending_and_workflow_references_to_be_removed', 'test scene deletion reference protection'],
  ]
  const endingAuthoringRequirements = [
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
  ]
  const crossRuntimeRequirements = [
    [tauriWorkflowSource, 'apply_story_event_definition', 'apply real workflow trigger effects through the shared executor'],
    [tauriQualitySuiteSource, 'event_catalog: &StoryEventCatalog', 'run quality scenarios against project event rules'],
    [tauriStoryProgressSource, 'monogatari-story-progress/v1', 'version persistent story progress'],
    [tauriStoryProgressSource, 'monogatari-story-event-application/v1', 'version event application audit reports'],
    [tauriStoryProgressSource, 'validate_and_normalize', 'validate restored story progress before activation'],
    [tauriStoryProgressSource, 'nonrepeatable_event_applies_once_per_character_scope', 'test idempotent nonrepeatable effects'],
    [tauriStoryProgressSource, 'repeatable_event_increments_count_but_unlocks_idempotently', 'test repeatable event accounting'],
  ]
  const requirementGroups = {
    storyCatalog: storyCatalogRequirements,
    eventRuntime: eventRuntimeRequirements,
    dialogueAuthoring: dialogueAuthoringRequirements,
    knowledgeAuthoring: knowledgeAuthoringRequirements,
    sceneAuthoring: sceneAuthoringRequirements,
    endingAuthoring: endingAuthoringRequirements,
    crossRuntime: crossRuntimeRequirements,
  }
  for (const requirements of Object.values(requirementGroups)) {
    appendRequirements(requirements, issues)
  }
  if (/fn (?:normalize_dialogue_script|validate_dialogue_script|validate_dialogue_text)\s*\(/.test(tauriDialogueCommandsSource)) {
    issues.push('Story content integration must keep Dialogue normalization and validation out of Tauri commands')
  }

  return {
    issues,
    requirementCounts: Object.fromEntries(
      Object.entries(requirementGroups).map(([name, requirements]) => [name, requirements.length]),
    ),
    structuralCheckCount: 1,
  }
}

function appendRequirements(requirements, issues) {
  for (const [source, needle, description] of requirements) {
    if (!source.includes(needle)) {
      issues.push(`Story content integration must ${description}`)
    }
  }
}

function resolveBoundaries(options) {
  const boundaries = {
    rustDirectory: options.rustDirectory,
    tauriAppDirectory: options.tauriAppDirectory,
  }
  for (const [name, value] of Object.entries(boundaries)) {
    if (typeof value !== 'string' || value.length === 0) {
      throw new Error(`Tauri Story Content policy requires ${name}.`)
    }
  }
  return boundaries
}
