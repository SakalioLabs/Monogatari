import { createHash } from 'node:crypto'
import { mkdir, readFile, rename, stat, unlink, writeFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

import { compileSceneRoleplayBlueprint } from './lib/scene-roleplay-blueprint.mjs'

function usage() {
  console.error(
    'Usage: node scripts/compile-scene-roleplay-blueprint.mjs --project-root <root> --blueprint <file> [--write --expected-plan-fingerprint <sha256>]',
  )
}

function parseArguments(argv) {
  const values = new Map()
  const flags = new Set()
  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index]
    if (token === '--write') {
      flags.add(token)
      continue
    }
    if (!token.startsWith('--') || index + 1 >= argv.length) {
      throw new Error(`Invalid argument: ${token}`)
    }
    values.set(token, argv[index + 1])
    index += 1
  }
  return { values, flags }
}

function inside(root, candidate) {
  const relative = path.relative(root, candidate)
  return relative !== '' && !relative.startsWith('..') && !path.isAbsolute(relative)
}

async function sha256OrMissing(file) {
  try {
    const bytes = await readFile(file)
    return createHash('sha256').update(bytes).digest('hex')
  } catch (error) {
    if (error?.code === 'ENOENT') return 'missing'
    throw error
  }
}

async function atomicWrite(file, content) {
  await mkdir(path.dirname(file), { recursive: true })
  const temporary = `${file}.tmp-${process.pid}-${Date.now()}`
  try {
    await writeFile(temporary, content, { encoding: 'utf8', flag: 'wx' })
    await rename(temporary, file)
  } catch (error) {
    await unlink(temporary).catch(() => {})
    throw error
  }
}

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')

try {
  const { values, flags } = parseArguments(process.argv.slice(2))
  const projectRoot = path.resolve(repositoryRoot, values.get('--project-root') ?? '')
  const blueprintPath = path.resolve(repositoryRoot, values.get('--blueprint') ?? '')
  if (!values.has('--project-root') || !values.has('--blueprint')) {
    throw new Error('Both --project-root and --blueprint are required')
  }
  if (!(await stat(projectRoot)).isDirectory()) {
    throw new Error('Project root must be an existing directory')
  }

  const blueprint = JSON.parse(await readFile(blueprintPath, 'utf8'))
  const compiled = compileSceneRoleplayBlueprint(blueprint)
  const outputs = []
  for (const [portablePath, content] of Object.entries(compiled.serialized)) {
    if (!portablePath.startsWith('roleplays/') && !portablePath.startsWith('quality_suites/')) {
      throw new Error(`Output path is outside the supported catalogs: ${portablePath}`)
    }
    const destination = path.resolve(projectRoot, portablePath)
    if (!inside(projectRoot, destination)) {
      throw new Error(`Output path escapes the project root: ${portablePath}`)
    }
    outputs.push({
      path: portablePath.replaceAll('\\', '/'),
      destination,
      precondition: await sha256OrMissing(destination),
      resulting_sha256: createHash('sha256').update(content, 'utf8').digest('hex'),
      content,
    })
  }

  const planPayload = {
    schema: 'monogatari-scene-roleplay-blueprint-plan/v1',
    blueprint_path: path.relative(repositoryRoot, blueprintPath).replaceAll('\\', '/'),
    project_root: projectRoot,
    content_fingerprint: compiled.content_fingerprint,
    outputs: outputs.map(({ path: outputPath, precondition, resulting_sha256 }) => ({
      path: outputPath,
      precondition,
      resulting_sha256,
    })),
  }
  const planFingerprint = createHash('sha256')
    .update(JSON.stringify(planPayload), 'utf8')
    .digest('hex')
  const plan = { ...planPayload, plan_fingerprint: planFingerprint }

  if (!flags.has('--write')) {
    console.log(JSON.stringify(plan, null, 2))
    process.exit(0)
  }
  if (values.get('--expected-plan-fingerprint') !== planFingerprint) {
    throw new Error('Blueprint plan fingerprint is missing or stale')
  }

  for (const output of outputs) {
    await atomicWrite(output.destination, output.content)
  }
  console.log(
    JSON.stringify(
      {
        schema: 'monogatari-scene-roleplay-blueprint-result/v1',
        plan_fingerprint: planFingerprint,
        content_fingerprint: compiled.content_fingerprint,
        outputs: outputs.map(({ path: outputPath, resulting_sha256 }) => ({
          path: outputPath,
          sha256: resulting_sha256,
        })),
      },
      null,
      2,
    ),
  )
} catch (error) {
  console.error(JSON.stringify({ code: 'blueprint_compile_failed', message: error.message }, null, 2))
  usage()
  process.exitCode = 1
}
