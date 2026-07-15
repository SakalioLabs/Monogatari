import { readFile as readFileFromDisk } from 'node:fs/promises'
import path from 'node:path'

import { collectTauriCommandRegistrationEvidence } from './tauri-packaging/command-registration-policy.mjs'
import { collectTauriInstallationPolicyEvidence } from './tauri-packaging/installation-policy.mjs'
import { collectTauriPackagePolicyEvidence } from './tauri-packaging/package-policy.mjs'

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
  const issues = [
    ...packagePolicyEvidence.issues,
    ...installationPolicyEvidence.issues,
    ...commandRegistrationEvidence.issues,
  ]
  const tauriBuildSource = await readFile(path.join(tauriAppDir, 'build.rs'), 'utf8')
  const rustToolchainSource = await readFile(path.join(rustDir, 'rust-toolchain.toml'), 'utf8')
  const releaseVerifierSource = await readFile(path.join(root, 'scripts', 'verify-release.mjs'), 'utf8')
  const tauriMainSource = await readFile(path.join(tauriAppDir, 'src', 'main.rs'), 'utf8')
  const tauriStateSource = await readFile(path.join(tauriAppDir, 'src', 'state.rs'), 'utf8')
  const tauriStoryEventsSource = await readFile(path.join(tauriAppDir, 'src', 'story_events.rs'), 'utf8')
  const authoringStoryEventsSource = await readFile(path.join(rustDir, 'crates', 'authoring', 'src', 'story_events.rs'), 'utf8')
  const authoringConversationQualitySource = await readFile(path.join(rustDir, 'crates', 'authoring', 'src', 'conversation_quality.rs'), 'utf8')
  const authoringQualityExecutionSource = await readFile(path.join(rustDir, 'crates', 'authoring', 'src', 'quality_suite_execution.rs'), 'utf8')
  const authoringQualityExecutionTests = await readFile(path.join(rustDir, 'crates', 'authoring', 'src', 'quality_suite_execution', 'tests.rs'), 'utf8')
  const authoringQualitySuiteSource = await readFile(path.join(rustDir, 'crates', 'authoring', 'src', 'quality_suite_validation.rs'), 'utf8')
  const authoringWorkflowSource = await readFile(path.join(rustDir, 'crates', 'authoring', 'src', 'workflow_validation.rs'), 'utf8')
  const authoringWorkflowPreviewSource = await readFile(path.join(rustDir, 'crates', 'authoring', 'src', 'workflow_preview.rs'), 'utf8')
  const authoringWorkflowPreviewTests = await readFile(path.join(rustDir, 'crates', 'authoring', 'src', 'workflow_preview', 'tests.rs'), 'utf8')
  const authoringRuntimeValidationSource = await readFile(path.join(rustDir, 'crates', 'authoring', 'src', 'runtime_validation.rs'), 'utf8')
  const tauriStoryProgressSource = await readFile(path.join(tauriAppDir, 'src', 'story_progress.rs'), 'utf8')
  const tauriStoryAccessSource = await readFile(path.join(tauriAppDir, 'src', 'story_access.rs'), 'utf8')
  const authoringFilesystemSource = await readFile(path.join(rustDir, 'crates', 'authoring', 'src', 'filesystem.rs'), 'utf8')
  const authoringProjectSource = await readFile(path.join(rustDir, 'crates', 'authoring', 'src', 'project.rs'), 'utf8')
  const authoringProjectPackageSource = await readFile(path.join(rustDir, 'crates', 'authoring', 'src', 'project_package.rs'), 'utf8')
  const authoringProjectPackageReaderSource = await readFile(path.join(rustDir, 'crates', 'authoring', 'src', 'project_package', 'archive_reader.rs'), 'utf8')
  const authoringProjectPackageReaderTests = await readFile(path.join(rustDir, 'crates', 'authoring', 'src', 'project_package', 'archive_reader', 'tests.rs'), 'utf8')
  const authoringProjectPackageWriterSource = await readFile(path.join(rustDir, 'crates', 'authoring', 'src', 'project_package', 'archive_writer.rs'), 'utf8')
  const authoringProjectPackageWriterTests = await readFile(path.join(rustDir, 'crates', 'authoring', 'src', 'project_package', 'archive_writer', 'tests.rs'), 'utf8')
  const authoringProjectPackageExportSource = await readFile(path.join(rustDir, 'crates', 'authoring', 'src', 'project_package', 'export.rs'), 'utf8')
  const authoringProjectPackagePathSource = await readFile(path.join(rustDir, 'crates', 'authoring', 'src', 'project_package', 'portable_path.rs'), 'utf8')
  const authoringProjectPackageManifestSource = await readFile(path.join(rustDir, 'crates', 'authoring', 'src', 'project_package', 'manifest.rs'), 'utf8')
  const tauriContentReferencesSource = await readFile(path.join(tauriAppDir, 'src', 'content_references.rs'), 'utf8')
  const tauriStoryEventCommandsSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'story_events.rs'), 'utf8')
  const tauriEndingCommandsSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'endings.rs'), 'utf8')
  const tauriDialogueCommandsSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'dialogue.rs'), 'utf8')
  const tauriKnowledgeCommandsSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'knowledge.rs'), 'utf8')
  const gameCharacterSource = await readFile(path.join(rustDir, 'crates', 'game', 'src', 'characters', 'character.rs'), 'utf8')
  const gameDialogueScriptSource = await readFile(path.join(rustDir, 'crates', 'game', 'src', 'dialogue', 'dialogue_script.rs'), 'utf8')
  const gameDialogueNodeSource = await readFile(path.join(rustDir, 'crates', 'game', 'src', 'dialogue', 'dialogue_node.rs'), 'utf8')
  const tauriEngineSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'engine.rs'), 'utf8')
  const tauriProjectSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'project.rs'), 'utf8')
  const tauriProjectArchiveSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'project_archive.rs'), 'utf8')
  const tauriProjectArchiveCommandsSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'project_archive', 'commands.rs'), 'utf8')
  const tauriProjectArchiveTests = await readFile(path.join(tauriAppDir, 'src', 'commands', 'project_archive', 'tests.rs'), 'utf8')
  const tauriScenesSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'scenes.rs'), 'utf8')
  const tauriChatSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'chat.rs'), 'utf8')
  const authoringPromptGuardSource = await readFile(path.join(rustDir, 'crates', 'authoring', 'src', 'prompt_guard.rs'), 'utf8')
  const tauriPromptGuardFacadeSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'prompt_guard.rs'), 'utf8')
  const tauriMultiChatSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'multi_chat.rs'), 'utf8')
  const tauriQualitySuiteSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'quality_suite.rs'), 'utf8')
  const tauriWorkflowSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'workflow.rs'), 'utf8')
  const tauriAnalyticsSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'analytics.rs'), 'utf8')
  const tauriCloudSyncSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'cloud_sync.rs'), 'utf8')
  const tauriTtsSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'tts.rs'), 'utf8')

  const runtimeDataRootRequirements = [
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
    [authoringProjectPackageExportSource, 'ARCHIVE_SCHEMA', 'emit the shared versioned project export manifest schema'],
    [authoringProjectPackageExportSource, '"project_path": "."', 'avoid leaking author filesystem paths in project handoff manifests'],
    [tauriProjectSource, 'project_export_provenance', 'centralize project export build provenance at the transport boundary'],
    [tauriProjectSource, 'CARGO_PKG_VERSION', 'bind project exports to the engine package version'],
    [tauriProjectSource, 'MONOGATARI_GIT_COMMIT', 'bind project exports to the build git commit'],
    [tauriProjectSource, 'MONOGATARI_GIT_SHORT_COMMIT', 'export a compact project export source commit'],
    [authoringProjectPackageExportSource, 'collect_project_file_inventory', 'include a file inventory in project export manifests'],
    [authoringProjectPackageExportSource, 'checksum_md5', 'keep legacy per-file MD5 checksums in project export manifests'],
    [authoringProjectPackageExportSource, 'checksum_sha256', 'include per-file SHA-256 checksums in project export manifests'],
    [authoringProjectPackageExportSource, 'fn checksum_sha256(bytes: &[u8])', 'centralize project export SHA-256 checksum generation'],
    [authoringProjectPackageExportSource, 'package_fingerprint(files.iter())', 'include whole-package SHA-256 fingerprints through the shared package protocol'],
    [authoringProjectPackageExportSource, 'project_content_summary', 'include content summaries in project export manifests'],
    [authoringProjectPackageExportSource, 'monogatari-project-content-summary/v1', 'version project export content summaries'],
    [authoringProjectPackageExportSource, 'category_counts', 'record per-category file counts in project export manifests'],
    [authoringProjectPackageExportSource, 'category_bytes', 'record per-category byte counts in project export manifests'],
    [authoringProjectPackageExportSource, 'category_fingerprint_algorithm', 'record project export category fingerprint algorithms'],
    [authoringProjectPackageExportSource, 'category_fingerprints', 'record per-category project export fingerprints'],
    [authoringProjectPackageExportSource, 'package_fingerprint(files.iter().copied())', 'fingerprint project export categories through the same shared algorithm'],
    [authoringProjectPackageManifestSource, 'hasher.update(record.path.as_bytes())', 'share project export file fingerprint inputs across generation and validation'],
    [authoringProjectPackageExportSource, 'json_file_count', 'record JSON source counts in project export manifests'],
    [authoringProjectPackageExportSource, 'asset_file_count', 'record asset counts in project export manifests'],
    [authoringProjectPackageManifestSource, 'sha256:path-size-file-sha256-v1', 'record one shared project export package fingerprint algorithm'],
    [authoringProjectPackageExportSource, 'content_sha256', 'emit package content fingerprints in project export manifests'],
    [authoringProjectPackageExportSource, 'sanitize_export_config', 'redact sensitive settings in project export manifests'],
    [tauriProjectSource, 'save_project_config_to_disk(&root, config).await', 'delegate project settings persistence to the headless authoring crate'],
    [tauriProjectSource, 'story_content_authoring_lock.lock().await', 'serialize settings changes with other project content mutations'],
    [authoringProjectSource, 'scrub_runtime_secret_config(&config)', 'scrub runtime secrets before saving or returning project settings'],
    [authoringProjectSource, 'MAX_PROJECT_SETTINGS_BYTES', 'bound project settings payloads'],
    [authoringProjectSource, 'stage_json_replacement(', 'atomically stage project settings saves'],
    [authoringProjectSource, 'staged.rollback().await?', 'restore previous settings after rejected staged saves'],
    [authoringProjectSource, 'settings_not_regular_file', 'reject non-regular and symlinked project settings'],
    [authoringProjectSource, 'settings_too_large', 'reject oversized project settings before reading them'],
    [authoringProjectSource, 'inspect_reports_non_regular_and_oversized_settings', 'test non-regular project settings rejection'],
    [authoringProjectSource, 'inspect_reports_non_regular_and_oversized_settings', 'test bounded project settings reads'],
    [authoringProjectSource, 'scrub_runtime_secret_string', 'scrub token-like and assignment-shaped secrets inside project setting string values'],
    [authoringProjectSource, 'scrub_token_like_values', 'scrub token-shaped values from project settings payloads'],
    [authoringProjectSource, 'scrub_secret_assignments', 'scrub secret assignments from project settings payloads'],
    [authoringProjectSource, 'is_secret_config_key', 'centralize project config secret key matching'],
    [authoringProjectSource, 'SECRET_CONFIG_KEYS', 'centralize sensitive export config keys'],
    [authoringProjectSource, 'save_is_atomic_and_scrubs_runtime_secrets', 'test project settings secret scrubbing before save'],
    [authoringProjectSource, 'inspect_scrubs_legacy_persisted_secrets_from_returned_state', 'test legacy project settings secrets are not returned to the frontend'],
    [authoringProjectPackageSource, 'mod export;', 'isolate headless project package generation'],
    [authoringProjectPackageExportSource, 'PROJECT_EXPORT_DIRECTORIES', 'declare exportable project directories explicitly'],
    [authoringProjectPackageExportSource, '("events", "events")', 'include story event catalogs in project exports'],
    [authoringProjectPackageExportSource, '("endings", "endings")', 'include story ending catalogs in project exports'],
    [authoringProjectPackageExportSource, 'ProjectExportRuntimeSnapshot', 'accept transport-neutral runtime fallback evidence'],
    [authoringProjectPackageExportSource, 'ProjectExportProvenance', 'keep export time and build identity caller-supplied'],
    [authoringProjectPackageExportSource, 'project_export_settings_bytes', 'fingerprint the same sanitized settings bytes written into project packages'],
    [authoringProjectPackageExportSource, 'Project exports cannot include symbolic links', 'reject symlinked project export sources'],
    [authoringProjectPackageExportSource, 'MAX_ARCHIVE_FILES', 'share project inventory file-count limits with import validation'],
    [authoringProjectPackageExportSource, 'MAX_ARCHIVE_DIRECTORIES', 'share bounded project inventory directory traversal'],
    [authoringProjectPackageExportSource, 'validate_portable_path', 'apply shared portable path policy before hashing'],
    [authoringProjectPackageExportSource, 'portable_case_key', 'reject portable case aliases during inventory generation'],
    [authoringProjectPackageExportSource, 'MAX_ARCHIVE_TOTAL_BYTES', 'share project inventory byte limits with import validation'],
    [authoringProjectPackageExportSource, 'checksum_export_file', 'stream project inventory checksums with fixed memory'],
    [authoringProjectPackageExportSource, 'validate_manifest(manifest.clone())?', 'self-validate generated project package manifests'],
    [authoringProjectPackageExportSource, 'export_manifest_inventories_files_and_redacts_secrets_headlessly', 'test real project package generation without Tauri'],
    [authoringProjectPackageExportSource, 'export_inventory_rejects_portable_aliases_and_unsafe_paths_without_io', 'test export-side portable collision rejection'],
    [tauriProjectSource, 'build_headless_project_export_manifest', 'delegate export generation to the shared headless package domain'],
    [tauriProjectSource, 'project_export_provenance()', 'inject desktop build and time provenance at the adapter boundary'],
    [authoringProjectPackageSource, 'pub const ARCHIVE_MANIFEST_PATH', 'pin the shared project package manifest path'],
    [authoringProjectPackageSource, 'mod archive_writer;', 'isolate shared atomic project package writing'],
    [authoringProjectPackageWriterSource, 'pub fn write_project_package', 'expose project package writing without Tauri'],
    [authoringProjectPackageWriterSource, 'ProjectPackageTargetPolicy', 'make package create and replacement intent explicit'],
    [authoringProjectPackageWriterSource, 'write_export_record', 'stream project files into ZIP output with fixed memory'],
    [authoringProjectPackageWriterSource, 'writer.write_all(&buffer[..read])', 'write project package assets incrementally'],
    [authoringProjectPackageWriterSource, 'commit_staged_package', 'commit generated project packages through a staged file'],
    [authoringProjectPackageWriterSource, 'MAX_ARCHIVE_COMPRESSED_BYTES', 'bound completed project package output before commit'],
    [authoringProjectPackageWriterTests, 'writes_streamed_credential_free_packages_without_tauri', 'test real project package output without desktop state'],
    [authoringProjectPackageWriterTests, 'failed_package_replacement_preserves_the_previous_file', 'test shared atomic package export rollback'],
    [authoringProjectPackageSource, 'mod manifest;', 'isolate shared project package manifest semantics'],
    [authoringProjectPackageManifestSource, 'MAX_ARCHIVE_TOTAL_BYTES', 'bound expanded project package sizes'],
    [authoringProjectPackageManifestSource, 'MAX_ARCHIVE_FILE_BYTES', 'bound individual project package files'],
    [authoringProjectPackageManifestSource, 'MAX_ARCHIVE_MANIFEST_BYTES', 'bound shared project package manifest wire size'],
    [authoringProjectPackageManifestSource, 'MAX_ARCHIVE_JSON_BYTES', 'bound JSON package files before archive I/O'],
    [authoringProjectPackageSource, 'mod archive_reader;', 'isolate shared project package inspection and extraction'],
    [authoringProjectPackageReaderSource, 'pub fn inspect_project_package', 'expose package verification without Tauri'],
    [authoringProjectPackageReaderSource, 'pub fn extract_project_package', 'expose package extraction without Tauri'],
    [authoringProjectPackageReaderSource, 'MAX_ARCHIVE_FILES', 'bound project package entry counts'],
    [authoringProjectPackageReaderSource, 'canonical_empty_extraction_root', 'require a caller-owned empty regular extraction root'],
    [authoringProjectPackageReaderSource, 'reject_non_regular_zip_entry', 'reject encrypted, symlink, and special ZIP entries'],
    [authoringProjectPackageReaderSource, 'verify_and_extract_entry', 'stream and verify project package contents during extraction'],
    [authoringProjectPackageReaderSource, '.create_new(true)', 'refuse to replace files while extracting project packages'],
    [authoringProjectPackageReaderSource, 'SHA-256 mismatch', 'reject tampered package files'],
    [authoringProjectPackageReaderSource, 'scrub_runtime_secret_config(&settings) != settings', 'reject imported settings containing runtime secrets'],
    [authoringProjectPackageReaderTests, 'generated_packages_inspect_and_extract_without_tauri', 'round-trip generated packages without desktop state'],
    [authoringProjectPackageReaderTests, 'package_reader_rejects_tampered_content', 'test shared archive checksum rejection'],
    [authoringProjectPackageReaderTests, 'package_reader_rejects_undeclared_and_duplicate_entries', 'test shared archive inventory rejection'],
    [authoringProjectPackageReaderTests, 'package_reader_rejects_secret_bearing_settings_after_checksum_validation', 'test shared archive secret rejection'],
    [authoringProjectPackageReaderTests, 'package_extraction_requires_an_existing_empty_regular_directory', 'test shared extraction-root boundaries'],
    [authoringProjectPackageReaderTests, 'package_reader_rejects_invalid_extensions_and_special_zip_entries', 'test shared package and ZIP entry type boundaries'],
    [authoringProjectPackageSource, 'mod portable_path;', 'isolate shared project package path policy'],
    [authoringProjectPackagePathSource, 'validate_portable_path', 'reject traversal and non-portable archive paths'],
    [authoringProjectPackagePathSource, 'portable_paths_reject_escape_reserved_and_platform_specific_shapes', 'independently test portable archive path rejection'],
    [authoringProjectPackageManifestSource, 'minimal_manifest_validates_without_zip_io', 'independently test project package manifest acceptance'],
    [authoringProjectPackageManifestSource, 'manifest_rejects_declared_size_bombs_without_allocating', 'independently test package size-bomb rejection'],
    [tauriProjectArchiveSource, 'use llm_authoring::project_package::{', 'reuse the shared project package protocol'],
    [tauriProjectArchiveSource, 'write_project_package(', 'delegate archive output to the shared headless writer'],
    [tauriProjectArchiveSource, 'remove_import_staging', 'remove rejected project import staging directories'],
    [tauriProjectArchiveSource, 'pub(crate) mod commands;', 'keep project package transport orchestration in its own module'],
    [tauriProjectArchiveCommandsSource, 'inspect_project_package(&archive_path)', 'delegate package inspection to the shared reader'],
    [tauriProjectArchiveCommandsSource, 'extract_project_package(&archive_for_task, &staging_for_task)', 'delegate package extraction to the shared reader'],
    [tauriProjectArchiveCommandsSource, 'validate_project_delivery(&staging_root)', 'share core-runtime and delivery acceptance before committing imported projects'],
    [tauriProjectArchiveTests, 'checked_in_project_packages_reload_as_runtime_content', 'round-trip checked-in project content through a real package'],
    [tauriProjectArchiveTests, 'failed_archive_exports_preserve_existing_packages', 'test atomic package export rollback'],
    [gameCharacterSource, 'deserialize_relationships', 'migrate numeric and detailed legacy relationship values'],
    [gameDialogueScriptSource, 'node.id.clone_from(node_id)', 'treat dialogue node map keys as authoritative IDs'],
    [gameDialogueScriptSource, 'validate_graph', 'reject broken or unreachable dialogue graphs during runtime loading'],
    [gameDialogueNodeSource, 'pub is_ending: bool', 'preserve authored dialogue ending metadata'],
    [gameDialogueNodeSource, 'pub ending_type: Option<String>', 'preserve authored dialogue ending classifications'],
  ]
  for (const [source, needle, description] of runtimeDataRootRequirements) {
    if (!source.includes(needle)) {
      issues.push(`Tauri runtime data-root handling must ${description}`)
    }
  }
  if (tauriProjectArchiveSource.includes('mod manifest;') || tauriProjectArchiveSource.includes('mod path_validation;')) {
    issues.push('Tauri project archive commands must not redeclare shared package manifest or portable-path policy modules')
  }
  if (tauriProjectSource.includes('state.set_project_data_root(root.clone()).await')) {
    issues.push('saving settings.json must not switch the active project without loading its content managers')
  }

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
  for (const [needle, description] of conversationQualityRequirements) {
    if (!authoringConversationQualitySource.includes(needle)) {
      issues.push(`Headless conversation quality must ${description}`)
    }
  }
  if (!tauriChatSource.includes('pub use llm_authoring::conversation_quality::{')) {
    issues.push('Tauri chat commands must reuse the shared headless conversation quality models')
  }
  if (/pub\s+struct\s+(?:ChatSafetyTrace|ConversationEvaluation)\s*\{/.test(tauriChatSource)) {
    issues.push('Tauri chat commands must not duplicate headless conversation quality models')
  }

  const sharedQualityInputRequirements = [
    [authoringQualitySuiteSource, 'pub struct QualitySuiteDocument', 'own the Quality Suite document model'],
    [authoringQualitySuiteSource, 'pub struct QualityScenarioDocument', 'own the Quality scenario model'],
    [authoringQualitySuiteSource, 'pub struct QualityMessage', 'own the Quality message model'],
    [authoringQualitySuiteSource, 'pub struct QualityExpectation', 'own the Quality expectation model'],
    [authoringQualitySuiteSource, 'pub workflow_run_contexts: Vec<WorkflowRunContext>', 'parse typed Workflow run contexts'],
    [authoringWorkflowSource, 'pub struct WorkflowRunContext', 'own the Workflow run-context model'],
    [tauriQualitySuiteSource, 'QualitySuiteDocument as QualitySuite', 'reuse the shared Quality Suite model'],
    [tauriQualitySuiteSource, 'QualityScenarioDocument as QualityScenario', 'reuse the shared Quality scenario model'],
    [tauriQualitySuiteSource, 'QualityMessage', 'reuse the shared Quality message model'],
    [tauriQualitySuiteSource, 'QualityExpectation', 'reuse the shared Quality expectation model'],
    [tauriWorkflowSource, 'WorkflowRunContext', 'reuse the shared Workflow run-context model'],
  ]
  for (const [source, needle, description] of sharedQualityInputRequirements) {
    if (!source.includes(needle)) {
      issues.push(`Headless Quality input contracts must ${description}`)
    }
  }
  if (/pub\s+struct\s+Quality(?:Suite|Scenario|Message|Expectation)\s*\{/.test(tauriQualitySuiteSource)) {
    issues.push('Tauri Quality commands must not duplicate shared headless input models')
  }
  if (/pub\s+struct\s+WorkflowRunContext\s*\{/.test(tauriWorkflowSource)) {
    issues.push('Tauri Workflow commands must not duplicate the shared run-context model')
  }
  const tauriQualityParserSource = tauriQualitySuiteSource.match(
    /pub\(crate\) fn parse_quality_suite[\s\S]*?\n\}/,
  )?.[0] ?? ''
  if (
    !tauriQualityParserSource.includes('parse_quality_suite_document(content)')
    || tauriQualityParserSource.includes('serde_json::from_str')
  ) {
    issues.push('Tauri Quality parsing must delegate directly to the shared headless parser')
  }

  const headlessWorkflowPreviewRequirements = [
    [authoringWorkflowPreviewSource, 'pub fn execute_workflow_preview', 'expose deterministic Workflow preview execution'],
    [authoringWorkflowPreviewSource, 'pub struct WorkflowPreviewEnvironment', 'accept transport-neutral preview state'],
    [authoringWorkflowPreviewSource, 'pub struct WorkflowPreviewOptions', 'accept bounded execution and deterministic branch options'],
    [authoringWorkflowPreviewSource, 'struct DeterministicRandom', 'make random branches reproducible'],
    [authoringWorkflowPreviewSource, '"simulated": true', 'simulate LLM nodes without requiring a provider'],
    [authoringWorkflowPreviewTests, 'executes_context_state_and_conditions_without_tauri', 'test stateful preview execution without Tauri'],
    [authoringWorkflowPreviewTests, 'random_branches_are_deterministic_and_injectable', 'test deterministic random branches'],
    [authoringWorkflowPreviewTests, 'event_decisions_use_typed_context_and_trigger_history', 'test Event decisions from typed preview context'],
    [tauriWorkflowSource, 'execute_workflow_preview(', 'delegate run-context previews to the headless executor'],
    [tauriWorkflowSource, 'workflow_preview_environment', 'adapt desktop state into the headless preview environment'],
    [authoringQualityExecutionSource, 'execute_workflow_preview(', 'run Quality Workflow coverage without desktop state'],
  ]
  for (const [source, needle, description] of headlessWorkflowPreviewRequirements) {
    if (!source.includes(needle)) {
      issues.push(`Headless Workflow preview must ${description}`)
    }
  }
  if (/struct\s+WorkflowPreviewState\s*\{/.test(tauriWorkflowSource)) {
    issues.push('Tauri Workflow commands must not redeclare the headless preview state machine')
  }
  const qualityWorkflowCoverageSource = authoringQualityExecutionSource.match(
    /fn scenario_workflow_coverage[\s\S]*?\n\}/,
  )?.[0] ?? ''
  if (
    !qualityWorkflowCoverageSource.includes('execute_workflow_preview(')
    || qualityWorkflowCoverageSource.includes('AppState::new()')
  ) {
    issues.push('Quality Workflow coverage must execute through the headless preview domain')
  }

  const headlessQualityExecutionRequirements = [
    [authoringQualityExecutionSource, 'pub fn execute_quality_suite', 'own complete Quality Suite execution'],
    [authoringQualityExecutionSource, 'fn run_quality_scenario', 'own scenario evidence aggregation'],
    [authoringQualityExecutionSource, 'fn scenario_knowledge_evidence', 'own project knowledge evidence'],
    [authoringQualityExecutionSource, 'pub struct QualitySuiteReport', 'own the structured Quality report contract'],
    [authoringQualityExecutionTests, 'checked_in_character_stability_suite_passes_without_tauri', 'test the built-in suite without Tauri'],
    [authoringQualityExecutionTests, 'tideglass_quality_workflows_reach_full_coverage_without_tauri', 'test Tideglass Workflow coverage without Tauri'],
    [authoringQualityExecutionTests, 'failed_expectations_return_actionable_headless_evidence', 'test structured failure evidence without Tauri'],
    [tauriQualitySuiteSource, 'execute_quality_suite(', 'delegate execution to the headless Quality domain'],
    [tauriQualitySuiteSource, 'quality_suite_run_provenance', 'adapt build provenance for headless reports'],
  ]
  for (const [source, needle, description] of headlessQualityExecutionRequirements) {
    if (!source.includes(needle)) {
      issues.push(`Headless Quality execution must ${description}`)
    }
  }
  if (/fn\s+(?:run_quality_scenario|scenario_knowledge_evidence|validate_scenario_expectations)\s*\(/.test(tauriQualitySuiteSource)) {
    issues.push('Tauri Quality commands must not redeclare headless scenario execution or evidence logic')
  }

  const chatSafetyContractSource = `${authoringConversationQualitySource}\n${tauriChatSource}`
  for (const [needle, description] of chatSafetyTraceRequirements) {
    if (!chatSafetyContractSource.includes(needle)) {
      issues.push(`Chat runtime safety tracing must ${description}`)
    }
  }

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
  if (!tauriPromptGuardFacadeSource.includes('pub use llm_authoring::prompt_guard::*;')) {
    issues.push('Tauri prompt guard commands must delegate to the shared headless authoring domain')
  }
  for (const [needle, description] of multilingualPromptGuardRequirements) {
    if (!authoringPromptGuardSource.includes(needle)) {
      issues.push(`Prompt guard multilingual coverage must ${description}`)
    }
  }

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
  for (const [needle, description] of multilingualFallbackScoringRequirements) {
    if (!authoringConversationQualitySource.includes(needle)) {
      issues.push(`Fallback scoring multilingual coverage must ${description}`)
    }
  }

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
  for (const [needle, description] of groupChatSafetyTraceRequirements) {
    if (!tauriMultiChatSource.includes(needle)) {
      issues.push(`Group chat runtime safety tracing must ${description}`)
    }
  }

  const qualityRuntimeTraceRequirements = [
    ['runtime_safety_trace: Option<chat::ChatSafetyTrace>', 'export runtime safety traces in quality scenario reports'],
    ['runtime_safety_trace_required', 'let quality suites require runtime safety trace evidence'],
    ['required_runtime_guard_notes', 'let quality suites require specific guard notes'],
    ['runtime_guard_interventions', 'count runtime guard interventions in audit summaries'],
    ['scenario_runtime_safety_trace', 'centralize quality runtime trace construction'],
    ['chat::build_chat_safety_trace', 'reuse the chat safety trace contract in quality reports'],
    ['chat::build_event_trigger_decisions', 'reuse the chat story event decision contract in quality reports'],
    ['rule_fingerprint', 'carry story event rule fingerprints into quality reports'],
    ['expected.rule_fingerprint', 'let quality suites pin event rule fingerprints when needed'],
    ['pinned_knowledge_ref_count', 'carry pinned knowledge evidence into quality runtime traces'],
    ['pinned_knowledge_ref_ids', 'carry pinned knowledge ref ids into quality runtime traces'],
    ['guard_workflow_story_output', 'reuse runtime workflow LLM output finalization in quality reports'],
    ['workflow_output_equals', 'let workflow quality scenarios assert finalized workflow output text'],
    ['workflow_output: Option<String>', 'export finalized workflow output text in quality reports'],
    ['workflow_output_report', 'omit empty workflow output evidence from non-workflow scenarios'],
    ['pub struct QualitySuiteSummary', 'export quality suite summaries for the workbench'],
    ['QualitySuiteRunMetadata', 'export quality suite run metadata'],
    ['quality_suite_run_metadata', 'centralize quality suite run metadata generation'],
    ['LoadedQualitySuite', 'return backend-confirmed quality suite source paths with loaded suites'],
    ['source_sha256', 'return backend-confirmed quality suite content fingerprints with loaded suites'],
    ['quality_suite_source_path', 'normalize quality suite source paths for QA reports'],
    ['quality_suite_sha256', 'hash quality suite source content for QA reports'],
    ['quality_suite_loader_reports_relative_source_path', 'test quality suite report source paths stay project-relative'],
    ['quality_suite_summary_reports_source_fingerprint', 'test quality suite summaries expose source fingerprints'],
    ['suite_sha256', 'export quality suite content fingerprints in run metadata'],
    ['CARGO_PKG_VERSION', 'bind quality suite run metadata to the engine package version'],
    ['MONOGATARI_GIT_COMMIT', 'bind quality suite run metadata to the build git commit'],
    ['MONOGATARI_GIT_SHORT_COMMIT', 'export a compact git commit for quality report UI evidence'],
    ['reports_workflow_output_finalization_mismatches', 'test finalized workflow output expectations fail loudly'],
  ]
  const qualityExecutionContractSource = `${authoringQualityExecutionSource}\n${tauriQualitySuiteSource}`
  for (const [needle, description] of qualityRuntimeTraceRequirements) {
    if (!qualityExecutionContractSource.includes(needle)) {
      issues.push(`Quality suite runtime safety tracing must ${description}`)
    }
  }

  const buildMetadataRequirements = [
    ['cargo:rustc-env=MONOGATARI_GIT_COMMIT', 'inject the build git commit into the Tauri binary'],
    ['cargo:rustc-env=MONOGATARI_GIT_SHORT_COMMIT', 'inject a short build git commit into the Tauri binary'],
    ['rev-parse', 'derive build commit metadata from git'],
    ['symbolic-ref', 'rerun the build script when the current branch ref changes'],
  ]
  for (const [needle, description] of buildMetadataRequirements) {
    if (!tauriBuildSource.includes(needle)) {
      issues.push(`Tauri build metadata must ${description}`)
    }
  }

  const rustToolchainRequirements = [
    [rustToolchainSource, 'channel = "nightly-2026-07-03"', 'pin the verified Rust nightly by exact date'],
    [rustToolchainSource, 'profile = "minimal"', 'keep release toolchain installation minimal'],
    [rustToolchainSource, 'components = ["clippy", "rustfmt"]', 'install the linter and formatter used by release verification'],
    [releaseVerifierSource, "env: { CARGO_INCREMENTAL: '0' }", 'disable incremental Tauri test compilation deterministically'],
  ]
  for (const [source, needle, description] of rustToolchainRequirements) {
    if (!source.includes(needle)) {
      issues.push(`Rust release toolchain must ${description}`)
    }
  }
  const forbiddenTestProfileOverride = ['CARGO', 'PROFILE', 'TEST', 'DEBUG'].join('_')
  if (releaseVerifierSource.includes(forbiddenTestProfileOverride)) {
    issues.push('Rust release verification must not override the Tauri test debug-profile environment')
  }

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
