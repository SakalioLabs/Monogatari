#!/usr/bin/env node

import { existsSync } from 'node:fs'
import { readFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

import { callMcpStdioTool, McpStdioClientError } from './lib/mcp-stdio-client.mjs'

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')

function usage() {
  return [
    'Usage:',
    '  node scripts/call-monogatari-mcp.mjs --project-root <root> --tool <name> [--arguments-file <json>] [--allow-write]',
    '  node scripts/call-monogatari-mcp.mjs --project-root <root> --tool apply_transaction --transaction-file <json> --expected-precondition-fingerprint <sha256> --allow-write',
    '  Optional: --binary <path> --package-output-dir <dir> --timeout-ms <1000..300000>',
  ].join('\n')
}

function parseArguments(arguments_) {
  const values = {}
  let allowWrite = false
  for (let index = 0; index < arguments_.length; index += 1) {
    const argument = arguments_[index]
    if (argument === '--allow-write') {
      if (allowWrite) throw new Error('Duplicate argument: --allow-write')
      allowWrite = true
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
    '--tool',
    '--arguments-file',
    '--transaction-file',
    '--expected-precondition-fingerprint',
    '--binary',
    '--package-output-dir',
    '--timeout-ms',
  ])
  const unknown = Object.keys(values).find(key => !allowed.has(key))
  if (unknown) throw new Error(`Unknown argument: ${unknown}`)
  for (const required of ['--project-root', '--tool']) {
    if (!values[required]) throw new Error(`Missing required argument: ${required}`)
  }
  if (values['--arguments-file'] && values['--transaction-file']) {
    throw new Error('--arguments-file and --transaction-file are mutually exclusive')
  }
  if (values['--transaction-file']) {
    if (values['--tool'] !== 'apply_transaction') throw new Error('--transaction-file requires --tool apply_transaction')
    if (!values['--expected-precondition-fingerprint']) {
      throw new Error('--transaction-file requires --expected-precondition-fingerprint')
    }
    if (!allowWrite) throw new Error('--transaction-file requires --allow-write')
  } else if (values['--expected-precondition-fingerprint']) {
    throw new Error('--expected-precondition-fingerprint requires --transaction-file')
  }
  return { values, allowWrite }
}

function defaultBinary() {
  const executable = process.platform === 'win32' ? 'monogatari-mcp.exe' : 'monogatari-mcp'
  const candidates = [
    path.join(repositoryRoot, 'rust-engine', 'target', 'debug', executable),
    path.join(repositoryRoot, 'rust-engine', 'target', 'release', executable),
  ]
  return candidates.find(existsSync) || candidates[0]
}

try {
  const { values, allowWrite } = parseArguments(process.argv.slice(2))
  const projectRoot = path.resolve(repositoryRoot, values['--project-root'])
  const binary = values['--binary']
    ? path.resolve(repositoryRoot, values['--binary'])
    : defaultBinary()
  if (!existsSync(binary)) throw new Error(`MCP binary does not exist: ${binary}`)

  let toolArguments = {}
  if (values['--arguments-file']) {
    const argumentsPath = path.resolve(repositoryRoot, values['--arguments-file'])
    toolArguments = JSON.parse(await readFile(argumentsPath, 'utf8'))
    if (toolArguments === null || typeof toolArguments !== 'object' || Array.isArray(toolArguments)) {
      throw new Error('MCP arguments file must contain one JSON object.')
    }
  }
  if (values['--transaction-file']) {
    const transactionPath = path.resolve(repositoryRoot, values['--transaction-file'])
    const transaction = JSON.parse(await readFile(transactionPath, 'utf8'))
    if (transaction === null || typeof transaction !== 'object' || Array.isArray(transaction)) {
      throw new Error('MCP transaction file must contain one JSON object.')
    }
    toolArguments = {
      transaction,
      expected_precondition_fingerprint: values['--expected-precondition-fingerprint'],
    }
  }

  const commandArguments = ['--project-root', projectRoot]
  if (values['--package-output-dir']) {
    commandArguments.push('--package-output-dir', path.resolve(repositoryRoot, values['--package-output-dir']))
  }
  if (allowWrite) commandArguments.push('--allow-write')
  const timeoutMs = values['--timeout-ms'] ? Number(values['--timeout-ms']) : undefined
  const result = await callMcpStdioTool({
    command: binary,
    commandArguments,
    toolName: values['--tool'],
    toolArguments,
    timeoutMs,
    cwd: repositoryRoot,
  })
  process.stdout.write(`${JSON.stringify({
    schema: 'monogatari-mcp-client-call/v1',
    tool: values['--tool'],
    is_error: result.isError === true,
    structured_content: result.structuredContent ?? null,
    content: result.structuredContent == null ? (result.content ?? []) : [],
  }, null, 2)}\n`)
  if (result.isError === true) process.exitCode = 2
} catch (error) {
  const code = error instanceof McpStdioClientError ? error.code : 'arguments_invalid'
  process.stderr.write(`${JSON.stringify({
    schema: 'monogatari-mcp-client-error/v1',
    code,
    message: String(error.message || error),
    details: error.details || null,
  }, null, 2)}\n`)
  process.stderr.write(`${usage()}\n`)
  process.exitCode = 1
}
