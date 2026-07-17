import { createHash } from 'node:crypto'
import { statSync as statFileFromDisk } from 'node:fs'
import { readdir as readDirectoryFromDisk, readFile as readFileFromDisk } from 'node:fs/promises'
import path from 'node:path'

const requiredRendererAssetCharacterIds = ['sakura', 'luna', 'kenji']
const requiredModel3dFixtureCharacterId = 'renderer_fox'
const requiredModel3dFixturePath = 'assets/models/fox.glb'
const requiredModel3dFixtureLicensePath = 'assets/models/fox.LICENSE.txt'
const requiredModel3dFixtureSha256 = 'd97044e701822bac5a62696459b27d7b375aada5de8574ed4362edbba94771f7'

const rendererAssetFields = [
  {
    names: ['live2d_model_path', 'live2dModelPath'],
    label: 'Live2D model',
    extensions: new Set(['.json', '.model3.json']),
  },
  {
    names: ['model_3d_path', 'model3dPath', 'model3DPath'],
    label: '3D model',
    extensions: new Set(['.glb', '.gltf']),
  },
  {
    names: ['portrait_path', 'portraitPath'],
    label: 'portrait',
    extensions: new Set(['.png', '.jpg', '.jpeg', '.webp', '.svg']),
  },
  {
    names: ['sprite_path', 'spritePath'],
    label: 'sprite',
    extensions: new Set(['.png', '.jpg', '.jpeg', '.webp', '.svg']),
  },
]

const sceneBackgroundExtensions = new Set(['.png', '.jpg', '.jpeg', '.webp', '.svg'])
const sceneModel3dExtensions = new Set(['.glb', '.gltf'])
export function createProjectRendererAssetPolicy(options = {}) {
  const {
    repositoryRoot,
    dataRoots: rendererDataRoots,
  } = resolveBoundaries(options)
  const readDirectory = options.readDirectory ?? readDirectoryFromDisk
  const readFile = options.readFile ?? readFileFromDisk
  const statFile = options.statFile ?? statFileFromDisk
  const log = options.log ?? console.log
  const relative = options.relativePath
    ?? ((file) => path.relative(repositoryRoot, file).replaceAll('\\', '/'))

  async function jsonFilesInDir(dir, issues) {
    try {
      return (await readDirectory(dir, { withFileTypes: true }))
        .filter((entry) => entry.isFile() && entry.name.endsWith('.json'))
        .map((entry) => path.join(dir, entry.name))
    } catch (error) {
      issues.push(`${relative(dir)}: ${error.message}`)
      return []
    }
  }

  async function collectRendererAssetEvidence() {
    const issues = []
    let characterCount = 0
    let sceneCount = 0
    let sceneBackgroundCount = 0
    let sceneModel3dCount = 0
    let declaredCharacterAssetCount = 0

    for (const dataRoot of rendererDataRoots) {
      const charactersDir = path.join(dataRoot.dir, 'characters')
      const scenesDir = path.join(dataRoot.dir, 'scenes')
      const coreCharactersWithRendererAssets = new Set()
      let model3dFixtureDeclared = false

      for (const file of await jsonFilesInDir(charactersDir, issues)) {
        const value = JSON.parse(await readFile(file, 'utf8'))
        const characters = Array.isArray(value) ? value : [value]
        for (const character of characters) {
          characterCount += 1
          const assetCount = verifyCharacterRendererAssets(
            character,
            dataRoot,
            relative(file),
            issues,
          )
          declaredCharacterAssetCount += assetCount
          if (assetCount > 0 && requiredRendererAssetCharacterIds.includes(character?.id)) {
            coreCharactersWithRendererAssets.add(character.id)
          }
          if (character?.id === requiredModel3dFixtureCharacterId) {
            const modelPath = stringField(character, ['model_3d_path', 'model3dPath', 'model3DPath'])
            model3dFixtureDeclared = modelPath === requiredModel3dFixturePath
            if (!model3dFixtureDeclared) {
              issues.push(`${relative(file)}:${requiredModel3dFixtureCharacterId} must reference ${requiredModel3dFixturePath}`)
            }
          }
        }
      }

      for (const characterId of requiredRendererAssetCharacterIds) {
        if (!coreCharactersWithRendererAssets.has(characterId)) {
          issues.push(`${dataRoot.label}: core sample character ${characterId} must declare a checked-in renderer asset`)
        }
      }
      if (!model3dFixtureDeclared) {
        issues.push(`${dataRoot.label}: ${requiredModel3dFixtureCharacterId} must declare the checked-in animated GLB fixture`)
      }
      await verifyModel3dFixture(dataRoot, issues)

      for (const file of await jsonFilesInDir(scenesDir, issues)) {
        const scene = JSON.parse(await readFile(file, 'utf8'))
        sceneCount += 1
        if (!nonEmptyString(scene.id)) issues.push(`${relative(file)}: scene id is required`)
        if (!nonEmptyString(scene.name)) issues.push(`${relative(file)}: scene name is required`)
        const backgroundPath = stringField(scene, ['background_path', 'backgroundPath'])
        const model3dPath = stringField(scene, ['model_3d_path', 'model3dPath', 'model3DPath'])
        if (!backgroundPath && !model3dPath) {
          issues.push(`${relative(file)}: scene must declare background_path or model_3d_path for renderer staging`)
        }
        if (backgroundPath) {
          sceneBackgroundCount += 1
          verifyLocalAssetPath({
            value: backgroundPath,
            dataRoot,
            label: `${relative(file)} background`,
            extensions: sceneBackgroundExtensions,
            issues,
          })
        }
        if (model3dPath) {
          sceneModel3dCount += 1
          verifyLocalAssetPath({
            value: model3dPath,
            dataRoot,
            label: `${relative(file)} 3D model`,
            extensions: sceneModel3dExtensions,
            issues,
          })
        }
      }
    }

    if (characterCount === 0) issues.push('Renderer asset verification found no character files')
    if (sceneCount === 0) issues.push('Renderer asset verification found no scene files')
    if (sceneBackgroundCount === 0) issues.push('Renderer asset verification found no scene backgrounds')

    return {
      issues,
      characterCount,
      sceneCount,
      sceneBackgroundCount,
      sceneModel3dCount,
      declaredCharacterAssetCount,
    }
  }

  async function verifyModel3dFixture(dataRoot, issues) {
    const modelPath = path.join(dataRoot.dir, requiredModel3dFixturePath)
    const licensePath = path.join(dataRoot.dir, requiredModel3dFixtureLicensePath)
    try {
      const model = await readFile(modelPath)
      if (model.length < 20 || model.subarray(0, 4).toString('ascii') !== 'glTF') {
        issues.push(`${dataRoot.label}: 3D fixture must be a binary glTF file`)
      } else {
        if (model.readUInt32LE(4) !== 2) {
          issues.push(`${dataRoot.label}: 3D fixture must use glTF version 2`)
        }
        if (model.readUInt32LE(8) !== model.length) {
          issues.push(`${dataRoot.label}: 3D fixture declared length must match file size`)
        }
        const sha256 = createHash('sha256').update(model).digest('hex')
        if (sha256 !== requiredModel3dFixtureSha256) {
          issues.push(`${dataRoot.label}: 3D fixture SHA-256 mismatch: ${sha256}`)
        }
      }
    } catch (error) {
      issues.push(`${dataRoot.label}: cannot read 3D fixture: ${error.message}`)
    }

    try {
      const license = await readFile(licensePath, 'utf8')
      for (const requiredText of ['PixelMannen', 'tomkranis', 'CC BY 4.0']) {
        if (!license.includes(requiredText)) {
          issues.push(`${dataRoot.label}: 3D fixture attribution must include ${requiredText}`)
        }
      }
    } catch (error) {
      issues.push(`${dataRoot.label}: cannot read 3D fixture attribution: ${error.message}`)
    }
  }

  function verifyCharacterRendererAssets(character, dataRoot, fileLabel, issues) {
    const characterLabel = `${fileLabel}:${character?.id || character?.name || '<missing-character-id>'}`
    let declaredAssetCount = 0

    if (!character || typeof character !== 'object' || Array.isArray(character)) {
      issues.push(`${fileLabel}: character records must be JSON objects`)
      return declaredAssetCount
    }
    if (!nonEmptyString(character.id)) issues.push(`${characterLabel}: character id is required`)
    if (!nonEmptyString(character.name)) issues.push(`${characterLabel}: character name is required`)

    for (const field of rendererAssetFields) {
      const value = stringField(character, field.names)
      if (!value) continue
      declaredAssetCount += 1
      verifyLocalAssetPath({
        value,
        dataRoot,
        label: `${characterLabel} ${field.label}`,
        extensions: field.extensions,
        issues,
      })
    }

    for (const [fieldName, value] of rendererMapFields(character)) {
      declaredAssetCount += 1
      verifyLocalAssetPath({
        value,
        dataRoot,
        label: `${characterLabel} ${fieldName}`,
        extensions: rendererAssetFields.find((field) => field.label === 'sprite').extensions,
        issues,
      })
    }

    return declaredAssetCount
  }

  function rendererMapFields(character) {
    const fields = []
    for (const fieldName of ['sprite_paths', 'spritePaths', 'sprites']) {
      const value = character?.[fieldName]
      if (!value || typeof value !== 'object' || Array.isArray(value)) continue
      for (const [key, pathValue] of Object.entries(value)) {
        if (nonEmptyString(pathValue)) fields.push([`${fieldName}.${key}`, pathValue])
      }
    }
    return fields
  }

  function stringField(object, names) {
    for (const name of names) {
      const value = object?.[name]
      if (typeof value === 'string' && value.trim()) return value.trim()
    }
    return null
  }

  function verifyLocalAssetPath({ value, dataRoot, label, extensions, issues }) {
    const normalized = value.replaceAll('\\', '/').trim()
    if (!normalized) return
    if (/^(https?:|data:|blob:|asset:)/i.test(normalized)) {
      issues.push(`${label}: checked-in renderer assets must use project-relative paths, got ${value}`)
      return
    }
    if (/^[a-zA-Z]:\//.test(normalized) || normalized.startsWith('//')) {
      issues.push(`${label}: renderer asset path must not be absolute: ${value}`)
      return
    }
    if (normalized.split('/').includes('..')) {
      issues.push(`${label}: renderer asset path must not contain parent traversal: ${value}`)
      return
    }

    const extension = assetExtension(normalized)
    if (!extensions.has(extension)) {
      issues.push(`${label}: unsupported renderer asset extension ${extension || '<none>'}`)
    }

    const candidate = path.resolve(dataRoot.dir, normalized)
    const rootPath = path.resolve(dataRoot.dir)
    if (!candidate.startsWith(rootPath + path.sep) && candidate !== rootPath) {
      issues.push(`${label}: renderer asset path escapes ${dataRoot.label}: ${value}`)
      return
    }
    if (!fileExistsSync(candidate)) {
      issues.push(`${label}: renderer asset does not exist: ${dataRoot.label}/${normalized}`)
    }
  }

  function assetExtension(value) {
    const lower = value.toLowerCase()
    if (lower.endsWith('.model3.json')) return '.model3.json'
    return path.extname(lower)
  }

  function fileExistsSync(filePath) {
    try {
      return statFile(filePath).isFile()
    } catch {
      return false
    }
  }

  async function verifyRendererAssets() {
    const evidence = await collectRendererAssetEvidence()
    if (evidence.issues.length > 0) {
      throw new Error(`Renderer asset verification failed:\n${evidence.issues.join('\n')}`)
    }
    log(
      `[release] Renderer assets OK (${evidence.characterCount} character record(s), ${evidence.sceneBackgroundCount}/${evidence.sceneCount} scene background(s), ${evidence.sceneModel3dCount} scene 3D model(s), ${evidence.declaredCharacterAssetCount} declared character asset(s))`,
    )
    return evidence
  }

  return Object.freeze({
    collectRendererAssetEvidence,
    verifyRendererAssets,
  })
}

export async function collectProjectRendererAssetEvidence(options = {}) {
  return createProjectRendererAssetPolicy(options).collectRendererAssetEvidence()
}

function nonEmptyString(value) {
  return typeof value === 'string' && value.trim().length > 0
}

function resolveBoundaries(options) {
  const repositoryRoot = options.repositoryRoot
  const rustDirectory = options.rustDirectory
  for (const [name, value] of Object.entries({ repositoryRoot, rustDirectory })) {
    if (typeof value !== 'string' || value.length === 0) {
      throw new Error(`Project Renderer Asset policy requires ${name}.`)
    }
  }
  const dataRoots = options.dataRoots ?? [
    { label: 'data', dir: path.join(repositoryRoot, 'data') },
    { label: 'rust-engine/data', dir: path.join(rustDirectory, 'data') },
  ]
  if (!Array.isArray(dataRoots) || dataRoots.length === 0) {
    throw new Error('Project Renderer Asset policy requires at least one data root.')
  }
  const labels = new Set()
  for (const [index, dataRoot] of dataRoots.entries()) {
    if (!dataRoot || typeof dataRoot.label !== 'string' || dataRoot.label.length === 0) {
      throw new Error(`Project Renderer Asset policy dataRoots[${index}] requires label.`)
    }
    if (typeof dataRoot.dir !== 'string' || dataRoot.dir.length === 0) {
      throw new Error(`Project Renderer Asset policy dataRoots[${index}] requires dir.`)
    }
    if (labels.has(dataRoot.label)) {
      throw new Error(`Project Renderer Asset policy data root label is duplicated: ${dataRoot.label}`)
    }
    labels.add(dataRoot.label)
  }
  return {
    repositoryRoot,
    dataRoots: dataRoots.map((dataRoot) => ({ ...dataRoot })),
  }
}
