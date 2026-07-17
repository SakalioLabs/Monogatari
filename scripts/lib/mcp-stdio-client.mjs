import { spawn } from 'node:child_process'
import { StringDecoder } from 'node:string_decoder'

const DEFAULT_TIMEOUT_MS = 30_000
const MAX_STDOUT_BYTES = 32 * 1024 * 1024
const MAX_STDERR_BYTES = 256 * 1024

export class McpStdioClientError extends Error {
  constructor(code, message, details = {}) {
    super(message)
    this.name = 'McpStdioClientError'
    this.code = code
    this.details = details
  }
}

export async function callMcpStdioTool({
  command,
  commandArguments = [],
  toolName,
  toolArguments = {},
  timeoutMs = DEFAULT_TIMEOUT_MS,
  cwd,
  environment = process.env,
}) {
  if (typeof command !== 'string' || command.length === 0) {
    throw new McpStdioClientError('command_invalid', 'MCP command is required.')
  }
  if (typeof toolName !== 'string' || toolName.length === 0) {
    throw new McpStdioClientError('tool_invalid', 'MCP tool name is required.')
  }
  if (!isObject(toolArguments)) {
    throw new McpStdioClientError('arguments_invalid', 'MCP tool arguments must be a JSON object.')
  }
  if (!Number.isSafeInteger(timeoutMs) || timeoutMs < 1_000 || timeoutMs > 300_000) {
    throw new McpStdioClientError('timeout_invalid', 'MCP timeout must be an integer from 1000 to 300000 milliseconds.')
  }

  const child = spawn(command, commandArguments, {
    cwd,
    env: environment,
    stdio: ['pipe', 'pipe', 'pipe'],
    windowsHide: true,
  })
  const pending = new Map()
  const decoder = new StringDecoder('utf8')
  let stdoutBuffer = ''
  let stdoutBytes = 0
  let stderr = ''
  let stderrBytes = 0
  let exited = false
  let startupError = null

  const rejectPending = (error) => {
    for (const { reject, timer } of pending.values()) {
      clearTimeout(timer)
      reject(error)
    }
    pending.clear()
  }

  child.on('error', (error) => {
    startupError = error
    rejectPending(new McpStdioClientError('process_start_failed', `Failed to start MCP server: ${error.message}`))
  })
  child.stderr.on('data', (chunk) => {
    if (stderrBytes >= MAX_STDERR_BYTES) return
    const bytes = Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk)
    const remaining = MAX_STDERR_BYTES - stderrBytes
    stderr += bytes.subarray(0, remaining).toString('utf8')
    stderrBytes += Math.min(bytes.length, remaining)
  })
  child.stdout.on('data', (chunk) => {
    const bytes = Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk)
    stdoutBytes += bytes.length
    if (stdoutBytes > MAX_STDOUT_BYTES) {
      const error = new McpStdioClientError('response_too_large', `MCP stdout exceeded ${MAX_STDOUT_BYTES} bytes.`)
      rejectPending(error)
      child.kill()
      return
    }
    stdoutBuffer += decoder.write(bytes)
    let newlineIndex = stdoutBuffer.indexOf('\n')
    while (newlineIndex >= 0) {
      const line = stdoutBuffer.slice(0, newlineIndex).trim()
      stdoutBuffer = stdoutBuffer.slice(newlineIndex + 1)
      if (line.length > 0) handleMessage(line, pending, stderr)
      newlineIndex = stdoutBuffer.indexOf('\n')
    }
  })
  child.on('exit', (code, signal) => {
    exited = true
    const tail = `${stdoutBuffer}${decoder.end()}`.trim()
    if (tail.length > 0) handleMessage(tail, pending, stderr)
    if (pending.size > 0) {
      rejectPending(new McpStdioClientError(
        'process_exited',
        `MCP server exited before replying (code=${code ?? 'null'}, signal=${signal ?? 'none'}).`,
        { stderr: stderr.trim() },
      ))
    }
  })

  let requestId = 0
  const request = (method, params) => new Promise((resolve, reject) => {
    if (startupError) {
      reject(new McpStdioClientError('process_start_failed', `Failed to start MCP server: ${startupError.message}`))
      return
    }
    if (exited || child.stdin.destroyed) {
      reject(new McpStdioClientError('process_exited', 'MCP server is no longer running.', { stderr: stderr.trim() }))
      return
    }
    requestId += 1
    const id = requestId
    const timer = setTimeout(() => {
      pending.delete(id)
      reject(new McpStdioClientError('request_timeout', `MCP request '${method}' timed out after ${timeoutMs}ms.`, {
        stderr: stderr.trim(),
      }))
      child.kill()
    }, timeoutMs)
    pending.set(id, { resolve, reject, timer })
    child.stdin.write(`${JSON.stringify({ jsonrpc: '2.0', id, method, params })}\n`, 'utf8', (error) => {
      if (!error) return
      clearTimeout(timer)
      pending.delete(id)
      reject(new McpStdioClientError('request_write_failed', `Failed to write MCP request: ${error.message}`))
    })
  })

  try {
    await request('initialize', {
      protocolVersion: '2024-11-05',
      capabilities: {},
      clientInfo: { name: 'monogatari-agent-cli', version: '1.0.0' },
    })
    child.stdin.write(`${JSON.stringify({ jsonrpc: '2.0', method: 'notifications/initialized', params: {} })}\n`, 'utf8')
    const response = await request('tools/call', { name: toolName, arguments: toolArguments })
    return response
  } finally {
    await stopChild(child, exited)
  }
}

function handleMessage(line, pending, stderr) {
  let message
  try {
    message = JSON.parse(line)
  } catch (error) {
    const malformed = new McpStdioClientError('response_invalid', `MCP server emitted invalid JSON: ${error.message}`, {
      line: line.slice(0, 1024),
      stderr: stderr.trim(),
    })
    for (const { reject, timer } of pending.values()) {
      clearTimeout(timer)
      reject(malformed)
    }
    pending.clear()
    return
  }
  if (!Object.hasOwn(message, 'id')) return
  const waiter = pending.get(message.id)
  if (!waiter) return
  pending.delete(message.id)
  clearTimeout(waiter.timer)
  if (message.error) {
    waiter.reject(new McpStdioClientError(
      'rpc_error',
      String(message.error.message || 'MCP JSON-RPC request failed.'),
      { rpc_error: message.error, stderr: stderr.trim() },
    ))
    return
  }
  waiter.resolve(message.result)
}

async function stopChild(child, alreadyExited) {
  if (alreadyExited) return
  child.stdin.end()
  await new Promise((resolve) => {
    const timer = setTimeout(() => {
      child.kill()
      resolve()
    }, 1_000)
    child.once('exit', () => {
      clearTimeout(timer)
      resolve()
    })
  })
}

function isObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value)
}
