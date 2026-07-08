import { readFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const frontendDir = path.join(root, 'frontend')
const tauriAppDir = path.join(root, 'rust-engine', 'crates', 'tauri-app')
const issues = []

const [viteConfig, frontendPackage, tauriConfig, tauriCargo, mobileDocs] = await Promise.all([
  readText(path.join(frontendDir, 'vite.config.ts')),
  readJson(path.join(frontendDir, 'package.json')),
  readJson(path.join(tauriAppDir, 'tauri.conf.json')),
  readText(path.join(tauriAppDir, 'Cargo.toml')),
  readText(path.join(root, 'docs', 'MOBILE_DEPLOYMENT.md')),
])

const buildConfig = tauriConfig.build ?? {}
const mainWindow = tauriConfig.app?.windows?.[0] ?? {}
const bundleIcons = tauriConfig.bundle?.icon ?? []
const packageScripts = frontendPackage.scripts ?? {}

requireMatch(
  tauriConfig.identifier,
  /^com\.sakaliolabs\.monogatari$/,
  'tauri.conf.json identifier must stay stable for Android/iOS bundle identity',
)
requireIncludes(buildConfig.frontendDist, 'frontend/dist', 'tauri.conf.json build.frontendDist must point at the production frontend dist')
requireIncludes(buildConfig.devUrl, 'localhost:5173', 'tauri.conf.json build.devUrl must match the Vite mobile dev port')
requireIncludes(buildConfig.beforeDevCommand, 'npm run dev', 'tauri.conf.json build.beforeDevCommand must start the Vite dev server')
requireIncludes(buildConfig.beforeBuildCommand, 'npm run build', 'tauri.conf.json build.beforeBuildCommand must run the production frontend gate')

if ((mainWindow.minWidth ?? Number.POSITIVE_INFINITY) > 390) {
  issues.push(`tauri.conf.json app.windows[0].minWidth must be <= 390 for compact mobile shells, got ${mainWindow.minWidth}`)
}
if ((mainWindow.minHeight ?? Number.POSITIVE_INFINITY) > 640) {
  issues.push(`tauri.conf.json app.windows[0].minHeight must be <= 640 for compact mobile shells, got ${mainWindow.minHeight}`)
}

for (const icon of ['icons/icon_128x128.png', 'icons/icon_256x256.png', 'icons/icon_512x512.png']) {
  if (!bundleIcons.includes(icon)) {
    issues.push(`tauri.conf.json bundle.icon must include ${icon} for mobile launcher icon generation`)
  }
}

for (const [scriptName, expected] of [
  ['build', 'verify-responsive-shell.mjs'],
  ['build:web', 'verify-responsive-shell.mjs'],
  ['verify:mobile-readiness', 'verify-mobile-readiness.mjs'],
  ['verify:responsive-shell', 'verify-responsive-shell.mjs'],
]) {
  requireIncludes(packageScripts[scriptName], expected, `frontend/package.json ${scriptName} must include ${expected}`)
}

for (const [needle, description] of [
  ['const mobileDevHost = process.env.TAURI_DEV_HOST', 'read the Tauri mobile dev host from the environment'],
  ['host: mobileDevHost || false', 'bind Vite to the Tauri-selected mobile host'],
  ['hmr: mobileDevHost', 'configure mobile HMR only when Tauri provides a host'],
  ["protocol: 'ws'", 'use a WebSocket HMR protocol for device previews'],
  ['port: 5174', 'reserve a stable HMR port next to the dev server'],
  ["envPrefix: ['VITE_', 'TAURI_']", 'allow TAURI_* environment values in Vite'],
]) {
  requireIncludes(viteConfig, needle, `frontend/vite.config.ts must ${description}`)
}

for (const [needle, description] of [
  ['tauri = { version = "2"', 'pin the Tauri app crate to the v2 mobile-capable line'],
  ['tauri-plugin-shell = "2"', 'pin shell plugin compatibility to the v2 line'],
]) {
  requireIncludes(tauriCargo, needle, `tauri-app Cargo.toml must ${description}`)
}

for (const [needle, description] of [
  ['cargo tauri android init', 'document Android project initialization'],
  ['cargo tauri android dev', 'document Android device development'],
  ['cargo tauri android build', 'document Android release builds'],
  ['cargo tauri android run', 'document Android production runs'],
  ['cargo tauri ios init', 'document iOS project initialization'],
  ['cargo tauri ios dev', 'document iOS device development'],
  ['cargo tauri ios build', 'document iOS release builds'],
  ['cargo tauri ios run', 'document iOS production runs'],
  ['TAURI_DEV_HOST', 'document the mobile dev host contract'],
  ['ANDROID_HOME', 'document Android SDK environment setup'],
  ['NDK_HOME', 'document Android NDK environment setup'],
  ['iOS commands require a macOS host', 'document the iOS host constraint'],
  ['cargo tauri ios dev --open --host', 'document physical iOS host forwarding'],
  ['node scripts/verify-tauri-mobile-preflight.mjs', 'document the preflight evidence command'],
]) {
  requireIncludes(mobileDocs, needle, `docs/MOBILE_DEPLOYMENT.md must ${description}`)
}

if (issues.length > 0) {
  throw new Error(`Tauri mobile preflight failed:\n${issues.join('\n')}`)
}

console.log('[tauri-mobile] OK: SDK-free Android/iOS preflight, Vite mobile host, and Tauri shell config verified')

async function readText(file) {
  return readFile(file, 'utf8')
}

async function readJson(file) {
  return JSON.parse(await readText(file))
}

function requireIncludes(source, needle, message) {
  if (!String(source ?? '').includes(needle)) {
    issues.push(message)
  }
}

function requireMatch(value, pattern, message) {
  if (!pattern.test(String(value ?? ''))) {
    issues.push(message)
  }
}
