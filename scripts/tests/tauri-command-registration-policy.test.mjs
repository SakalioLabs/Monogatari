import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'

import {
  collectTauriCommandRegistrationEvidence,
  extractTauriCommandDeclarations,
  extractTauriCommandRegistrations,
} from '../lib/tauri-packaging/command-registration-policy.mjs'

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..')
const tauriAppDirectory = path.join(repositoryRoot, 'rust-engine', 'crates', 'tauri-app')

test('every checked-in Tauri command declaration is registered exactly once', async () => {
  const evidence = await collectTauriCommandRegistrationEvidence({ tauriAppDirectory })
  assert.deepEqual(evidence.issues, [])
  assert.equal(evidence.declaredCount, 119)
  assert.equal(evidence.registeredCount, 119)
  assert.equal(evidence.commandFileCount, 26)
})

test('command declaration and handler parsers fail closed on unsupported shapes', () => {
  const declarations = extractTauriCommandDeclarations(
    `
#[tauri::command]
pub async fn alpha() {}

#[tauri::command(rename_all = "snake_case")]
pub(crate) fn beta() {}
`,
    'commands::fixture',
  )
  assert.deepEqual(declarations, ['commands::fixture::alpha', 'commands::fixture::beta'])

  const registrations = extractTauriCommandRegistrations(`
tauri::Builder::default().invoke_handler(tauri::generate_handler![
  commands::fixture::alpha,
  commands::fixture::beta,
])
`)
  assert.deepEqual(registrations, declarations)
  assert.throws(
    () => extractTauriCommandDeclarations('#[tauri::command]\npub struct NotACommand;', 'commands::fixture'),
    /parsed 0 of 1/,
  )
  assert.throws(
    () => extractTauriCommandRegistrations('tauri::generate_handler![direct_command]'),
    /unsupported generate_handler entry direct_command/,
  )
})

test('missing, undeclared, and duplicate registrations stay independently actionable', async () => {
  const mainPath = path.join(tauriAppDirectory, 'src', 'main.rs')
  const evidence = await collectTauriCommandRegistrationEvidence({
    tauriAppDirectory,
    async readTextFile(filePath, encoding) {
      const source = await readFile(filePath, encoding)
      if (path.resolve(filePath) !== mainPath) return source
      return source
        .replace('commands::story_events::get_story_progress,', '')
        .replace(
          'commands::engine::initialize_engine,',
          [
            'commands::engine::initialize_engine,',
            'commands::engine::initialize_engine,',
            'commands::engine::ghost_command,',
          ].join('\n'),
        )
    },
  })

  assert(
    evidence.issues.includes(
      'Tauri command registration is missing declared command commands::story_events::get_story_progress',
    ),
  )
  assert(
    evidence.issues.includes(
      'Tauri command registration references undeclared command commands::engine::ghost_command',
    ),
  )
  assert(
    evidence.issues.includes(
      'Tauri command registration duplicates command commands::engine::initialize_engine',
    ),
  )
})

test('dialog plugin and capability drift remains separate from command-set evidence', async () => {
  const mainPath = path.join(tauriAppDirectory, 'src', 'main.rs')
  const capabilityPath = path.join(tauriAppDirectory, 'capabilities', 'default.json')
  const evidence = await collectTauriCommandRegistrationEvidence({
    tauriAppDirectory,
    async readTextFile(filePath, encoding) {
      const source = await readFile(filePath, encoding)
      const resolved = path.resolve(filePath)
      if (resolved === mainPath) {
        return source.replace('tauri_plugin_dialog::init()', 'tauri_plugin_dialog::drifted()')
      }
      if (resolved === capabilityPath) {
        const capability = JSON.parse(source)
        capability.permissions = capability.permissions.filter(
          (permission) => permission !== 'dialog:allow-save',
        )
        return JSON.stringify(capability)
      }
      return source
    },
  })

  assert(
    evidence.issues.includes(
      'Tauri command registration must initialize the native project package dialog plugin',
    ),
  )
  assert(evidence.issues.includes('Tauri command capability must include dialog:allow-save'))
  assert.equal(evidence.declaredCount, evidence.registeredCount)
})

test('command registration policy requires the Tauri filesystem boundary', async () => {
  await assert.rejects(
    () => collectTauriCommandRegistrationEvidence(),
    /requires tauriAppDirectory/,
  )
})
