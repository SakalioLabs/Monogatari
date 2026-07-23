import assert from 'node:assert/strict'
import test from 'node:test'

import { callMcpStdioTool } from '../lib/mcp-stdio-client.mjs'

const fakeServer = String.raw`
let buffer = ''
let initialized = false
process.stdin.setEncoding('utf8')
process.stdin.on('data', chunk => {
  buffer += chunk
  let newline = buffer.indexOf('\n')
  while (newline >= 0) {
    const line = buffer.slice(0, newline).trim()
    buffer = buffer.slice(newline + 1)
    if (line) {
      const message = JSON.parse(line)
      if (message.method === 'initialize') {
        process.stdout.write(JSON.stringify({
          jsonrpc: '2.0',
          id: message.id,
          result: { protocolVersion: '2024-11-05', capabilities: {}, serverInfo: { name: 'fake', version: '1' } },
        }) + '\n')
      } else if (message.method === 'notifications/initialized') {
        initialized = true
      } else if (message.method === 'tools/call') {
        process.stdout.write(JSON.stringify({
          jsonrpc: '2.0',
          id: message.id,
          result: {
            isError: false,
            structuredContent: { initialized, text: message.params.arguments.text },
            content: [],
          },
        }) + '\n')
      }
    }
    newline = buffer.indexOf('\n')
  }
})
`

test('stdio client performs the MCP handshake and preserves UTF-8 tool arguments', async () => {
  const result = await callMcpStdioTool({
    command: process.execPath,
    commandArguments: ['-e', fakeServer],
    toolName: 'echo',
    toolArguments: { text: '潮镜：蓝色定格' },
    timeoutMs: 15_000,
  })

  assert.equal(result.isError, false)
  assert.deepEqual(result.structuredContent, {
    initialized: true,
    text: '潮镜：蓝色定格',
  })
})

test('stdio client keeps unresponsive MCP requests bounded', async () => {
  await assert.rejects(
    callMcpStdioTool({
      command: process.execPath,
      commandArguments: ['-e', 'setInterval(() => {}, 1000)'],
      toolName: 'never-replies',
      timeoutMs: 1_000,
    }),
    error => error?.code === 'request_timeout'
      && error.message.includes("MCP request 'initialize' timed out after 1000ms."),
  )
})
