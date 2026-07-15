import { readFile as readFileFromDisk } from 'node:fs/promises'
import path from 'node:path'

export async function collectTauriProjectRuntimeEvidence(options = {}) {
  const { rustDirectory, tauriAppDirectory } = resolveBoundaries(options)
  const readFile = options.readTextFile ?? readFileFromDisk
  const authoringDirectory = path.join(rustDirectory, 'crates', 'authoring', 'src')
  const gameDirectory = path.join(rustDirectory, 'crates', 'game', 'src')
  const tauriSourceDirectory = path.join(tauriAppDirectory, 'src')
  const commandDirectory = path.join(tauriSourceDirectory, 'commands')
  const tauriMainSource = await readFile(path.join(tauriSourceDirectory, 'main.rs'), 'utf8')
  const tauriStateSource = await readFile(path.join(tauriSourceDirectory, 'state.rs'), 'utf8')
  const authoringRuntimeValidationSource = await readFile(
    path.join(authoringDirectory, 'runtime_validation.rs'),
    'utf8',
  )
  const authoringFilesystemSource = await readFile(
    path.join(authoringDirectory, 'filesystem.rs'),
    'utf8',
  )
  const tauriContentReferencesSource = await readFile(
    path.join(tauriSourceDirectory, 'content_references.rs'),
    'utf8',
  )
  const tauriEngineSource = await readFile(path.join(commandDirectory, 'engine.rs'), 'utf8')
  const tauriDialogueCommandsSource = await readFile(
    path.join(commandDirectory, 'dialogue.rs'),
    'utf8',
  )
  const tauriEndingCommandsSource = await readFile(
    path.join(commandDirectory, 'endings.rs'),
    'utf8',
  )
  const tauriKnowledgeCommandsSource = await readFile(
    path.join(commandDirectory, 'knowledge.rs'),
    'utf8',
  )
  const tauriScenesSource = await readFile(path.join(commandDirectory, 'scenes.rs'), 'utf8')
  const tauriAnalyticsSource = await readFile(
    path.join(commandDirectory, 'analytics.rs'),
    'utf8',
  )
  const tauriCloudSyncSource = await readFile(
    path.join(commandDirectory, 'cloud_sync.rs'),
    'utf8',
  )
  const tauriTtsSource = await readFile(path.join(commandDirectory, 'tts.rs'), 'utf8')
  const gameCharacterSource = await readFile(
    path.join(gameDirectory, 'characters', 'character.rs'),
    'utf8',
  )
  const gameDialogueScriptSource = await readFile(
    path.join(gameDirectory, 'dialogue', 'dialogue_script.rs'),
    'utf8',
  )
  const gameDialogueNodeSource = await readFile(
    path.join(gameDirectory, 'dialogue', 'dialogue_node.rs'),
    'utf8',
  )
  const issues = []

  const runtimeRootRequirements = [
    [tauriMainSource, 'resource_dir()', 'resolve the Tauri resource directory during setup'],
    [tauriMainSource, 'discover_bundled_project_data_root', 'look for bundled data resources at startup'],
    [tauriMainSource, 'set_project_data_root(data_root)', 'bind discovered project data into AppState at startup'],
    [tauriStateSource, 'pub fn default_project_data_root()', 'centralize default project data-root discovery'],
    [tauriStateSource, 'pub fn discover_bundled_project_data_root', 'centralize bundled Tauri data-resource discovery'],
    [tauriStateSource, 'pub fn is_project_data_root', 'validate project data roots before binding them'],
    [tauriStateSource, 'AssetManager::new(&data_path)', 'rebind the asset manager when project roots change'],
    [tauriStateSource, 'SaveManager::new(data_path.join("saves"))', 'rebind the save manager when project roots change'],
    [tauriStateSource, 'story_event_catalog: Arc<RwLock<StoryEventCatalog>>', 'keep the active story event catalog project-scoped'],
    [tauriStateSource, 'story_progress: Arc<RwLock<StoryProgressState>>', 'keep persistent story progress project-scoped'],
    [tauriEngineSource, 'current_project_data_root().await', 'keep empty engine initialization paths on the active or discovered default root'],
    [tauriEngineSource, 'load_project_content(&path).await?', 'stage all project content before replacing active managers'],
    [authoringRuntimeValidationSource, 'StoryEventCatalog::load_from_project_root(project_root)', 'stage project story events during shared engine initialization'],
    [authoringRuntimeValidationSource, 'validate_character_references', 'validate character-scoped story events before activation'],
    [tauriEngineSource, 'core.story_events', 'activate the shared validated Story Event catalog'],
    [tauriEngineSource, 'let root_changed = state.set_project_data_root(path).await', 'rebind project managers after staged engine initialization'],
    [tauriEngineSource, 'project_content_loading_replaces_instead_of_merging_managers', 'test project reloads do not merge old content'],
    [tauriEngineSource, 'checked_in_project_data_loads_as_real_runtime_content', 'load both checked-in project roots through real runtime managers'],
    [tauriStateSource, 'reset_project_runtime_state', 'clear mutable chat, scene, and script state across project reloads'],
    [tauriStateSource, 'StoryProgressState::default()', 'clear story progress across project reloads'],
    [tauriStateSource, 'changing_project_root_clears_project_runtime_state', 'test project root changes clear runtime state'],
    [tauriStateSource, 'same_root_reload_can_explicitly_clear_project_runtime_state', 'test same-root project reloads clear runtime state'],
    [tauriStateSource, 'story_content_authoring_lock', 'serialize project content authoring mutations'],
    [authoringFilesystemSource, 'pub struct StagedContentMutation', 'share rollback-capable content mutations through the headless authoring crate'],
    [authoringFilesystemSource, 'stage_json_replacement', 'stage bounded atomic JSON replacements'],
    [authoringFilesystemSource, 'stage_json_deletion', 'stage rollback-capable JSON deletions'],
    [authoringFilesystemSource, 'replacements_commit_or_restore_the_previous_document', 'test shared content replacement rollback'],
    [authoringFilesystemSource, 'ensure_portable_replacement_target(target_path, label)?', 'reject portable filename case aliases before staging replacement files'],
    [authoringFilesystemSource, 'replacements_reject_portable_case_aliases_before_mutation', 'test case-alias rejection leaves existing content unchanged'],
    [tauriDialogueCommandsSource, 'use llm_authoring::filesystem', 'delegate dialogue mutations to the headless authoring crate'],
    [tauriEndingCommandsSource, 'use llm_authoring::filesystem', 'delegate ending mutations to the headless authoring crate'],
    [tauriKnowledgeCommandsSource, 'use llm_authoring::filesystem', 'delegate knowledge mutations to the headless authoring crate'],
    [tauriScenesSource, 'use llm_authoring::filesystem', 'delegate scene mutations to the headless authoring crate'],
    [tauriContentReferencesSource, 'scene_references', 'scan scene references before metadata deletion'],
    [tauriContentReferencesSource, 'dialogue_references', 'scan dialogue references before script deletion'],
    [tauriContentReferencesSource, 'workflow_scene_references', 'include workflow scene transitions in reference protection'],
    [tauriScenesSource, 'Ok(default_project_data_root())', 'scan scene assets from the discovered default root before explicit initialization'],
    [tauriAnalyticsSource, 'state.current_project_data_root().await', 'persist analytics under the active project root'],
    [tauriAnalyticsSource, 'project_root.join("analytics.json")', 'keep analytics files project-scoped'],
    [tauriAnalyticsSource, 'HashMap<PathBuf, Vec<AnalyticsEvent>>', 'keep in-memory analytics stores project-scoped'],
    [tauriCloudSyncSource, 'state.current_project_data_root().await', 'persist cloud sync manifests under the active project root'],
    [tauriCloudSyncSource, 'saves_dir(project_root).join(".sync_manifest.json")', 'keep sync manifests in the active project saves directory'],
    [tauriCloudSyncSource, 'analyze_sync_inventory', 'centralize local sync manifest status analysis'],
    [tauriCloudSyncSource, 'is_save_json', 'avoid counting the sync manifest itself as a save file'],
    [tauriCloudSyncSource, 'endpoint_configured', 'report endpoint readiness without persisting endpoint secrets'],
    [tauriCloudSyncSource, 'token_configured', 'report token readiness without persisting token values'],
    [tauriTtsSource, 'state.current_project_data_root().await', 'write generated TTS assets under the active project root'],
    [tauriTtsSource, 'project_root.join("assets").join("tts")', 'keep generated TTS files project-scoped'],
  ]
  const runtimeCompatibilityRequirements = [
    [gameCharacterSource, 'deserialize_relationships', 'migrate numeric and detailed legacy relationship values'],
    [gameDialogueScriptSource, 'node.id.clone_from(node_id)', 'treat dialogue node map keys as authoritative IDs'],
    [gameDialogueScriptSource, 'validate_graph', 'reject broken or unreachable dialogue graphs during runtime loading'],
    [gameDialogueNodeSource, 'pub is_ending: bool', 'preserve authored dialogue ending metadata'],
    [gameDialogueNodeSource, 'pub ending_type: Option<String>', 'preserve authored dialogue ending classifications'],
  ]
  appendRequirements(runtimeRootRequirements, issues)
  appendRequirements(runtimeCompatibilityRequirements, issues)

  for (const [source, file] of [
    [tauriAnalyticsSource, 'analytics.rs'],
    [tauriCloudSyncSource, 'cloud_sync.rs'],
    [tauriTtsSource, 'tts.rs'],
  ]) {
    if (source.includes('current_dir()')) {
      issues.push(`Tauri project-scoped command ${file} must not derive data paths from current_dir()`)
    }
  }

  return {
    issues,
    requirementCounts: {
      runtimeRoot: runtimeRootRequirements.length,
      runtimeCompatibility: runtimeCompatibilityRequirements.length,
    },
    structuralCheckCount: 3,
  }
}

function appendRequirements(requirements, issues) {
  for (const [source, needle, description] of requirements) {
    if (!source.includes(needle)) {
      issues.push(`Tauri runtime data-root handling must ${description}`)
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
      throw new Error(`Tauri project runtime policy requires ${name}.`)
    }
  }
  return boundaries
}
