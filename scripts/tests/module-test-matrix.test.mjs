import assert from 'node:assert/strict'
import { mkdtemp, mkdir, readFile, rm, stat, writeFile } from 'node:fs/promises'
import os from 'node:os'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'

import {
  aggregateReportStatus,
  assertOwnedReportDocument,
  isModuleSupported,
  parseArguments,
  selectModules,
  spawnSpecForPlatform,
  validateMatrix,
} from '../lib/module-test-matrix.mjs'
import { projectMirrorDiff, synchronizeProjectMirror } from '../lib/project-mirror.mjs'
import {
  extractBrowserWorkflowNodeCatalog,
  extractRustWorkflowNodeCatalog,
} from '../lib/source-invariant-verifier.mjs'

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..')

test('checked-in matrix is valid and owns every primary implementation surface', async () => {
  const source = JSON.parse(await readFile(path.join(repositoryRoot, 'scripts', 'module-test-matrix.json'), 'utf8'))
  const matrix = validateMatrix(source, repositoryRoot)
  const ownerPaths = new Set(matrix.modules.flatMap((module) => module.ownerPaths))

  for (const requiredPath of [
    'frontend/src',
    'frontend/vitest.config.ts',
    'frontend/playwright.config.ts',
    'rust-engine/crates/core',
    'rust-engine/crates/ai',
    'rust-engine/crates/assets',
    'rust-engine/crates/scripting',
    'rust-engine/crates/game',
    'rust-engine/crates/authoring',
    'rust-engine/crates/mcp-server',
    'rust-engine/crates/tauri-app',
    'src',
    'tests',
    '.agents/skills',
  ]) {
    assert(ownerPaths.has(requiredPath), `missing module owner for ${requiredPath}`)
  }
})

test('checked-in Workflow node catalogs remain structurally equivalent', async () => {
  const rustSource = await readFile(path.join(
    repositoryRoot,
    'rust-engine',
    'crates',
    'authoring',
    'src',
    'workflow_validation.rs',
  ), 'utf8')
  const browserSource = await readFile(path.join(
    repositoryRoot,
    'frontend',
    'src',
    'lib',
    'workflowAuthoring.ts',
  ), 'utf8')

  const rustCatalog = extractRustWorkflowNodeCatalog(rustSource)
  const browserCatalog = extractBrowserWorkflowNodeCatalog(browserSource)
  assert.equal(rustCatalog.length, 21)
  assert.deepEqual(browserCatalog, rustCatalog)
  assert.deepEqual(
    rustCatalog.find((entry) => entry.nodeType === 'dialogue')?.configurableFields,
    ['speaker', 'text', 'emotion', 'use_llm'],
  )
  assert.deepEqual(
    rustCatalog.find((entry) => entry.nodeType === 'llm_generate')?.configurableFields,
    ['prompt', 'system_prompt', 'max_tokens'],
  )
})

test('Workflow node catalog extraction fails closed for opaque definitions', () => {
  assert.deepEqual(extractRustWorkflowNodeCatalog('pub fn another_catalog() {}'), [])
  assert.deepEqual(extractBrowserWorkflowNodeCatalog('const OTHER_TYPES = []'), [])
})

test('matrix rejects duplicate IDs and paths outside the repository', () => {
  const module = validModule()
  assert.throws(
    () => validateMatrix({ schema: 'monogatari-module-test-matrix/v1', modules: [module, module] }, repositoryRoot),
    /Duplicate module ID/,
  )
  assert.throws(
    () => validateMatrix({
      schema: 'monogatari-module-test-matrix/v1',
      modules: [{ ...module, cwd: '../outside' }],
    }, repositoryRoot),
    /escapes the repository root/,
  )
  assert.throws(
    () => validateMatrix({
      schema: 'monogatari-module-test-matrix/v1',
      modules: [{ ...module, owner_paths: ['definitely-missing-module-path'] }],
    }, repositoryRoot),
    /does not exist/,
  )
})

test('selection defaults to required gates and supports module and group unions', () => {
  const matrix = validateMatrix({
    schema: 'monogatari-module-test-matrix/v1',
    modules: [
      validModule({ id: 'core', group: 'rust', default: true }),
      validModule({ id: 'app', group: 'rust', default: false }),
      validModule({ id: 'web', group: 'frontend', default: true }),
    ],
  }, repositoryRoot)

  assert.deepEqual(selectModules(matrix).map((module) => module.id), ['core', 'web'])
  assert.deepEqual(
    selectModules(matrix, { moduleIds: ['web'], groups: ['rust'] }).map((module) => module.id),
    ['core', 'app', 'web'],
  )
  assert.throws(() => selectModules(matrix, { moduleIds: ['missing'] }), /Unknown module/)
})

test('CLI arguments remain explicit and reject unknown options', () => {
  assert.deepEqual(parseArguments([
    '--module', 'rust-core',
    '--group', 'frontend',
    '--report', 'report.json',
    '--dry-run',
    '--fail-fast',
  ]), {
    moduleIds: ['rust-core'],
    groups: ['frontend'],
    reportPath: 'report.json',
    dryRun: true,
    failFast: true,
    list: false,
    help: false,
  })
  assert.throws(() => parseArguments(['--module']), /requires a value/)
  assert.throws(() => parseArguments(['--wat']), /Unknown argument/)
})

test('Windows command adaptation launches npm through the current Node runtime', () => {
  const windows = spawnSpecForPlatform('npm', ['run', 'build'], {
    platform: 'win32',
    nodeExecutable: 'C:\\Node\\node.exe',
  })
  assert.equal(windows.command, 'C:\\Node\\node.exe')
  assert.deepEqual(windows.args, [
    'C:\\Node\\node_modules\\npm\\bin\\npm-cli.js',
    'run',
    'build',
  ])
  assert.deepEqual(
    spawnSpecForPlatform('cargo', ['test'], { platform: 'win32', nodeExecutable: 'node.exe' }),
    { command: 'cargo', args: ['test'] },
  )
  assert.deepEqual(
    spawnSpecForPlatform('npm', ['test'], { platform: 'linux', nodeExecutable: '/usr/bin/node' }),
    { command: 'npm', args: ['test'] },
  )
})

test('report status distinguishes plans from execution evidence', () => {
  assert.equal(aggregateReportStatus([{ status: 'planned' }]), 'planned')
  assert.equal(aggregateReportStatus([{ status: 'passed' }, { status: 'planned' }]), 'passed')
  assert.equal(aggregateReportStatus([{ status: 'passed' }, { status: 'failed' }]), 'failed')
})

test('report ownership rejects unrelated JSON documents', () => {
  assert.equal(
    assertOwnedReportDocument({ schema: 'monogatari-module-test-report/v1' }).schema,
    'monogatari-module-test-report/v1',
  )
  assert.throws(() => assertOwnedReportDocument({ schema: 'some-project-schema/v1' }), /Existing report/)
  assert.throws(() => assertOwnedReportDocument([]), /Existing report/)
})

test('platform constraints are explicit and portable', () => {
  const matrix = validateMatrix({
    schema: 'monogatari-module-test-matrix/v1',
    modules: [validModule({ platforms: ['win32'] })],
  }, repositoryRoot)
  assert.equal(isModuleSupported(matrix.modules[0], 'win32'), true)
  assert.equal(isModuleSupported(matrix.modules[0], 'linux'), false)
  assert.throws(
    () => validateMatrix({
      schema: 'monogatari-module-test-matrix/v1',
      modules: [validModule({ platforms: ['windows'] })],
    }, repositoryRoot),
    /unknown platform/,
  )
})

test('project mirror synchronization makes the packaged data root byte-equivalent', async () => {
  const temp = await mkdtemp(path.join(os.tmpdir(), 'monogatari-project-mirror-'))
  const source = path.join(temp, 'data')
  const mirror = path.join(temp, 'mirror')
  try {
    await mkdir(path.join(source, 'dialogue'), { recursive: true })
    await mkdir(path.join(mirror, 'dialogue'), { recursive: true })
    await writeFile(path.join(source, 'settings.json'), '{"title":"source"}\n')
    await writeFile(path.join(source, 'dialogue', 'intro.json'), '{"id":"intro"}\n')
    await writeFile(path.join(source, '.monogatari-mcp-project.lock'), '')
    await writeFile(path.join(mirror, 'settings.json'), '{"title":"stale"}\n')
    await writeFile(path.join(mirror, 'obsolete.json'), '{}\n')
    await writeFile(path.join(mirror, '.monogatari-mcp-project.lock'), '')

    const before = await projectMirrorDiff(source, mirror)
    assert.deepEqual(before.missing, ['dialogue/intro.json'])
    assert.deepEqual(before.changed, ['settings.json'])
    assert.deepEqual(before.extra, ['.monogatari-mcp-project.lock', 'obsolete.json'])

    const synchronized = await synchronizeProjectMirror(source, mirror)
    assert.equal(synchronized.valid, true)
    assert.equal(synchronized.sourceCount, 2)
    assert.equal(synchronized.mirrorCount, 2)
    await assert.rejects(stat(path.join(mirror, '.monogatari-mcp-project.lock')), { code: 'ENOENT' })
  } finally {
    await rm(temp, { recursive: true, force: true })
  }
})

function validModule(overrides = {}) {
  return {
    id: 'module-one',
    label: 'Module one',
    group: 'test',
    kind: 'unit',
    default: true,
    cwd: '.',
    command: 'node',
    args: ['--version'],
    owner_paths: ['scripts'],
    ...overrides,
  }
}
