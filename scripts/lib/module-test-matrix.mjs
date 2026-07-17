import { statSync } from 'node:fs'
import path from 'node:path'

export const MATRIX_SCHEMA = 'monogatari-module-test-matrix/v1'
export const REPORT_SCHEMA = 'monogatari-module-test-report/v1'

const idPattern = /^[a-z0-9]+(?:-[a-z0-9]+)*$/
const envKeyPattern = /^[A-Za-z_][A-Za-z0-9_]*$/
const maxModules = 128
const maxArguments = 128
const maxOwners = 128
const maxEnvironmentEntries = 32
const knownPlatforms = new Set(['darwin', 'linux', 'win32'])

export function validateMatrix(value, repositoryRoot) {
  if (!isRecord(value) || value.schema !== MATRIX_SCHEMA || !Array.isArray(value.modules)) {
    throw new Error(`Module matrix must use schema ${MATRIX_SCHEMA} and contain a modules array.`)
  }
  if (value.modules.length === 0) throw new Error('Module matrix must contain at least one module.')
  if (value.modules.length > maxModules) throw new Error(`Module matrix cannot exceed ${maxModules} modules.`)

  const root = path.resolve(repositoryRoot)
  const ids = new Set()
  const modules = value.modules.map((entry, index) => validateModule(entry, index, root, ids))
  return { schema: MATRIX_SCHEMA, modules }
}

export function selectModules(matrix, { moduleIds = [], groups = [] } = {}) {
  const requestedIds = new Set(moduleIds)
  const requestedGroups = new Set(groups)
  const knownIds = new Set(matrix.modules.map((module) => module.id))
  const knownGroups = new Set(matrix.modules.map((module) => module.group))

  for (const id of requestedIds) {
    if (!knownIds.has(id)) throw new Error(`Unknown module: ${id}`)
  }
  for (const group of requestedGroups) {
    if (!knownGroups.has(group)) throw new Error(`Unknown module group: ${group}`)
  }

  const hasSelectors = requestedIds.size > 0 || requestedGroups.size > 0
  return matrix.modules.filter((module) => hasSelectors
    ? requestedIds.has(module.id) || requestedGroups.has(module.group)
    : module.default)
}

export function parseArguments(args) {
  const options = {
    moduleIds: [],
    groups: [],
    reportPath: null,
    dryRun: false,
    failFast: false,
    list: false,
    help: false,
  }

  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index]
    if (arg === '--module') options.moduleIds.push(requiredValue(args, ++index, arg))
    else if (arg === '--group') options.groups.push(requiredValue(args, ++index, arg))
    else if (arg === '--report') options.reportPath = requiredValue(args, ++index, arg)
    else if (arg === '--dry-run') options.dryRun = true
    else if (arg === '--fail-fast') options.failFast = true
    else if (arg === '--list') options.list = true
    else if (arg === '--help' || arg === '-h') options.help = true
    else throw new Error(`Unknown argument: ${arg}`)
  }

  return options
}

export function spawnSpecForPlatform(
  command,
  args,
  { platform = process.platform, nodeExecutable = process.execPath } = {},
) {
  if (platform === 'win32' && ['npm', 'npx'].includes(command)) {
    const cliPath = path.win32.join(
      path.win32.dirname(nodeExecutable),
      'node_modules',
      'npm',
      'bin',
      `${command}-cli.js`,
    )
    return { command: nodeExecutable, args: [cliPath, ...args] }
  }
  return { command, args: [...args] }
}

export function displayCommand(module) {
  return [module.command, ...module.args].map(quoteArgument).join(' ')
}

export function isModuleSupported(module, platform = process.platform) {
  return module.platforms.length === 0 || module.platforms.includes(platform)
}

export function aggregateReportStatus(results) {
  if (results.some((result) => result.status === 'failed')) return 'failed'
  if (results.length > 0 && results.every((result) => result.status === 'planned')) return 'planned'
  return 'passed'
}

export function assertOwnedReportDocument(value) {
  if (!isRecord(value) || value.schema !== REPORT_SCHEMA) {
    throw new Error(`Existing report must use schema ${REPORT_SCHEMA}.`)
  }
  return value
}

function validateModule(value, index, root, ids) {
  const label = `modules[${index}]`
  if (!isRecord(value)) throw new Error(`${label} must be an object.`)
  if (!boundedString(value.id, 64) || !idPattern.test(value.id)) {
    throw new Error(`${label}.id must be a lowercase kebab-case ID of at most 64 characters.`)
  }
  if (ids.has(value.id)) throw new Error(`Duplicate module ID: ${value.id}`)
  ids.add(value.id)

  if (!boundedString(value.label, 160)) throw new Error(`${label}.label must be 160 characters or fewer.`)
  if (!boundedString(value.group, 64) || !boundedString(value.kind, 64)) {
    throw new Error(`${label}.group and ${label}.kind must be 64 characters or fewer.`)
  }
  if (!boundedString(value.cwd, 512)) throw new Error(`${label}.cwd must be 512 characters or fewer.`)
  if (!boundedString(value.command, 64)) throw new Error(`${label}.command must be 64 characters or fewer.`)
  if (!/^[A-Za-z0-9_.-]+$/.test(value.command)) {
    throw new Error(`${label}.command must be a portable executable name.`)
  }
  if (!idPattern.test(value.group) || !idPattern.test(value.kind)) {
    throw new Error(`${label}.group and ${label}.kind must be lowercase kebab-case IDs.`)
  }
  if (typeof value.default !== 'boolean') throw new Error(`${label}.default must be a boolean.`)
  if (!Array.isArray(value.args) || value.args.length > maxArguments
    || value.args.some((arg) => !boundedString(arg, 2048, false))) {
    throw new Error(`${label}.args must contain at most ${maxArguments} bounded strings without control characters.`)
  }
  if (!Array.isArray(value.owner_paths) || value.owner_paths.length === 0 || value.owner_paths.length > maxOwners) {
    throw new Error(`${label}.owner_paths must contain between 1 and ${maxOwners} repository-relative paths.`)
  }

  const cwd = resolveInsideRoot(root, value.cwd, `${label}.cwd`)
  requireExistingPath(cwd, `${label}.cwd`, true)
  const ownerPaths = value.owner_paths.map((ownerPath, ownerIndex) => {
    if (!boundedString(ownerPath, 512)) throw new Error(`${label}.owner_paths[${ownerIndex}] must be a bounded safe string.`)
    const resolvedOwner = resolveInsideRoot(root, ownerPath, `${label}.owner_paths[${ownerIndex}]`)
    requireExistingPath(resolvedOwner, `${label}.owner_paths[${ownerIndex}]`)
    return ownerPath.replaceAll('\\', '/')
  })

  const env = value.env === undefined ? {} : validateEnvironment(value.env, label)
  const platforms = validatePlatforms(value.platforms, label)
  return {
    id: value.id,
    label: value.label,
    group: value.group,
    kind: value.kind,
    default: value.default,
    cwd,
    cwdRelative: value.cwd.replaceAll('\\', '/'),
    command: value.command,
    args: [...value.args],
    env,
    platforms,
    ownerPaths,
  }
}

function validatePlatforms(value, label) {
  if (value === undefined) return []
  if (!Array.isArray(value) || value.length === 0 || value.length > knownPlatforms.size) {
    throw new Error(`${label}.platforms must be a non-empty array of supported platform IDs.`)
  }
  const platforms = [...new Set(value)]
  if (platforms.length !== value.length || platforms.some((platform) => !knownPlatforms.has(platform))) {
    throw new Error(`${label}.platforms contains duplicate or unknown platform IDs.`)
  }
  return platforms
}

function validateEnvironment(value, label) {
  if (!isRecord(value)) throw new Error(`${label}.env must be an object.`)
  const entries = Object.entries(value)
  if (entries.length > maxEnvironmentEntries) {
    throw new Error(`${label}.env cannot exceed ${maxEnvironmentEntries} entries.`)
  }
  return Object.fromEntries(entries.map(([key, entry]) => {
    if (!envKeyPattern.test(key) || !boundedString(entry, 2048, false)) {
      throw new Error(`${label}.env must contain safe string values under portable environment keys.`)
    }
    return [key, entry]
  }))
}

function resolveInsideRoot(root, relativePath, label) {
  if (!safeString(relativePath) || path.isAbsolute(relativePath)) {
    throw new Error(`${label} must be a safe repository-relative path.`)
  }
  const resolved = path.resolve(root, relativePath)
  const relative = path.relative(root, resolved)
  if (relative === '..' || relative.startsWith(`..${path.sep}`) || path.isAbsolute(relative)) {
    throw new Error(`${label} escapes the repository root.`)
  }
  return resolved
}

function requiredValue(args, index, option) {
  const value = args[index]
  if (!value || value.startsWith('--')) throw new Error(`${option} requires a value.`)
  return value
}

function requireExistingPath(target, label, directory = false) {
  let metadata
  try {
    metadata = statSync(target)
  } catch {
    throw new Error(`${label} does not exist.`)
  }
  if (directory && !metadata.isDirectory()) throw new Error(`${label} must be a directory.`)
}

function quoteArgument(value) {
  return /^[A-Za-z0-9_./:=@+-]+$/.test(value) ? value : JSON.stringify(value)
}

function boundedString(value, maxLength, requireNonEmpty = true) {
  return safeString(value)
    && value.length <= maxLength
    && (!requireNonEmpty || value.trim().length > 0)
}

function safeString(value) {
  return typeof value === 'string' && !/[\u0000-\u001f\u007f]/u.test(value)
}

function isRecord(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value)
}
