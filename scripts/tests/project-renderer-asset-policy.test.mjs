import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'

import {
  collectProjectRendererAssetEvidence,
  createProjectRendererAssetPolicy,
} from '../lib/project-content/renderer-asset-policy.mjs'

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..')
const rustDirectory = path.join(repositoryRoot, 'rust-engine')
const boundaries = { repositoryRoot, rustDirectory }

test('checked-in renderer assets return cross-root passing evidence', async () => {
  const messages = []
  const policy = createProjectRendererAssetPolicy({
    ...boundaries,
    log(message) {
      messages.push(message)
    },
  })
  const evidence = await policy.verifyRendererAssets()

  assert.deepEqual(evidence.issues, [])
  assert.deepEqual(
    {
      characterCount: evidence.characterCount,
      sceneCount: evidence.sceneCount,
      sceneBackgroundCount: evidence.sceneBackgroundCount,
      sceneModel3dCount: evidence.sceneModel3dCount,
      declaredCharacterAssetCount: evidence.declaredCharacterAssetCount,
    },
    {
      characterCount: 40,
      sceneCount: 24,
      sceneBackgroundCount: 24,
      sceneModel3dCount: 6,
      declaredCharacterAssetCount: 34,
    },
  )
  assert.deepEqual(messages, [
    '[release] Renderer assets OK (40 character record(s), 24/24 scene background(s), 6 scene 3D model(s), 34 declared character asset(s))',
  ])
})

test('character and scene asset drift stays independently actionable', async () => {
  const kenjiPath = path.join(repositoryRoot, 'data', 'characters', 'kenji.json')
  const sakuraPath = path.join(repositoryRoot, 'data', 'characters', 'sakura.json')
  const crossroadsPath = path.join(repositoryRoot, 'data', 'scenes', 'crossroads.json')
  const libraryPath = path.join(repositoryRoot, 'data', 'scenes', 'great_library.json')
  const orbitPath = path.join(repositoryRoot, 'data', 'scenes', 'blue_frame_orbit.json')
  const evidence = await collectProjectRendererAssetEvidence({
    ...boundaries,
    async readFile(filePath, encoding) {
      const source = await readFile(filePath, encoding)
      const resolved = path.resolve(filePath)
      if (resolved === kenjiPath) {
        const character = JSON.parse(source)
        character.portrait_path = null
        character.sprite_path = null
        return JSON.stringify(character)
      }
      if (resolved === sakuraPath) {
        const character = JSON.parse(source)
        character.portrait_path = '../outside.png'
        return JSON.stringify(character)
      }
      if (resolved === crossroadsPath) {
        const scene = JSON.parse(source)
        scene.background_path = 'https://example.invalid/crossroads.svg'
        return JSON.stringify(scene)
      }
      if (resolved === libraryPath) {
        const scene = JSON.parse(source)
        scene.background_path = 'assets/backgrounds/missing.bmp'
        return JSON.stringify(scene)
      }
      if (resolved === orbitPath) {
        const scene = JSON.parse(source)
        scene.model_3d_path = 'assets/models/missing.glb'
        return JSON.stringify(scene)
      }
      return source
    },
  })

  for (const issue of [
    'data: core sample character kenji must declare a checked-in renderer asset',
    'data/characters/sakura.json:sakura portrait: renderer asset path must not contain parent traversal: ../outside.png',
    'data/scenes/crossroads.json background: checked-in renderer assets must use project-relative paths, got https://example.invalid/crossroads.svg',
    'data/scenes/great_library.json background: unsupported renderer asset extension .bmp',
    'data/scenes/great_library.json background: renderer asset does not exist: data/assets/backgrounds/missing.bmp',
    'data/scenes/blue_frame_orbit.json 3D model: renderer asset does not exist: data/assets/models/missing.glb',
  ]) {
    assert(evidence.issues.includes(issue), issue)
  }
})

test('GLB structure, fingerprint, and attribution drift stays independently actionable', async () => {
  const dataModelPath = path.join(repositoryRoot, 'data', 'assets', 'models', 'fox.glb')
  const rustModelPath = path.join(rustDirectory, 'data', 'assets', 'models', 'fox.glb')
  const evidence = await collectProjectRendererAssetEvidence({
    ...boundaries,
    async readFile(filePath, encoding) {
      const resolved = path.resolve(filePath)
      if (resolved === dataModelPath) {
        const model = Buffer.from(await readFile(filePath))
        model.writeUInt32LE(1, 4)
        model.writeUInt32LE(model.length + 4, 8)
        model[model.length - 1] ^= 1
        return model
      }
      if (resolved === rustModelPath) return Buffer.from('not-a-binary-gltf')
      const source = await readFile(filePath, encoding)
      if (filePath.endsWith('fox.LICENSE.txt')) {
        return source.replaceAll('CC BY 4.0', 'missing-license-marker')
      }
      return source
    },
  })

  for (const issue of [
    'data: 3D fixture must use glTF version 2',
    'data: 3D fixture declared length must match file size',
    'data: 3D fixture SHA-256 mismatch:',
    'data: 3D fixture attribution must include CC BY 4.0',
    'rust-engine/data: 3D fixture must be a binary glTF file',
    'rust-engine/data: 3D fixture attribution must include CC BY 4.0',
  ]) {
    assert(evidence.issues.some((entry) => entry.startsWith(issue)), issue)
  }
})

test('Project Renderer Asset policy requires explicit roots and unique data-root labels', () => {
  assert.throws(() => createProjectRendererAssetPolicy(), /requires repositoryRoot/)
  assert.throws(
    () => createProjectRendererAssetPolicy({ repositoryRoot }),
    /requires rustDirectory/,
  )
  assert.throws(
    () => createProjectRendererAssetPolicy({
      ...boundaries,
      dataRoots: [
        { label: 'project', dir: path.join(repositoryRoot, 'data') },
        { label: 'project', dir: path.join(rustDirectory, 'data') },
      ],
    }),
    /label is duplicated/,
  )
})

test('release runner delegates Renderer Asset policy without retaining asset rules', async () => {
  const runnerSource = await readFile(path.join(repositoryRoot, 'scripts', 'verify-release.mjs'), 'utf8')
  const policySource = await readFile(
    path.join(repositoryRoot, 'scripts', 'lib', 'project-content', 'renderer-asset-policy.mjs'),
    'utf8',
  )

  assert(runnerSource.includes('createProjectRendererAssetPolicy'))
  assert(runnerSource.includes('verifyRendererAssets'))
  assert(!runnerSource.includes('async function verifyRendererAssets'))
  assert(!runnerSource.includes('requiredModel3dFixtureSha256'))
  assert(!runnerSource.includes('verifyLocalAssetPath'))
  assert(policySource.includes('async function collectRendererAssetEvidence'))
  assert(policySource.includes('requiredModel3dFixtureSha256'))
  assert(policySource.includes('verifyLocalAssetPath'))
})
