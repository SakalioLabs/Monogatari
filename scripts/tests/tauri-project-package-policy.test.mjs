import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'

import { collectTauriProjectPackageEvidence } from '../lib/tauri-packaging/project-package-policy.mjs'

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..')
const rustDirectory = path.join(repositoryRoot, 'rust-engine')
const tauriAppDirectory = path.join(rustDirectory, 'crates', 'tauri-app')
const authoringDirectory = path.join(rustDirectory, 'crates', 'authoring', 'src')
const packageDirectory = path.join(authoringDirectory, 'project_package')
const commandDirectory = path.join(tauriAppDirectory, 'src', 'commands')
const boundaries = { rustDirectory, tauriAppDirectory }

test('checked-in project export and package protocol return passing evidence', async () => {
  const evidence = await collectTauriProjectPackageEvidence(boundaries)
  assert.deepEqual(evidence.issues, [])
  assert.deepEqual(evidence.requirementCounts, {
    projectExport: 60,
    packageProtocol: 45,
  })
  assert.equal(evidence.structuralCheckCount, 2)
})

test('export, writer, reader, portable path, and Tauri adapter drift stays actionable', async () => {
  const exportPath = path.join(packageDirectory, 'export.rs')
  const writerPath = path.join(packageDirectory, 'archive_writer.rs')
  const readerPath = path.join(packageDirectory, 'archive_reader.rs')
  const portablePath = path.join(packageDirectory, 'portable_path.rs')
  const tauriProjectPath = path.join(commandDirectory, 'project.rs')
  const tauriArchivePath = path.join(commandDirectory, 'project_archive.rs')
  const evidence = await collectTauriProjectPackageEvidence({
    ...boundaries,
    async readTextFile(filePath, encoding) {
      const source = await readFile(filePath, encoding)
      const resolved = path.resolve(filePath)
      if (resolved === exportPath) {
        return source.replaceAll('ARCHIVE_SCHEMA', 'PROJECT_WIRE_VERSION')
      }
      if (resolved === writerPath) {
        return source.replaceAll('pub fn write_project_package', 'pub fn write_drifted_package')
      }
      if (resolved === readerPath) {
        return source.replaceAll('pub fn inspect_project_package', 'pub fn inspect_drifted_package')
      }
      if (resolved === portablePath) {
        return source.replaceAll('validate_portable_path', 'accept_nonportable_path')
      }
      if (resolved === tauriProjectPath) {
        return [
          source.replaceAll('project_export_provenance', 'drifted_export_provenance'),
          'async fn drifted_settings_save(state: &AppState, root: PathBuf) {',
          '    state.set_project_data_root(root.clone()).await;',
          '}',
          '',
        ].join('\\n')
      }
      if (resolved === tauriArchivePath) {
        return [source, 'mod manifest;', ''].join('\\n')
      }
      return source
    },
  })

  for (const issue of [
    'Project package integration must emit the shared versioned project export manifest schema',
    'Project package integration must centralize project export build provenance at the transport boundary',
    'Project package integration must expose project package writing without Tauri',
    'Project package integration must expose package verification without Tauri',
    'Project package integration must reject traversal and non-portable archive paths',
    'Tauri project archive commands must not redeclare shared package manifest or portable-path policy modules',
    'saving settings.json must not switch the active project without loading its content managers',
  ]) {
    assert(evidence.issues.includes(issue), issue)
  }
})

test('project package policy requires both Rust filesystem boundaries', async () => {
  await assert.rejects(
    () => collectTauriProjectPackageEvidence(),
    /requires rustDirectory/,
  )
  await assert.rejects(
    () => collectTauriProjectPackageEvidence({ rustDirectory }),
    /requires tauriAppDirectory/,
  )
})
