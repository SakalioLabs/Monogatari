import { createHash } from 'node:crypto'
import { copyFile, mkdir, readdir, readFile, rm, stat } from 'node:fs/promises'
import path from 'node:path'

const SOURCE_TRANSIENT_PATHS = new Set(['.monogatari-mcp-project.lock'])

export async function projectMirrorDiff(sourceRoot, mirrorRoot) {
  const [source, mirror] = await Promise.all([
    fileInventory(sourceRoot, { ignoredPaths: SOURCE_TRANSIENT_PATHS }),
    fileInventory(mirrorRoot, { missingRootIsEmpty: true }),
  ])
  const missing = []
  const changed = []
  const extra = []

  for (const [relativePath, sourceFile] of source) {
    const mirrorFile = mirror.get(relativePath)
    if (!mirrorFile) missing.push(relativePath)
    else if (sourceFile.sha256 !== mirrorFile.sha256 || sourceFile.size !== mirrorFile.size) {
      changed.push(relativePath)
    }
  }
  for (const relativePath of mirror.keys()) {
    if (!source.has(relativePath)) extra.push(relativePath)
  }

  return {
    sourceCount: source.size,
    mirrorCount: mirror.size,
    missing: missing.sort(),
    changed: changed.sort(),
    extra: extra.sort(),
    valid: missing.length === 0 && changed.length === 0 && extra.length === 0,
  }
}

export async function synchronizeProjectMirror(sourceRoot, mirrorRoot) {
  const source = path.resolve(sourceRoot)
  const mirror = path.resolve(mirrorRoot)
  if (source === mirror || mirror.startsWith(`${source}${path.sep}`) || source.startsWith(`${mirror}${path.sep}`)) {
    throw new Error('Project mirror roots must be distinct and must not contain one another.')
  }

  const sourceFiles = await fileInventory(source, { ignoredPaths: SOURCE_TRANSIENT_PATHS })
  const mirrorFiles = await fileInventory(mirror, { missingRootIsEmpty: true })
  await mkdir(mirror, { recursive: true })

  for (const [relativePath, sourceFile] of sourceFiles) {
    const target = path.join(mirror, ...relativePath.split('/'))
    const current = mirrorFiles.get(relativePath)
    if (current?.sha256 === sourceFile.sha256 && current.size === sourceFile.size) continue
    await mkdir(path.dirname(target), { recursive: true })
    await copyFile(path.join(source, ...relativePath.split('/')), target)
  }

  const extras = [...mirrorFiles.keys()]
    .filter((relativePath) => !sourceFiles.has(relativePath))
    .sort((left, right) => right.localeCompare(left))
  for (const relativePath of extras) {
    const target = path.resolve(mirror, ...relativePath.split('/'))
    if (!target.startsWith(`${mirror}${path.sep}`)) throw new Error(`Mirror path escaped its root: ${relativePath}`)
    await rm(target, { force: true })
  }
  await removeEmptyDirectories(mirror)
  return projectMirrorDiff(source, mirror)
}

async function fileInventory(root, { missingRootIsEmpty = false, ignoredPaths = new Set() } = {}) {
  const absoluteRoot = path.resolve(root)
  let rootStats
  try {
    rootStats = await stat(absoluteRoot)
  } catch (error) {
    if (missingRootIsEmpty && error?.code === 'ENOENT') return new Map()
    throw error
  }
  if (!rootStats.isDirectory()) throw new Error(`Project data root is not a directory: ${absoluteRoot}`)

  const files = new Map()
  const visit = async (directory) => {
    const entries = await readdir(directory, { withFileTypes: true })
    entries.sort((left, right) => left.name.localeCompare(right.name))
    for (const entry of entries) {
      const absolutePath = path.join(directory, entry.name)
      if (entry.isSymbolicLink()) throw new Error(`Project mirrors cannot contain symbolic links: ${absolutePath}`)
      if (entry.isDirectory()) {
        await visit(absolutePath)
        continue
      }
      if (!entry.isFile()) throw new Error(`Project mirrors require regular files: ${absolutePath}`)
      const relativePath = path.relative(absoluteRoot, absolutePath).split(path.sep).join('/')
      if (ignoredPaths.has(relativePath)) continue
      const bytes = await readFile(absolutePath)
      files.set(relativePath, {
        sha256: createHash('sha256').update(bytes).digest('hex'),
        size: bytes.length,
      })
    }
  }
  await visit(absoluteRoot)
  return files
}

async function removeEmptyDirectories(root, directory = root) {
  const entries = await readdir(directory, { withFileTypes: true })
  for (const entry of entries) {
    if (entry.isDirectory()) await removeEmptyDirectories(root, path.join(directory, entry.name))
  }
  if (directory === root) return
  if ((await readdir(directory)).length === 0) await rm(directory, { recursive: false })
}
