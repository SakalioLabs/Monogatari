import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
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

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..')

test('checked-in matrix is valid and owns every primary implementation surface', async () => {
  const source = JSON.parse(await readFile(path.join(repositoryRoot, 'scripts', 'module-test-matrix.json'), 'utf8'))
  const matrix = validateMatrix(source, repositoryRoot)
  const ownerPaths = new Set(matrix.modules.flatMap((module) => module.ownerPaths))

  for (const requiredPath of [
    'frontend/src',
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
