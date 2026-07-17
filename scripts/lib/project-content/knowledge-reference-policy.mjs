import { readdir as readDirectoryFromDisk, readFile as readFileFromDisk } from 'node:fs/promises'
import path from 'node:path'

export function createProjectKnowledgeReferencePolicy(options = {}) {
  const {
    repositoryRoot,
    dataRoots,
  } = resolveBoundaries(options)
  const readDirectory = options.readDirectory ?? readDirectoryFromDisk
  const readTextFile = options.readTextFile ?? readFileFromDisk
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

  async function collectKnowledgeReferenceEvidence() {
    const issues = []
    let characterCount = 0
    let knowledgeCount = 0
    let pinnedRefCount = 0
    let relatedRefCount = 0

    for (const dataRoot of dataRoots) {
      const knowledgeDir = path.join(dataRoot.dir, 'knowledge')
      const charactersDir = path.join(dataRoot.dir, 'characters')
      const knowledgeIds = new Set()
      const knowledgeRecords = []

      for (const file of await jsonFilesInDir(knowledgeDir, issues)) {
        const value = JSON.parse(await readTextFile(file, 'utf8'))
        const entries = Array.isArray(value) ? value : [value]
        for (const entry of entries) {
          const entryLabel = `${relative(file)}:${entry?.id || '<missing-knowledge-id>'}`
          if (!entry || typeof entry !== 'object' || Array.isArray(entry)) {
            issues.push(`${relative(file)}: knowledge records must be JSON objects`)
            continue
          }
          const id = stringField(entry, ['id'])
          if (!id) {
            issues.push(`${entryLabel}: knowledge id is required`)
            continue
          }
          if (knowledgeIds.has(id)) {
            issues.push(`${entryLabel}: duplicate knowledge id in ${dataRoot.label}`)
            continue
          }
          knowledgeIds.add(id)
          knowledgeRecords.push({ entry, entryLabel, id })
          knowledgeCount += 1
        }
      }

      for (const { entry, entryLabel, id } of knowledgeRecords) {
        for (const [fieldName, refs] of relatedKnowledgeRefFields(entry, entryLabel, issues)) {
          const seen = new Set()
          for (const ref of refs) {
            relatedRefCount += 1
            if (seen.has(ref)) {
              issues.push(`${entryLabel} ${fieldName}: duplicate related knowledge ref "${ref}"`)
              continue
            }
            seen.add(ref)
            if (ref === id) {
              issues.push(`${entryLabel} ${fieldName}: knowledge entries cannot reference themselves`)
            } else if (!knowledgeIds.has(ref)) {
              issues.push(`${entryLabel} ${fieldName}: missing related knowledge ref "${ref}" in ${dataRoot.label}/knowledge`)
            }
          }
        }
      }

      for (const file of await jsonFilesInDir(charactersDir, issues)) {
        const value = JSON.parse(await readTextFile(file, 'utf8'))
        const characters = Array.isArray(value) ? value : [value]
        for (const character of characters) {
          characterCount += 1
          const characterLabel = `${relative(file)}:${character?.id || character?.name || '<missing-character-id>'}`
          if (!character || typeof character !== 'object' || Array.isArray(character)) {
            issues.push(`${relative(file)}: character records must be JSON objects`)
            continue
          }

          for (const [fieldName, refs] of knowledgeRefFields(character, characterLabel, issues)) {
            for (const ref of refs) {
              pinnedRefCount += 1
              if (!knowledgeIds.has(ref)) {
                issues.push(`${characterLabel} ${fieldName}: missing pinned knowledge ref "${ref}" in ${dataRoot.label}/knowledge`)
              }
            }
          }
        }
      }
    }

    return {
      issues,
      pinnedRefCount,
      relatedRefCount,
      knowledgeCount,
      characterCount,
    }
  }

  async function verifyKnowledgeReferences() {
    const evidence = await collectKnowledgeReferenceEvidence()
    if (evidence.issues.length > 0) {
      throw new Error(`Knowledge ref verification failed:\n${evidence.issues.join('\n')}`)
    }
    log(
      `[release] Knowledge refs OK (${evidence.pinnedRefCount} pinned ref(s), ${evidence.relatedRefCount} related ref(s), ${evidence.knowledgeCount} knowledge record(s), ${evidence.characterCount} character record(s))`,
    )
    return evidence
  }

  return Object.freeze({
    collectKnowledgeReferenceEvidence,
    verifyKnowledgeReferences,
  })
}

export async function collectProjectKnowledgeReferenceEvidence(options = {}) {
  return createProjectKnowledgeReferencePolicy(options).collectKnowledgeReferenceEvidence()
}

function knowledgeRefFields(character, characterLabel, issues) {
  const fields = []
  for (const fieldName of ['knowledge_refs', 'knowledgeRefs', 'knowledge']) {
    const value = character?.[fieldName]
    if (value === undefined || value === null) continue
    if (!Array.isArray(value)) {
      issues.push(`${characterLabel} ${fieldName}: pinned knowledge refs must be an array`)
      continue
    }
    const refs = []
    for (const [index, item] of value.entries()) {
      if (typeof item !== 'string' || !item.trim()) {
        issues.push(`${characterLabel} ${fieldName}[${index}]: pinned knowledge ref must be a non-empty string`)
        continue
      }
      refs.push(item.trim())
    }
    fields.push([fieldName, refs])
  }
  return fields
}

function relatedKnowledgeRefFields(entry, entryLabel, issues) {
  const fields = []
  const present = ['related_entries', 'relatedEntries']
    .filter((fieldName) => entry?.[fieldName] !== undefined && entry?.[fieldName] !== null)
  if (present.length > 1) {
    issues.push(`${entryLabel}: related_entries and relatedEntries cannot both be present`)
  }
  for (const fieldName of present) {
    const value = entry[fieldName]
    if (!Array.isArray(value)) {
      issues.push(`${entryLabel} ${fieldName}: related knowledge refs must be an array`)
      continue
    }
    const refs = []
    for (const [index, item] of value.entries()) {
      if (typeof item !== 'string' || !item.trim()) {
        issues.push(`${entryLabel} ${fieldName}[${index}]: related knowledge ref must be a non-empty string`)
        continue
      }
      refs.push(item.trim())
    }
    fields.push([fieldName, refs])
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

function resolveBoundaries(options) {
  const repositoryRoot = options.repositoryRoot
  const rustDirectory = options.rustDirectory
  for (const [name, value] of Object.entries({ repositoryRoot, rustDirectory })) {
    if (typeof value !== 'string' || value.length === 0) {
      throw new Error(`Project Knowledge Reference policy requires ${name}.`)
    }
  }

  const dataRoots = options.dataRoots ?? [
    { label: 'data', dir: path.join(repositoryRoot, 'data') },
    { label: 'rust-engine/data', dir: path.join(rustDirectory, 'data') },
  ]
  if (!Array.isArray(dataRoots) || dataRoots.length === 0) {
    throw new Error('Project Knowledge Reference policy requires at least one data root.')
  }
  const labels = new Set()
  for (const [index, dataRoot] of dataRoots.entries()) {
    if (!dataRoot || typeof dataRoot.label !== 'string' || dataRoot.label.length === 0) {
      throw new Error(`Project Knowledge Reference policy dataRoots[${index}] requires label.`)
    }
    if (typeof dataRoot.dir !== 'string' || dataRoot.dir.length === 0) {
      throw new Error(`Project Knowledge Reference policy dataRoots[${index}] requires dir.`)
    }
    if (labels.has(dataRoot.label)) {
      throw new Error(`Project Knowledge Reference policy data root label is duplicated: ${dataRoot.label}`)
    }
    labels.add(dataRoot.label)
  }

  return {
    repositoryRoot,
    dataRoots: dataRoots.map((dataRoot) => ({ ...dataRoot })),
  }
}
