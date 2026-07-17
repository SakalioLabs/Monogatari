#!/usr/bin/env node

import path from 'node:path'
import { fileURLToPath } from 'node:url'

import {
  applyProjectAssetImport,
  planProjectAssetImport,
  ProjectAssetImportError,
} from './lib/project-asset-import.mjs'

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')

function usage() {
  return [
    'Usage:',
    '  node scripts/import-project-asset.mjs --project-root <root> --source <file> --destination <assets/path> --precondition <missing|sha256>',
    '  node scripts/import-project-asset.mjs ... --write --expected-plan-fingerprint <sha256>',
  ].join('\n')
}

function parseArguments(arguments_) {
  const values = {}
  let write = false
  for (let index = 0; index < arguments_.length; index += 1) {
    const argument = arguments_[index]
    if (argument === '--write') {
      write = true
      continue
    }
    if (!argument.startsWith('--')) throw new Error(`Unexpected argument: ${argument}`)
    const value = arguments_[index + 1]
    if (!value || value.startsWith('--')) throw new Error(`Missing value for ${argument}`)
    if (Object.hasOwn(values, argument)) throw new Error(`Duplicate argument: ${argument}`)
    values[argument] = value
    index += 1
  }
  const allowed = new Set([
    '--project-root',
    '--source',
    '--destination',
    '--precondition',
    '--expected-plan-fingerprint',
  ])
  const unknown = Object.keys(values).find(key => !allowed.has(key))
  if (unknown) throw new Error(`Unknown argument: ${unknown}`)
  for (const required of ['--project-root', '--source', '--destination', '--precondition']) {
    if (!values[required]) throw new Error(`Missing required argument: ${required}`)
  }
  if (!write && values['--expected-plan-fingerprint']) {
    throw new Error('--expected-plan-fingerprint is accepted only with --write')
  }
  return { values, write }
}

try {
  const { values, write } = parseArguments(process.argv.slice(2))
  const input = {
    projectRoot: path.resolve(repositoryRoot, values['--project-root']),
    sourcePath: path.resolve(values['--source']),
    destinationPath: values['--destination'],
    precondition: values['--precondition'],
    expectedPlanFingerprint: values['--expected-plan-fingerprint'],
  }
  const result = write
    ? await applyProjectAssetImport(input)
    : await planProjectAssetImport(input)
  process.stdout.write(`${JSON.stringify(result, null, 2)}\n`)
} catch (error) {
  const code = error instanceof ProjectAssetImportError ? error.code : 'arguments_invalid'
  process.stderr.write(`${JSON.stringify({ code, message: String(error.message || error) }, null, 2)}\n`)
  process.stderr.write(`${usage()}\n`)
  process.exitCode = 1
}
