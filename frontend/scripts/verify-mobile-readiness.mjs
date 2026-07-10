import { readFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const frontendDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const repoRoot = path.resolve(frontendDir, '..')
const issues = []

const [indexHtml, globalCss, appSource, manifest, packageJson, tauriConfig] = await Promise.all([
  readText(path.join(frontendDir, 'index.html')),
  readText(path.join(frontendDir, 'src', 'styles', 'main.css')),
  readText(path.join(frontendDir, 'src', 'App.vue')),
  readJson(path.join(frontendDir, 'public', 'manifest.webmanifest')),
  readJson(path.join(frontendDir, 'package.json')),
  readJson(path.join(repoRoot, 'rust-engine', 'crates', 'tauri-app', 'tauri.conf.json')),
])

requireIncludes(indexHtml, 'width=device-width', 'index.html viewport must fit device width')
requireIncludes(indexHtml, 'initial-scale=1.0', 'index.html viewport must set initial-scale=1.0')
requireIncludes(indexHtml, 'viewport-fit=cover', 'index.html viewport must enable safe-area layout with viewport-fit=cover')
requireIncludes(indexHtml, 'apple-mobile-web-app-capable', 'index.html must include iOS standalone PWA capability')
requireIncludes(indexHtml, 'apple-mobile-web-app-status-bar-style', 'index.html must define iOS status bar behavior')
requireIncludes(indexHtml, 'theme-color', 'index.html must define browser theme color')
requireIncludes(indexHtml, 'rel="manifest"', 'index.html must link the Web/PWA manifest')
requireIncludes(indexHtml, 'rel="apple-touch-icon"', 'index.html must link an Apple touch icon')

if (manifest.display !== 'standalone') {
  issues.push('manifest.webmanifest display must be standalone for app-like mobile installs')
}
if (!manifest.start_url || !manifest.scope) {
  issues.push('manifest.webmanifest must define start_url and scope')
}
const iconPurposes = new Set((manifest.icons ?? []).flatMap((icon) => String(icon.purpose ?? 'any').split(/\s+/)))
for (const purpose of ['any', 'maskable']) {
  if (!iconPurposes.has(purpose)) {
    issues.push(`manifest.webmanifest must include a ${purpose} icon purpose`)
  }
}

for (const needle of [
  '100svh',
  '-webkit-tap-highlight-color',
  'touch-action: manipulation',
  'overscroll-behavior',
]) {
  requireIncludes(globalCss, needle, `src/styles/main.css must include mobile readiness rule: ${needle}`)
}

for (const needle of [
  '@media (max-width: 860px)',
  'calc(60px + env(safe-area-inset-bottom',
  'calc(56px + env(safe-area-inset-bottom',
  'grid-template-columns: repeat(5, minmax(0, 1fr))',
]) {
  requireIncludes(appSource, needle, `src/App.vue must include compact shell rule: ${needle}`)
}

const mainWindow = tauriConfig.app?.windows?.[0] ?? {}
if ((mainWindow.minWidth ?? Number.POSITIVE_INFINITY) > 390) {
  issues.push(`tauri.conf.json app.windows[0].minWidth must be <= 390 for compact mobile/webview shells, got ${mainWindow.minWidth}`)
}
if ((mainWindow.minHeight ?? Number.POSITIVE_INFINITY) > 640) {
  issues.push(`tauri.conf.json app.windows[0].minHeight must be <= 640 for compact mobile/webview shells, got ${mainWindow.minHeight}`)
}
if (tauriConfig.identifier !== 'com.sakaliolabs.monogatari') {
  issues.push('tauri.conf.json identifier must remain stable for mobile bundle identity')
}
if (!String(tauriConfig.build?.frontendDist ?? '').includes('frontend/dist')) {
  issues.push('tauri.conf.json build.frontendDist must keep using the production Web/PWA dist')
}

if (!String(packageJson.scripts?.['verify:mobile-readiness'] ?? '').includes('verify-mobile-readiness.mjs')) {
  issues.push('package.json must expose verify:mobile-readiness')
}

if (issues.length > 0) {
  throw new Error(`Mobile readiness verification failed:\n${issues.join('\n')}`)
}

console.log('[mobile-readiness] OK: viewport, PWA metadata, safe-area CSS, and compact Tauri shell limits verified')

async function readText(file) {
  return readFile(file, 'utf8')
}

async function readJson(file) {
  return JSON.parse(await readText(file))
}

function requireIncludes(source, needle, message) {
  if (!source.includes(needle)) {
    issues.push(message)
  }
}
