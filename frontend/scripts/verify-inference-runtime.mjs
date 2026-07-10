import { readFile, readdir, stat } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const scriptDir = path.dirname(fileURLToPath(import.meta.url))
const frontendDir = path.resolve(scriptDir, '..')
const distDir = path.join(frontendDir, 'dist')
const runtime = JSON.parse(await readFile(path.join(distDir, 'inference-runtime.json'), 'utf8'))
const headers = await readFile(path.join(distDir, '_headers'), 'utf8')
const serviceWorker = await readFile(path.join(distDir, 'sw.js'), 'utf8')
const failures = []
const ortRuntimeFiles = [
  'ort/ort-wasm-simd-threaded.jsep.mjs',
]

if (runtime.schema !== 'monogatari-inference-runtime/v1') failures.push('runtime schema is invalid')
if (runtime.target !== 'web') failures.push('runtime target must be web')
if (runtime.backend !== 'webgpu') failures.push('web package backend must be webgpu')
if (typeof runtime.model_id !== 'string' || !runtime.model_id.trim()) failures.push('model_id is required')
if (!['q4', 'q4f16', 'q8', 'fp16', 'fp32'].includes(runtime.dtype)) failures.push('dtype is unsupported')
if (!Number.isInteger(runtime.max_new_tokens) || runtime.max_new_tokens < 1 || runtime.max_new_tokens > 2048) {
  failures.push('max_new_tokens must be an integer from 1 to 2048')
}
if (!headers.includes("script-src 'self' 'wasm-unsafe-eval'")) failures.push('CSP does not allow ONNX WebAssembly bootstrap')
if (!serviceWorker.includes('INFERENCE_RUNTIME_PATH')) failures.push('service worker does not cache the runtime contract')
for (const file of ortRuntimeFiles) {
  try {
    const fileStat = await stat(path.join(distDir, file))
    if (!fileStat.isFile() || fileStat.size === 0) failures.push(`${file} is empty`)
  } catch {
    failures.push(`${file} is missing`)
  }
}
const bundledAssets = await readdir(path.join(distDir, 'assets'))
if (!bundledAssets.some((file) => /^ort-wasm-simd-threaded\.jsep-[A-Za-z0-9_-]+\.wasm$/.test(file))) {
  failures.push('the bundled WebGPU ONNX runtime WASM asset is missing')
}
if (Object.keys(runtime).some((key) => /^(api[_-]?key|password|access[_-]?token|secret)$/i.test(key))) {
  failures.push('runtime contract contains a secret-shaped key')
}

if (failures.length > 0) {
  console.error(`[inference-runtime] Failed:\n- ${failures.join('\n- ')}`)
  process.exit(1)
}

console.log(`[inference-runtime] OK: WebGPU ${runtime.model_id} (${runtime.dtype}, ${runtime.max_new_tokens} tokens)`)
