import { readFile as readFileFromDisk } from 'node:fs/promises'
import path from 'node:path'

export async function collectTauriProjectPackageEvidence(options = {}) {
  const { rustDirectory, tauriAppDirectory } = resolveBoundaries(options)
  const readFile = options.readTextFile ?? readFileFromDisk
  const authoringDirectory = path.join(rustDirectory, 'crates', 'authoring', 'src')
  const projectPackageDirectory = path.join(authoringDirectory, 'project_package')
  const commandDirectory = path.join(tauriAppDirectory, 'src', 'commands')
  const tauriProjectSource = await readFile(path.join(commandDirectory, 'project.rs'), 'utf8')
  const tauriProjectArchiveSource = await readFile(
    path.join(commandDirectory, 'project_archive.rs'),
    'utf8',
  )
  const tauriProjectArchiveCommandsSource = await readFile(
    path.join(commandDirectory, 'project_archive', 'commands.rs'),
    'utf8',
  )
  const tauriProjectArchiveTests = await readFile(
    path.join(commandDirectory, 'project_archive', 'tests.rs'),
    'utf8',
  )
  const authoringProjectSource = await readFile(
    path.join(authoringDirectory, 'project.rs'),
    'utf8',
  )
  const authoringProjectPackageSource = await readFile(
    path.join(authoringDirectory, 'project_package.rs'),
    'utf8',
  )
  const authoringProjectPackageExportSource = await readFile(
    path.join(projectPackageDirectory, 'export.rs'),
    'utf8',
  )
  const authoringProjectPackageWriterSource = await readFile(
    path.join(projectPackageDirectory, 'archive_writer.rs'),
    'utf8',
  )
  const authoringProjectPackageWriterTests = await readFile(
    path.join(projectPackageDirectory, 'archive_writer', 'tests.rs'),
    'utf8',
  )
  const authoringProjectPackageReaderSource = await readFile(
    path.join(projectPackageDirectory, 'archive_reader.rs'),
    'utf8',
  )
  const authoringProjectPackageReaderTests = await readFile(
    path.join(projectPackageDirectory, 'archive_reader', 'tests.rs'),
    'utf8',
  )
  const authoringProjectPackagePathSource = await readFile(
    path.join(projectPackageDirectory, 'portable_path.rs'),
    'utf8',
  )
  const authoringProjectPackageManifestSource = await readFile(
    path.join(projectPackageDirectory, 'manifest.rs'),
    'utf8',
  )
  const issues = []

  const projectExportRequirements = [
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
  ]
  const packageProtocolRequirements = [
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
  ]
  appendRequirements(projectExportRequirements, issues)
  appendRequirements(packageProtocolRequirements, issues)

  if (tauriProjectArchiveSource.includes('mod manifest;') || tauriProjectArchiveSource.includes('mod path_validation;')) {
    issues.push('Tauri project archive commands must not redeclare shared package manifest or portable-path policy modules')
  }
  if (tauriProjectSource.includes('state.set_project_data_root(root.clone()).await')) {
    issues.push('saving settings.json must not switch the active project without loading its content managers')
  }

  return {
    issues,
    requirementCounts: {
      projectExport: projectExportRequirements.length,
      packageProtocol: packageProtocolRequirements.length,
    },
    structuralCheckCount: 2,
  }
}

function appendRequirements(requirements, issues) {
  for (const [source, needle, description] of requirements) {
    if (!source.includes(needle)) {
      issues.push(`Project package integration must ${description}`)
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
      throw new Error(`Tauri project package policy requires ${name}.`)
    }
  }
  return boundaries
}
