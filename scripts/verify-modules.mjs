#!/usr/bin/env node

import { spawn } from 'node:child_process'
import { lstat, readFile, rename, rm, writeFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

import {
  REPORT_SCHEMA,
  aggregateReportStatus,
  assertOwnedReportDocument,
  displayCommand,
  isModuleSupported,
  parseArguments,
  selectModules,
  spawnSpecForPlatform,
  validateMatrix,
} from './lib/module-test-matrix.mjs'

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const matrixPath = path.join(repositoryRoot, 'scripts', 'module-test-matrix.json')

async function main() {
  const options = parseArguments(process.argv.slice(2))
  if (options.help) return printHelp()

  const matrix = validateMatrix(JSON.parse(await readFile(matrixPath, 'utf8')), repositoryRoot)
  const selected = selectModules(matrix, options)
  if (options.list) return printModules(matrix.modules)
  if (selected.length === 0) throw new Error('No modules were selected.')

  const startedAt = new Date()
  const results = []
  for (const module of selected) {
    const result = options.dryRun ? dryRunResult(module) : await runModule(module)
    results.push(result)
    if (options.failFast && result.status === 'failed') break
  }

  const report = {
    schema: REPORT_SCHEMA,
    status: aggregateReportStatus(results),
    started_at: startedAt.toISOString(),
    completed_at: new Date().toISOString(),
    selected_module_count: selected.length,
    completed_module_count: results.length,
    results,
  }
  if (options.reportPath) await writeReport(options.reportPath, report)

  const passed = results.filter((result) => result.status === 'passed').length
  const failed = results.filter((result) => result.status === 'failed').length
  const planned = results.filter((result) => result.status === 'planned').length
  const skipped = results.filter((result) => result.status === 'skipped').length
  console.log(`\n[modules] ${passed} passed, ${failed} failed, ${planned} planned, ${skipped} skipped`)
  if (report.status === 'failed') process.exitCode = 1
}

function printHelp() {
  console.log(`Usage: node scripts/verify-modules.mjs [options]

Options:
  --list                 List all module gates without running them
  --module <id>          Run one module; repeat to select multiple modules
  --group <id>           Run every module in a group; repeat as needed
  --report <path>        Write a machine-readable JSON report atomically
  --dry-run              Resolve and print commands without executing them
  --fail-fast            Stop after the first failed module
  --help                 Show this help

Without selectors, every module marked as default in the matrix runs.`)
}

function printModules(modules) {
  const idWidth = Math.max('MODULE'.length, ...modules.map((module) => module.id.length))
  const groupWidth = Math.max('GROUP'.length, ...modules.map((module) => module.group.length))
  const platformWidth = Math.max('PLATFORM'.length, ...modules.map((module) => platformLabel(module).length))
  console.log(`${'MODULE'.padEnd(idWidth)}  ${'GROUP'.padEnd(groupWidth)}  ${'PLATFORM'.padEnd(platformWidth)}  DEFAULT  COMMAND`)
  for (const module of modules) {
    console.log(`${module.id.padEnd(idWidth)}  ${module.group.padEnd(groupWidth)}  ${platformLabel(module).padEnd(platformWidth)}  ${String(module.default).padEnd(7)}  ${displayCommand(module)}`)
  }
}

async function runModule(module) {
  console.log(`\n[modules] ${module.label} (${module.id})`)
  console.log(`[modules] ${module.cwdRelative}> ${displayCommand(module)}`)
  if (!isModuleSupported(module)) {
    console.log(`[modules] ${module.id} skipped on ${process.platform}`)
    return reportResult(module, 'skipped', 0, null)
  }
  const started = Date.now()
  let exitCode = 1
  let failure = null
  try {
    exitCode = await spawnModule(module)
  } catch (error) {
    failure = error instanceof Error ? error.message : String(error)
  }
  const durationMs = Date.now() - started
  const status = exitCode === 0 && failure === null ? 'passed' : 'failed'
  console.log(`[modules] ${module.id} ${status} in ${(durationMs / 1000).toFixed(1)}s`)
  if (failure) console.error(`[modules] ${failure}`)
  return reportResult(module, status, durationMs, exitCode, failure)
}

function spawnModule(module) {
  return new Promise((resolve, reject) => {
    const spec = spawnSpecForPlatform(module.command, module.args)
    const child = spawn(spec.command, spec.args, {
      cwd: module.cwd,
      env: { ...process.env, ...module.env },
      shell: false,
      stdio: 'inherit',
    })
    child.once('error', reject)
    child.once('exit', (code, signal) => {
      if (signal) reject(new Error(`${module.id} terminated by signal ${signal}.`))
      else resolve(code ?? 1)
    })
  })
}

function dryRunResult(module) {
  console.log(`[modules] planned ${module.id}: ${module.cwdRelative}> ${displayCommand(module)}`)
  return reportResult(module, 'planned', 0, null)
}

function reportResult(module, status, durationMs, exitCode, error = null) {
  const result = {
    id: module.id,
    label: module.label,
    group: module.group,
    kind: module.kind,
    status,
    duration_ms: durationMs,
    exit_code: exitCode,
    cwd: module.cwdRelative,
    command: module.command,
    args: module.args,
    platforms: module.platforms,
    owner_paths: module.ownerPaths,
  }
  if (error) result.error = error
  return result
}

function platformLabel(module) {
  return module.platforms.length === 0 ? 'all' : module.platforms.join(',')
}

async function writeReport(requestedPath, report) {
  const reportPath = path.resolve(repositoryRoot, requestedPath)
  if (path.extname(reportPath).toLowerCase() !== '.json' || /[\u0000-\u001f\u007f]/u.test(requestedPath)) {
    throw new Error('Module report path must be a control-free .json path.')
  }
  const parentMetadata = await lstat(path.dirname(reportPath))
  if (parentMetadata.isSymbolicLink() || !parentMetadata.isDirectory()) {
    throw new Error('Module report parent must be an existing regular directory.')
  }
  try {
    const targetMetadata = await lstat(reportPath)
    if (targetMetadata.isSymbolicLink() || !targetMetadata.isFile()) {
      throw new Error('Existing module report target must be a regular file.')
    }
    assertOwnedReportDocument(JSON.parse(await readFile(reportPath, 'utf8')))
  } catch (error) {
    if (error?.code !== 'ENOENT') throw error
  }
  const stagePath = `${reportPath}.${process.pid}.tmp`
  await writeFile(stagePath, `${JSON.stringify(report, null, 2)}\n`, { encoding: 'utf8', flag: 'wx' })
  try {
    await rename(stagePath, reportPath)
  } catch (error) {
    await rm(stagePath, { force: true })
    throw error
  }
  console.log(`[modules] report: ${reportPath}`)
}

main().catch((error) => {
  console.error(`[modules] ${error.message}`)
  process.exitCode = 1
})
