import { readdir, readFile as readFileFromDisk } from 'node:fs/promises'
import path from 'node:path'

const commandAttributePattern = /#\s*\[\s*tauri::command(?:\s*\([^\]]*\))?\s*\]/g
const commandDeclarationPattern = /#\s*\[\s*tauri::command(?:\s*\([^\]]*\))?\s*\]\s*(?:pub(?:\s*\([^)]*\))?\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)/g
const registeredCommandPattern = /^commands(?:::[A-Za-z_][A-Za-z0-9_]*){2,}$/

export async function collectTauriCommandRegistrationEvidence(options = {}) {
  const { tauriAppDirectory } = resolveBoundaries(options)
  const readFile = options.readTextFile ?? readFileFromDisk
  const sourceDirectory = path.join(tauriAppDirectory, 'src')
  const commandFiles = await rustSourceFiles(sourceDirectory)
  const issues = []
  const declarations = []
  let commandFileCount = 0

  for (const filePath of commandFiles) {
    const source = await readFile(filePath, 'utf8')
    const modulePath = rustModulePath(sourceDirectory, filePath)
    try {
      const fileDeclarations = extractTauriCommandDeclarations(source, modulePath)
      if (fileDeclarations.length > 0) commandFileCount += 1
      declarations.push(...fileDeclarations)
      if (fileDeclarations.length > 0 && !modulePath.startsWith('commands::')) {
        issues.push(`Tauri command declarations must live under commands/: ${modulePath}`)
      }
    } catch (error) {
      issues.push(`Tauri command declarations could not be parsed in ${modulePath}: ${error.message}`)
    }
  }

  const mainPath = path.join(sourceDirectory, 'main.rs')
  const mainSource = await readFile(mainPath, 'utf8')
  let registrations = []
  try {
    registrations = extractTauriCommandRegistrations(mainSource)
  } catch (error) {
    issues.push(`Tauri generate_handler registration could not be parsed: ${error.message}`)
  }

  for (const command of duplicateValues(declarations)) {
    issues.push(`Tauri command declarations duplicate command ${command}`)
  }
  for (const command of duplicateValues(registrations)) {
    issues.push(`Tauri command registration duplicates command ${command}`)
  }

  const declaredSet = new Set(declarations)
  const registeredSet = new Set(registrations)
  for (const command of [...declaredSet].filter((entry) => !registeredSet.has(entry)).sort()) {
    issues.push(`Tauri command registration is missing declared command ${command}`)
  }
  for (const command of [...registeredSet].filter((entry) => !declaredSet.has(entry)).sort()) {
    issues.push(`Tauri command registration references undeclared command ${command}`)
  }

  if (!mainSource.includes('tauri_plugin_dialog::init()')) {
    issues.push('Tauri command registration must initialize the native project package dialog plugin')
  }

  const capabilityPath = path.join(tauriAppDirectory, 'capabilities', 'default.json')
  const capabilitySource = await readFile(capabilityPath, 'utf8')
  let permissions = []
  try {
    const capability = JSON.parse(capabilitySource)
    if (!Array.isArray(capability.permissions)) {
      issues.push('Tauri default capability permissions must be an array')
    } else {
      permissions = capability.permissions
    }
  } catch {
    issues.push('Tauri default capability must contain valid JSON')
  }
  for (const permission of ['dialog:allow-open', 'dialog:allow-save']) {
    if (!permissions.includes(permission)) {
      issues.push(`Tauri command capability must include ${permission}`)
    }
  }

  return {
    issues,
    declaredCount: declarations.length,
    registeredCount: registrations.length,
    commandFileCount,
  }
}

export function extractTauriCommandDeclarations(source, modulePath) {
  if (typeof source !== 'string') throw new Error('command source must be a string')
  if (typeof modulePath !== 'string' || modulePath.length === 0) {
    throw new Error('command module path is required')
  }
  const markerCount = [...source.matchAll(commandAttributePattern)].length
  const declarations = [...source.matchAll(commandDeclarationPattern)].map(
    (match) => `${modulePath}::${match[1]}`,
  )
  if (declarations.length !== markerCount) {
    throw new Error(`parsed ${declarations.length} of ${markerCount} #[tauri::command] declaration(s)`)
  }
  return declarations
}

export function extractTauriCommandRegistrations(source) {
  if (typeof source !== 'string') throw new Error('main source must be a string')
  const markers = [...source.matchAll(/tauri::generate_handler!\s*\[/g)]
  if (markers.length !== 1) {
    throw new Error(`expected one tauri::generate_handler! block, found ${markers.length}`)
  }
  const start = markers[0].index + markers[0][0].length
  let depth = 1
  let end = -1
  for (let index = start; index < source.length; index += 1) {
    if (source[index] === '[') depth += 1
    if (source[index] === ']') depth -= 1
    if (depth === 0) {
      end = index
      break
    }
  }
  if (end < 0) throw new Error('tauri::generate_handler! block is not closed')

  const entries = source
    .slice(start, end)
    .split(',')
    .map((entry) => entry.trim())
    .filter(Boolean)
  for (const entry of entries) {
    if (!registeredCommandPattern.test(entry)) {
      throw new Error(`unsupported generate_handler entry ${entry}`)
    }
  }
  return entries
}

function resolveBoundaries(options) {
  if (typeof options.tauriAppDirectory !== 'string' || options.tauriAppDirectory.length === 0) {
    throw new Error('Tauri command registration policy requires tauriAppDirectory.')
  }
  return { tauriAppDirectory: options.tauriAppDirectory }
}

async function rustSourceFiles(root) {
  const files = []
  async function visit(directory) {
    const entries = await readdir(directory, { withFileTypes: true })
    entries.sort((left, right) => left.name.localeCompare(right.name))
    for (const entry of entries) {
      const entryPath = path.join(directory, entry.name)
      if (entry.isDirectory()) {
        await visit(entryPath)
      } else if (entry.isFile() && entry.name.endsWith('.rs')) {
        files.push(entryPath)
      }
    }
  }
  await visit(root)
  return files
}

function rustModulePath(sourceDirectory, filePath) {
  const relative = path.relative(sourceDirectory, filePath).replaceAll(path.sep, '/')
  const parts = relative.replace(/\.rs$/, '').split('/')
  if (parts.at(-1) === 'mod') parts.pop()
  return parts.join('::')
}

function duplicateValues(values) {
  const counts = new Map()
  for (const value of values) counts.set(value, (counts.get(value) ?? 0) + 1)
  return [...counts.entries()]
    .filter(([, count]) => count > 1)
    .map(([value]) => value)
    .sort()
}
