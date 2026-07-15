import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'

import { collectTauriProjectRuntimeEvidence } from '../lib/tauri-packaging/project-runtime-policy.mjs'

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..')
const rustDirectory = path.join(repositoryRoot, 'rust-engine')
const tauriAppDirectory = path.join(rustDirectory, 'crates', 'tauri-app')
const authoringDirectory = path.join(rustDirectory, 'crates', 'authoring', 'src')
const gameDirectory = path.join(rustDirectory, 'crates', 'game', 'src')
const commandDirectory = path.join(tauriAppDirectory, 'src', 'commands')
const boundaries = { rustDirectory, tauriAppDirectory }

test('checked-in project runtime roots and compatibility contracts return passing evidence', async () => {
  const evidence = await collectTauriProjectRuntimeEvidence(boundaries)
  assert.deepEqual(evidence.issues, [])
  assert.deepEqual(evidence.requirementCounts, {
    runtimeRoot: 48,
    runtimeCompatibility: 5,
  })
  assert.equal(evidence.structuralCheckCount, 3)
})

test('root binding, staged loading, compatibility, and current-directory drift stays actionable', async () => {
  const mainPath = path.join(tauriAppDirectory, 'src', 'main.rs')
  const statePath = path.join(tauriAppDirectory, 'src', 'state.rs')
  const runtimeValidationPath = path.join(authoringDirectory, 'runtime_validation.rs')
  const filesystemPath = path.join(authoringDirectory, 'filesystem.rs')
  const analyticsPath = path.join(commandDirectory, 'analytics.rs')
  const cloudSyncPath = path.join(commandDirectory, 'cloud_sync.rs')
  const ttsPath = path.join(commandDirectory, 'tts.rs')
  const dialogueScriptPath = path.join(gameDirectory, 'dialogue', 'dialogue_script.rs')
  const evidence = await collectTauriProjectRuntimeEvidence({
    ...boundaries,
    async readTextFile(filePath, encoding) {
      const source = await readFile(filePath, encoding)
      const resolved = path.resolve(filePath)
      if (resolved === mainPath) {
        return source.replaceAll('resource_dir()', 'resolve_app_resources()')
      }
      if (resolved === statePath) {
        return source.replaceAll('reset_project_runtime_state', 'retain_project_runtime_state')
      }
      if (resolved === runtimeValidationPath) {
        return source.replaceAll(
          'StoryEventCatalog::load_from_project_root(project_root)',
          'StoryEventCatalog::default()',
        )
      }
      if (resolved === filesystemPath) {
        return source.replaceAll('stage_json_replacement', 'write_json_replacement')
      }
      if (resolved === analyticsPath) {
        return [
          source.replaceAll(
            'state.current_project_data_root().await',
            'state.default_project_data_root()',
          ),
          'fn drifted_analytics_root() { let _ = std::env::current_dir(); }',
          '',
        ].join('\\n')
      }
      if (resolved === cloudSyncPath) {
        return [source, 'fn drifted_sync_root() { let _ = std::env::current_dir(); }', ''].join('\\n')
      }
      if (resolved === ttsPath) {
        return [source, 'fn drifted_tts_root() { let _ = std::env::current_dir(); }', ''].join('\\n')
      }
      if (resolved === dialogueScriptPath) {
        return source.replaceAll('validate_graph', 'accept_graph')
      }
      return source
    },
  })

  for (const issue of [
    'Tauri runtime data-root handling must resolve the Tauri resource directory during setup',
    'Tauri runtime data-root handling must clear mutable chat, scene, and script state across project reloads',
    'Tauri runtime data-root handling must stage project story events during shared engine initialization',
    'Tauri runtime data-root handling must stage bounded atomic JSON replacements',
    'Tauri runtime data-root handling must persist analytics under the active project root',
    'Tauri runtime data-root handling must reject broken or unreachable dialogue graphs during runtime loading',
    'Tauri project-scoped command analytics.rs must not derive data paths from current_dir()',
    'Tauri project-scoped command cloud_sync.rs must not derive data paths from current_dir()',
    'Tauri project-scoped command tts.rs must not derive data paths from current_dir()',
  ]) {
    assert(evidence.issues.includes(issue), issue)
  }
})

test('project runtime policy requires both Rust filesystem boundaries', async () => {
  await assert.rejects(
    () => collectTauriProjectRuntimeEvidence(),
    /requires rustDirectory/,
  )
  await assert.rejects(
    () => collectTauriProjectRuntimeEvidence({ rustDirectory }),
    /requires tauriAppDirectory/,
  )
})
