import assert from 'node:assert/strict'
import { mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises'
import os from 'node:os'
import path from 'node:path'
import test from 'node:test'

import {
  applyProjectAssetImport,
  planProjectAssetImport,
  ProjectAssetImportError,
} from '../lib/project-asset-import.mjs'

async function fixture() {
  const root = await mkdtemp(path.join(os.tmpdir(), 'monogatari_asset_import_'))
  const project = path.join(root, 'project')
  await mkdir(path.join(project, 'assets', 'models'), { recursive: true })
  await writeFile(path.join(project, 'settings.json'), '{}')
  const source = path.join(root, 'scene.glb')
  await writeFile(source, glbFixture())
  return { root, project, source }
}

function glbFixture(document = { asset: { version: '2.0', generator: 'test' }, scenes: [{ nodes: [] }], scene: 0 }) {
  const json = Buffer.from(JSON.stringify(document), 'utf8')
  const padding = (4 - (json.length % 4)) % 4
  const chunk = Buffer.concat([json, Buffer.alloc(padding, 0x20)])
  const output = Buffer.alloc(20 + chunk.length)
  output.write('glTF', 0, 'ascii')
  output.writeUInt32LE(2, 4)
  output.writeUInt32LE(output.length, 8)
  output.writeUInt32LE(chunk.length, 12)
  output.write('JSON', 16, 'ascii')
  chunk.copy(output, 20)
  return output
}

async function rejectsCode(promise, code) {
  await assert.rejects(promise, error => error instanceof ProjectAssetImportError && error.code === code)
}

test('plans and atomically applies a reviewed create-only GLB import', async () => {
  const value = await fixture()
  try {
    const input = {
      projectRoot: value.project,
      sourcePath: value.source,
      destinationPath: 'assets/models/memory_scene.glb',
      precondition: 'missing',
    }
    const plan = await planProjectAssetImport(input)
    assert.equal(plan.operation, 'create')
    assert.equal(plan.media_kind, 'model3d')
    assert.equal(plan.current_destination_sha256, null)

    const result = await applyProjectAssetImport({
      ...input,
      expectedPlanFingerprint: plan.plan_fingerprint,
    })
    assert.equal(result.destination_path, 'assets/models/memory_scene.glb')
    assert.deepEqual(
      await readFile(path.join(value.project, 'assets', 'models', 'memory_scene.glb')),
      await readFile(value.source),
    )
  } finally {
    await rm(value.root, { recursive: true, force: true })
  }
})

test('rejects a stale plan when source bytes change after review', async () => {
  const value = await fixture()
  try {
    const input = {
      projectRoot: value.project,
      sourcePath: value.source,
      destinationPath: 'assets/models/memory_scene.glb',
      precondition: 'missing',
    }
    const plan = await planProjectAssetImport(input)
    await writeFile(value.source, glbFixture({ asset: { version: '2.0' }, scenes: [{ name: 'changed' }] }))
    await rejectsCode(
      applyProjectAssetImport({ ...input, expectedPlanFingerprint: plan.plan_fingerprint }),
      'plan_fingerprint_stale',
    )
  } finally {
    await rm(value.root, { recursive: true, force: true })
  }
})

test('requires an exact destination SHA-256 before replacing an asset', async () => {
  const value = await fixture()
  try {
    const destination = path.join(value.project, 'assets', 'models', 'memory_scene.glb')
    await writeFile(destination, glbFixture())
    await rejectsCode(
      planProjectAssetImport({
        projectRoot: value.project,
        sourcePath: value.source,
        destinationPath: 'assets/models/memory_scene.glb',
        precondition: 'missing',
      }),
      'destination_precondition_failed',
    )
  } finally {
    await rm(value.root, { recursive: true, force: true })
  }
})

test('rejects traversal, unsupported destinations, and malformed GLB bytes', async () => {
  const value = await fixture()
  try {
    await rejectsCode(
      planProjectAssetImport({
        projectRoot: value.project,
        sourcePath: value.source,
        destinationPath: 'assets/models/../escape.glb',
        precondition: 'missing',
      }),
      'destination_path_invalid',
    )
    await rejectsCode(
      planProjectAssetImport({
        projectRoot: value.project,
        sourcePath: value.source,
        destinationPath: 'assets/models/scene.bin',
        precondition: 'missing',
      }),
      'asset_extension_mismatch',
    )
    await writeFile(value.source, 'not a glb')
    await rejectsCode(
      planProjectAssetImport({
        projectRoot: value.project,
        sourcePath: value.source,
        destinationPath: 'assets/models/scene.glb',
        precondition: 'missing',
      }),
      'glb_header_invalid',
    )
  } finally {
    await rm(value.root, { recursive: true, force: true })
  }
})
