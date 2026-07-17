import { createHash, randomUUID } from 'node:crypto'
import { copyFile, lstat, open, readdir, realpath, rename, rm, stat } from 'node:fs/promises'
import path from 'node:path'

export const PROJECT_ASSET_IMPORT_PLAN_SCHEMA_V1 = 'monogatari-project-asset-import-plan/v1'
export const PROJECT_ASSET_IMPORT_RESULT_SCHEMA_V1 = 'monogatari-project-asset-import-result/v1'

const MAX_ASSET_BYTES = 128 * 1024 * 1024
const SHA256_PATTERN = /^[a-f0-9]{64}$/
const PORTABLE_SEGMENT_PATTERN = /^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$/
const IMAGE_EXTENSIONS = new Set(['.png', '.jpg', '.jpeg', '.webp', '.bmp', '.gif', '.svg'])
const AUDIO_EXTENSIONS = new Set(['.mp3', '.ogg', '.wav', '.m4a', '.aac', '.flac'])

export class ProjectAssetImportError extends Error {
  constructor(code, message) {
    super(message)
    this.name = 'ProjectAssetImportError'
    this.code = code
  }
}

export async function planProjectAssetImport(input) {
  const projectRoot = await canonicalProjectRoot(input.projectRoot)
  const sourcePath = await canonicalSource(input.sourcePath)
  const destinationPath = portableDestination(input.destinationPath)
  const destination = path.join(projectRoot, ...destinationPath.split('/'))
  await ensureDestinationParent(projectRoot, destinationPath)

  const sourceInfo = await stat(sourcePath)
  if (sourceInfo.size <= 0 || sourceInfo.size > MAX_ASSET_BYTES) {
    throw new ProjectAssetImportError(
      'source_size_invalid',
      `Asset source must contain 1 to ${MAX_ASSET_BYTES} bytes; received ${sourceInfo.size}.`,
    )
  }

  const sourceExtension = path.extname(sourcePath).toLowerCase()
  const destinationExtension = path.extname(destinationPath).toLowerCase()
  if (sourceExtension !== destinationExtension) {
    throw new ProjectAssetImportError(
      'asset_extension_mismatch',
      `Source extension ${sourceExtension || '<none>'} does not match destination extension ${destinationExtension || '<none>'}.`,
    )
  }
  const mediaKind = mediaKindForDestination(destinationPath, destinationExtension)
  await validateAssetPayload(sourcePath, destinationExtension, sourceInfo.size)

  const sourceSha256 = await sha256File(sourcePath)
  const precondition = normalizePrecondition(input.precondition)
  const destinationState = await inspectDestination(destination, precondition)
  await rejectCaseCollision(path.dirname(destination), path.basename(destination))

  const fingerprintInput = {
    schema: PROJECT_ASSET_IMPORT_PLAN_SCHEMA_V1,
    operation: destinationState.exists ? 'replace' : 'create',
    project_root: projectRoot,
    source_path: sourcePath,
    destination_path: destinationPath,
    media_kind: mediaKind,
    byte_count: sourceInfo.size,
    source_sha256: sourceSha256,
    destination_precondition: precondition,
    current_destination_sha256: destinationState.sha256,
  }

  return {
    ...fingerprintInput,
    plan_fingerprint: sha256Text(JSON.stringify(fingerprintInput)),
  }
}

export async function applyProjectAssetImport(input) {
  const plan = await planProjectAssetImport(input)
  const expected = String(input.expectedPlanFingerprint || '').trim().toLowerCase()
  if (!SHA256_PATTERN.test(expected)) {
    throw new ProjectAssetImportError(
      'plan_fingerprint_required',
      'Applying an asset import requires the exact reviewed plan fingerprint.',
    )
  }
  if (expected !== plan.plan_fingerprint) {
    throw new ProjectAssetImportError(
      'plan_fingerprint_stale',
      'The asset import plan changed after review; plan the import again.',
    )
  }

  const destination = path.join(plan.project_root, ...plan.destination_path.split('/'))
  const temporary = path.join(
    path.dirname(destination),
    `.${path.basename(destination)}.${randomUUID()}.tmp`,
  )
  try {
    await copyFile(plan.source_path, temporary)
    const temporaryInfo = await lstat(temporary)
    if (!temporaryInfo.isFile() || temporaryInfo.isSymbolicLink()) {
      throw new ProjectAssetImportError('staged_asset_invalid', 'Staged asset is not a regular file.')
    }
    const stagedSha256 = await sha256File(temporary)
    if (temporaryInfo.size !== plan.byte_count || stagedSha256 !== plan.source_sha256) {
      throw new ProjectAssetImportError(
        'staged_asset_mismatch',
        'Staged asset bytes do not match the reviewed source.',
      )
    }
    await rename(temporary, destination)
  } catch (error) {
    await rm(temporary, { force: true }).catch(() => {})
    throw error
  }

  return {
    schema: PROJECT_ASSET_IMPORT_RESULT_SCHEMA_V1,
    operation: plan.operation,
    destination_path: plan.destination_path,
    media_kind: plan.media_kind,
    byte_count: plan.byte_count,
    sha256: plan.source_sha256,
    plan_fingerprint: plan.plan_fingerprint,
  }
}

async function canonicalProjectRoot(value) {
  const supplied = path.resolve(String(value || ''))
  const info = await regularPathInfo(supplied, 'project_root_invalid', 'Project root')
  if (!info.isDirectory()) {
    throw new ProjectAssetImportError('project_root_invalid', 'Project root must be a regular directory.')
  }
  const root = await realpath(supplied)
  const settings = path.join(root, 'settings.json')
  const settingsInfo = await regularPathInfo(settings, 'project_settings_missing', 'settings.json')
  if (!settingsInfo.isFile()) {
    throw new ProjectAssetImportError('project_settings_missing', 'Project root must contain a regular settings.json file.')
  }
  return root
}

async function canonicalSource(value) {
  const supplied = path.resolve(String(value || ''))
  const info = await regularPathInfo(supplied, 'source_invalid', 'Asset source')
  if (!info.isFile()) {
    throw new ProjectAssetImportError('source_invalid', 'Asset source must be a regular file.')
  }
  return realpath(supplied)
}

async function regularPathInfo(filePath, code, label) {
  let info
  try {
    info = await lstat(filePath)
  } catch {
    throw new ProjectAssetImportError(code, `${label} does not exist: ${filePath}`)
  }
  if (info.isSymbolicLink()) {
    throw new ProjectAssetImportError(code, `${label} cannot be a symbolic link: ${filePath}`)
  }
  return info
}

function portableDestination(value) {
  const destination = String(value || '')
  if (destination.length === 0 || destination.length > 512 || destination.trim() !== destination) {
    throw new ProjectAssetImportError('destination_path_invalid', 'Destination path is missing or not canonical.')
  }
  if (destination.includes('\\') || destination.includes(':') || destination.startsWith('/')) {
    throw new ProjectAssetImportError('destination_path_invalid', 'Destination path must be project-relative with forward slashes.')
  }
  const segments = destination.split('/')
  if (
    segments.length < 3
    || segments.length > 10
    || segments[0] !== 'assets'
    || segments.some(segment => !PORTABLE_SEGMENT_PATTERN.test(segment) || segment === '.' || segment === '..')
  ) {
    throw new ProjectAssetImportError(
      'destination_path_invalid',
      'Destination must use portable segments beneath an assets subdirectory.',
    )
  }
  return destination
}

async function ensureDestinationParent(projectRoot, destinationPath) {
  const segments = destinationPath.split('/').slice(0, -1)
  let current = projectRoot
  for (const segment of segments) {
    current = path.join(current, segment)
    const info = await regularPathInfo(current, 'destination_parent_invalid', 'Destination parent')
    if (!info.isDirectory()) {
      throw new ProjectAssetImportError('destination_parent_invalid', `Destination parent is not a directory: ${current}`)
    }
    const resolved = await realpath(current)
    if (!isInside(projectRoot, resolved)) {
      throw new ProjectAssetImportError('destination_parent_invalid', 'Destination parent escapes the project root.')
    }
  }
}

function mediaKindForDestination(destination, extension) {
  if (destination.startsWith('assets/models/') && extension === '.glb') return 'model3d'
  if (
    (destination.startsWith('assets/backgrounds/')
      || destination.startsWith('assets/scenes/')
      || destination.startsWith('assets/characters/'))
    && IMAGE_EXTENSIONS.has(extension)
  ) return 'image'
  if (destination.startsWith('assets/audio/') && AUDIO_EXTENSIONS.has(extension)) return 'audio'
  throw new ProjectAssetImportError(
    'asset_type_unsupported',
    `Asset destination and extension are not supported: ${destination}`,
  )
}

async function validateAssetPayload(sourcePath, extension, byteCount) {
  if (extension !== '.glb') return
  const handle = await open(sourcePath, 'r')
  try {
    const header = Buffer.alloc(20)
    const { bytesRead } = await handle.read(header, 0, header.length, 0)
    if (bytesRead !== header.length || header.toString('ascii', 0, 4) !== 'glTF') {
      throw new ProjectAssetImportError('glb_header_invalid', 'GLB source does not contain a valid glTF header.')
    }
    if (header.readUInt32LE(4) !== 2 || header.readUInt32LE(8) !== byteCount) {
      throw new ProjectAssetImportError('glb_header_invalid', 'GLB source must use glTF 2.0 and declare its exact byte length.')
    }
    const jsonLength = header.readUInt32LE(12)
    if (header.toString('ascii', 16, 20) !== 'JSON' || jsonLength <= 0 || 20 + jsonLength > byteCount) {
      throw new ProjectAssetImportError('glb_json_invalid', 'GLB source is missing its bounded JSON chunk.')
    }
    const json = Buffer.alloc(jsonLength)
    const payload = await handle.read(json, 0, jsonLength, 20)
    if (payload.bytesRead !== jsonLength) {
      throw new ProjectAssetImportError('glb_json_invalid', 'GLB JSON chunk is truncated.')
    }
    let document
    try {
      document = JSON.parse(json.toString('utf8').replace(/[\0\s]+$/u, ''))
    } catch {
      throw new ProjectAssetImportError('glb_json_invalid', 'GLB JSON chunk is not valid UTF-8 JSON.')
    }
    if (document?.asset?.version !== '2.0') {
      throw new ProjectAssetImportError('glb_json_invalid', 'GLB JSON asset.version must be 2.0.')
    }
  } finally {
    await handle.close()
  }
}

function normalizePrecondition(value) {
  if (value === 'missing' || value?.kind === 'missing') return { kind: 'missing' }
  const sha256 = typeof value === 'string' ? value : value?.value
  const normalized = String(sha256 || '').trim().toLowerCase()
  if (!SHA256_PATTERN.test(normalized)) {
    throw new ProjectAssetImportError(
      'destination_precondition_invalid',
      'Destination precondition must be missing or an exact lowercase SHA-256.',
    )
  }
  return { kind: 'sha256', value: normalized }
}

async function inspectDestination(destination, precondition) {
  let info
  try {
    info = await lstat(destination)
  } catch {
    if (precondition.kind !== 'missing') {
      throw new ProjectAssetImportError('destination_precondition_failed', 'Destination file is missing.')
    }
    return { exists: false, sha256: null }
  }
  if (info.isSymbolicLink() || !info.isFile()) {
    throw new ProjectAssetImportError('destination_invalid', 'Destination must be missing or a regular file.')
  }
  if (precondition.kind === 'missing') {
    throw new ProjectAssetImportError('destination_precondition_failed', 'Destination file already exists.')
  }
  const sha256 = await sha256File(destination)
  if (sha256 !== precondition.value) {
    throw new ProjectAssetImportError('destination_precondition_failed', 'Destination SHA-256 does not match the precondition.')
  }
  return { exists: true, sha256 }
}

async function rejectCaseCollision(parent, fileName) {
  const folded = fileName.toLowerCase()
  const collision = (await readdir(parent)).find(entry => entry.toLowerCase() === folded && entry !== fileName)
  if (collision) {
    throw new ProjectAssetImportError(
      'destination_case_collision',
      `Destination collides case-insensitively with existing path ${collision}.`,
    )
  }
}

async function sha256File(filePath) {
  const handle = await open(filePath, 'r')
  const hash = createHash('sha256')
  const buffer = Buffer.alloc(64 * 1024)
  try {
    let offset = 0
    while (true) {
      const { bytesRead } = await handle.read(buffer, 0, buffer.length, offset)
      if (bytesRead === 0) break
      hash.update(buffer.subarray(0, bytesRead))
      offset += bytesRead
    }
  } finally {
    await handle.close()
  }
  return hash.digest('hex')
}

function sha256Text(value) {
  return createHash('sha256').update(value).digest('hex')
}

function isInside(root, candidate) {
  const relative = path.relative(root, candidate)
  return relative === '' || (!relative.startsWith(`..${path.sep}`) && relative !== '..' && !path.isAbsolute(relative))
}
