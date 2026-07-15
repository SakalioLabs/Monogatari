import { createHash } from 'node:crypto'
import { readdir, readFile as readFileFromDisk } from 'node:fs/promises'
import path from 'node:path'

export const requiredStoryEventRuleIds = Object.freeze([
  'first_friend',
  'close_friend',
  'best_friend',
  'high_engagement',
  'creative_talk',
  'dedicated_player',
  'super_dedicated',
])

const requiredEventRules = requiredStoryEventRuleIds
const storyEventControlPattern = /[\u0000-\u001f\u007f]/

export function isPortableProjectContentId(value, maxLength) {
  return typeof value === 'string'
    && value.length > 0
    && value.length <= maxLength
    && value.trim() === value
    && /^[A-Za-z0-9_.-]+$/.test(value)
}

export function createProjectStoryEventPolicy(options = {}) {
  const {
    repositoryRoot: root,
    rustDirectory: rustDir,
    dataRoots: rendererDataRoots,
  } = resolveBoundaries(options)
  const readFile = options.readTextFile ?? readFileFromDisk
  const readDirectory = options.readDirectory ?? readdir
  const log = options.log ?? console.log
  const relative = options.relativePath ?? ((file) => path.relative(root, file).replaceAll('\\', '/'))
  const portableStoryEventId = isPortableProjectContentId

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

  function stableStringify(value) {
    if (Array.isArray(value)) return `[${value.map(stableStringify).join(',')}]`
    if (value && typeof value === 'object') {
      return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${stableStringify(value[key])}`).join(',')}}`
    }
    return JSON.stringify(value)
  }

  function nonEmptyString(value) {
    return typeof value === 'string' && value.trim().length > 0
  }

  async function collectStoryEventEvidence() {
    const issues = []
    const catalogs = []
    const contentInventories = []
    let fileCount = 0
    const rustStoryEventSource = await readFile(path.join(rustDir, 'crates', 'authoring', 'src', 'story_events.rs'), 'utf8')

    for (const dataRoot of rendererDataRoots) {
      const catalog = await loadStoryEventCatalog(dataRoot, issues)
      const contentInventory = await loadStoryContentInventory(dataRoot, issues)
      catalogs.push({ label: dataRoot.label, catalog })
      contentInventories.push({ label: dataRoot.label, inventory: contentInventory })
      fileCount += catalog.fileCount
      for (const eventId of requiredEventRules) {
        if (!catalog.events.has(eventId)) {
          issues.push(`${dataRoot.label}: missing required story event ${eventId}`)
        }
      }
      for (const event of catalog.events.values()) {
        for (const action of event.actions) {
          const target = action.type === 'unlock_scene'
            ? ['scene', action.scene_id, contentInventory.sceneIds]
            : action.type === 'unlock_dialogue'
              ? ['dialogue', action.dialogue_id, contentInventory.dialogueIds]
              : action.type === 'unlock_ending'
                ? ['ending', action.ending_id, contentInventory.endingIds]
                : null
          if (target && !target[2].has(target[1])) {
            issues.push(`${dataRoot.label}: story event ${event.event_id} unlocks missing ${target[0]} ${target[1]}`)
          }
        }
      }
    }

    if (catalogs.length === 2) {
      const left = normalizedStoryEventCatalog(catalogs[0].catalog)
      const right = normalizedStoryEventCatalog(catalogs[1].catalog)
      if (stableStringify(left) !== stableStringify(right)) {
        issues.push(`${catalogs[0].label} and ${catalogs[1].label} story event catalogs must match`)
      }
      if (catalogs[0].catalog.catalogFingerprint !== catalogs[1].catalog.catalogFingerprint) {
        issues.push(`${catalogs[0].label} and ${catalogs[1].label} normalized story event fingerprints must match`)
      }
      const leftEndings = [...contentInventories[0].inventory.endings.values()].sort((left, right) => left.id.localeCompare(right.id))
      const rightEndings = [...contentInventories[1].inventory.endings.values()].sort((left, right) => left.id.localeCompare(right.id))
      if (stableStringify(leftEndings) !== stableStringify(rightEndings)) {
        issues.push(`${contentInventories[0].label} and ${contentInventories[1].label} story ending catalogs must match`)
      }
    }
    for (const { label, catalog } of catalogs) {
      if (!/^[a-f0-9]{64}$/.test(catalog.catalogFingerprint)) {
        issues.push(`${label}: normalized story event fingerprint must be SHA-256`)
      }
    }
    const pinnedCatalogFingerprint = rustStoryEventSource.match(
      /fn checked_in_catalog_preserves_cross_runtime_catalog_fingerprint\(\)[\s\S]*?"([a-f0-9]{64})"/,
    )?.[1]
    if (!pinnedCatalogFingerprint) {
      issues.push('Rust story event catalog must pin the cross-runtime catalog fingerprint')
    } else if (catalogs[0]?.catalog.catalogFingerprint !== pinnedCatalogFingerprint) {
      issues.push(`Release and Rust story event catalog fingerprints differ: ${catalogs[0]?.catalog.catalogFingerprint} != ${pinnedCatalogFingerprint}`)
    }

    return {
      issues,
      fileCount,
      eventCount: catalogs[0]?.catalog.events.size ?? 0,
      endingCount: contentInventories[0]?.inventory.endings.size ?? 0,
      catalogFingerprint: catalogs[0]?.catalog.catalogFingerprint ?? null,
    }
  }

  async function loadStoryContentInventory(dataRoot, issues) {
    const sceneIds = await contentIdsInDirectory(dataRoot, 'scenes', issues)
    const dialogueIds = await contentIdsInDirectory(dataRoot, 'dialogue', issues)
    const endingFiles = await jsonFilesInDir(path.join(dataRoot.dir, 'endings'), issues)
    const endings = new Map()
    const allowedEndingFields = new Set(['schema', 'id', 'title', 'description', 'scene_id', 'dialogue_id'])
    for (const file of endingFiles) {
      const label = relative(file)
      const ending = JSON.parse(await readFile(file, 'utf8'))
      if (ending?.schema !== 'monogatari-story-ending/v1') issues.push(`${label}: schema must be monogatari-story-ending/v1`)
      for (const field of ['id', 'scene_id', 'dialogue_id']) {
        if (!portableStoryEventId(ending?.[field], 128)) issues.push(`${label}: ${field} must be a portable identifier`)
      }
      if (!nonEmptyString(ending?.title) || ending.title.length > 256) issues.push(`${label}: title must contain 1 to 256 characters`)
      if (!nonEmptyString(ending?.description) || ending.description.length > 2048) issues.push(`${label}: description must contain 1 to 2048 characters`)
      const unknownFields = Object.keys(ending ?? {}).filter((field) => !allowedEndingFields.has(field))
      if (unknownFields.length > 0) issues.push(`${label}: unknown fields ${unknownFields.join(', ')}`)
      if (sceneIds.size > 0 && !sceneIds.has(ending?.scene_id)) issues.push(`${label}: references missing scene ${String(ending?.scene_id)}`)
      if (dialogueIds.size > 0 && !dialogueIds.has(ending?.dialogue_id)) issues.push(`${label}: references missing dialogue ${String(ending?.dialogue_id)}`)
      if (endings.has(ending?.id)) issues.push(`${label}: duplicate ending id ${String(ending?.id)}`)
      else if (portableStoryEventId(ending?.id, 128)) endings.set(ending.id, ending)
    }
    return { sceneIds, dialogueIds, endingIds: new Set(endings.keys()), endings }
  }

  async function contentIdsInDirectory(dataRoot, directory, issues) {
    const files = await jsonFilesInDir(path.join(dataRoot.dir, directory), issues)
    const ids = new Set()
    for (const file of files) {
      const document = JSON.parse(await readFile(file, 'utf8'))
      if (!portableStoryEventId(document?.id, 128)) {
        issues.push(`${relative(file)}: id must be a portable identifier`)
      } else if (ids.has(document.id)) {
        issues.push(`${dataRoot.label}: duplicate ${directory} id ${document.id}`)
      } else {
        ids.add(document.id)
      }
    }
    return ids
  }

  async function loadStoryEventCatalog(dataRoot, issues) {
    const eventsDir = path.join(dataRoot.dir, 'events')
    const files = await jsonFilesInDir(eventsDir, issues)
    const events = new Map()

    if (files.length === 0) {
      issues.push(`${dataRoot.label}: events/ must contain at least one JSON catalog`)
    }

    for (const file of files) {
      const label = relative(file)
      const document = JSON.parse(await readFile(file, 'utf8'))
      if (document?.schema !== 'monogatari-story-event-catalog/v1') {
        issues.push(`${label}: schema must be monogatari-story-event-catalog/v1`)
      }
      if (!Array.isArray(document?.events)) {
        issues.push(`${label}: events must be an array`)
        continue
      }

      for (const event of document.events) {
        const eventId = typeof event?.event_id === 'string' ? event.event_id : ''
        const eventLabel = `${label}:${eventId || '<missing-id>'}`
        if (!portableStoryEventId(eventId, 128)) issues.push(`${eventLabel}: event_id must be a portable identifier`)
        if (!portableStoryEventId(event?.event_type, 128)) issues.push(`${eventLabel}: event_type must be a portable identifier`)
        if (!nonEmptyString(event?.description) || event.description.length > 2048 || storyEventControlPattern.test(event.description)) {
          issues.push(`${eventLabel}: description must be non-empty, bounded, and free of control characters`)
        }
        if (event?.data !== undefined && (!event.data || typeof event.data !== 'object' || Array.isArray(event.data))) {
          issues.push(`${eventLabel}: data must be a JSON object`)
        }
        if (event?.repeatable !== undefined && typeof event.repeatable !== 'boolean') {
          issues.push(`${eventLabel}: repeatable must be boolean`)
        }
        const actions = normalizeStoryEventActions(event, eventLabel, issues)

        const characterIds = event?.character_ids ?? []
        if (!Array.isArray(characterIds)) {
          issues.push(`${eventLabel}: character_ids must be an array`)
        } else {
          const uniqueCharacterIds = new Set()
          for (const characterId of characterIds) {
            if (!portableStoryEventId(characterId, 128)) issues.push(`${eventLabel}: invalid character_id ${String(characterId)}`)
            if (uniqueCharacterIds.has(characterId)) issues.push(`${eventLabel}: duplicate character_id ${String(characterId)}`)
            uniqueCharacterIds.add(characterId)
          }
        }

        const rule = event?.rule ?? {}
        if (!rule || typeof rule !== 'object' || Array.isArray(rule)) {
          issues.push(`${eventLabel}: rule must be an object`)
          continue
        }
        if (rule.min_relationship !== undefined && (!Number.isFinite(rule.min_relationship) || rule.min_relationship < -1 || rule.min_relationship > 1)) {
          issues.push(`${eventLabel}: min_relationship must be between -1 and 1`)
        }
        const hasMetric = rule.score_metric !== undefined && rule.score_metric !== null
        const hasMinimumScore = rule.min_score !== undefined && rule.min_score !== null
        if (hasMetric !== hasMinimumScore) {
          issues.push(`${eventLabel}: score_metric and min_score must be configured together`)
        }
        if (hasMetric && !['friendliness', 'engagement', 'creativity', 'overall'].includes(rule.score_metric)) {
          issues.push(`${eventLabel}: unsupported score_metric ${String(rule.score_metric)}`)
        }
        if (hasMinimumScore && (!Number.isFinite(rule.min_score) || rule.min_score < 0 || rule.min_score > 1)) {
          issues.push(`${eventLabel}: min_score must be between 0 and 1`)
        }
        if (rule.min_evaluation_count !== undefined && (!Number.isInteger(rule.min_evaluation_count) || rule.min_evaluation_count < 0 || rule.min_evaluation_count > 1_000_000)) {
          issues.push(`${eventLabel}: min_evaluation_count must be an integer between 0 and 1000000`)
        }

        if (events.has(eventId)) {
          issues.push(`${eventLabel}: duplicate event_id`)
          continue
        }
        events.set(eventId, {
          event_id: eventId,
          event_type: event?.event_type,
          description: event?.description,
          data: event?.data ?? {},
          actions,
          character_ids: Array.isArray(characterIds) ? [...characterIds].sort() : [],
          repeatable: event?.repeatable === true,
          rule: {
            min_relationship: rule.min_relationship ?? null,
            score_metric: rule.score_metric ?? null,
            min_score: rule.min_score ?? null,
            min_evaluation_count: rule.min_evaluation_count ?? null,
          },
          rule_fingerprint: storyEventRuleFingerprint(event),
        })
      }
    }

    return {
      events,
      fileCount: files.length,
      catalogFingerprint: storyEventCatalogFingerprint(events),
    }
  }

  function normalizeStoryEventActions(event, eventLabel, issues) {
    const sourceActions = event?.actions ?? []
    if (!Array.isArray(sourceActions)) {
      issues.push(`${eventLabel}: actions must be an array`)
      return []
    }
    if (sourceActions.length > 64) issues.push(`${eventLabel}: actions cannot exceed 64 entries`)
    const actions = []
    const seen = new Set()
    const append = (action, rejectDuplicate = true) => {
      const key = stableStringify(action)
      if (seen.has(key)) {
        if (rejectDuplicate) issues.push(`${eventLabel}: duplicate action ${key}`)
        return
      }
      seen.add(key)
      actions.push(action)
    }

    for (const action of sourceActions) {
      if (!action || typeof action !== 'object' || Array.isArray(action)) {
        issues.push(`${eventLabel}: each action must be an object`)
        continue
      }
      const allowedFields = {
        unlock_scene: ['type', 'scene_id'],
        unlock_dialogue: ['type', 'dialogue_id'],
        unlock_ending: ['type', 'ending_id'],
        set_flag: ['type', 'flag', 'value'],
      }[action.type]
      if (!allowedFields) {
        issues.push(`${eventLabel}: unsupported action type ${String(action.type)}`)
        continue
      }
      const extraFields = Object.keys(action).filter((field) => !allowedFields.includes(field))
      if (extraFields.length > 0) issues.push(`${eventLabel}: action ${action.type} has unknown fields ${extraFields.join(', ')}`)

      if (action.type === 'unlock_scene') {
        if (!portableStoryEventId(action.scene_id, 128)) issues.push(`${eventLabel}: invalid action scene_id`)
        else append({ type: action.type, scene_id: action.scene_id })
      } else if (action.type === 'unlock_dialogue') {
        if (!portableStoryEventId(action.dialogue_id, 128)) issues.push(`${eventLabel}: invalid action dialogue_id`)
        else append({ type: action.type, dialogue_id: action.dialogue_id })
      } else if (action.type === 'unlock_ending') {
        if (!portableStoryEventId(action.ending_id, 128)) issues.push(`${eventLabel}: invalid action ending_id`)
        else append({ type: action.type, ending_id: action.ending_id })
      } else {
        if (!portableStoryEventId(action.flag, 128)) issues.push(`${eventLabel}: invalid action flag`)
        if (typeof action.value !== 'boolean') issues.push(`${eventLabel}: set_flag value must be boolean`)
        if (portableStoryEventId(action.flag, 128) && typeof action.value === 'boolean') {
          append({ type: action.type, flag: action.flag, value: action.value })
        }
      }
    }

    const data = event?.data && typeof event.data === 'object' && !Array.isArray(event.data)
      ? event.data
      : {}
    for (const [field, type, targetField] of [
      ['unlock_scene', 'unlock_scene', 'scene_id'],
      ['dialogue_id', 'unlock_dialogue', 'dialogue_id'],
      ['unlock_ending', 'unlock_ending', 'ending_id'],
    ]) {
      if (!(field in data)) continue
      if (!portableStoryEventId(data[field], 128)) {
        issues.push(`${eventLabel}: legacy data.${field} must be a portable string id`)
        continue
      }
      append({ type, [targetField]: data[field] }, false)
    }
    if (actions.length > 64) issues.push(`${eventLabel}: normalized actions cannot exceed 64 entries`)
    return actions
  }

  function storyEventCatalogFingerprint(events) {
    const definitions = [...events.values()]
      .sort((left, right) => left.event_id.localeCompare(right.event_id))
      .map((event) => ({
        event_id: event.event_id,
        event_type: event.event_type,
        description: event.description,
        data: event.data,
        actions: event.actions,
        rule_fingerprint: event.rule_fingerprint,
      }))
    return createHash('sha256').update(stableStringify({
      schema: 'monogatari-story-event-catalog-fingerprint/v1',
      events: definitions,
    })).digest('hex')
  }

  function storyEventRuleFingerprint(event) {
    const rule = event?.rule ?? {}
    const characterIds = Array.isArray(event?.character_ids) ? [...event.character_ids].sort() : []
    const scoped = characterIds.length > 0 || event?.repeatable === true
    const payload = {
      schema: scoped ? 'monogatari-event-trigger-rule/v2' : 'monogatari-event-trigger-rule/v1',
      event_id: event?.event_id ?? '',
      event_type: event?.event_type ?? '',
      min_relationship: Number.isFinite(rule.min_relationship) ? rule.min_relationship.toFixed(6) : null,
      score_metric: rule.score_metric ?? null,
      min_score: Number.isFinite(rule.min_score) ? rule.min_score.toFixed(6) : null,
      min_evaluation_count: rule.min_evaluation_count ?? null,
      ...(scoped ? { character_ids: characterIds, repeatable: event?.repeatable === true } : {}),
    }
    return createHash('sha256').update(stableStringify(payload)).digest('hex')
  }

  function normalizedStoryEventCatalog(catalog) {
    return [...catalog.events.values()]
      .sort((left, right) => left.event_id.localeCompare(right.event_id))
  }

  async function verifyStoryEventCatalogs() {
    const evidence = await collectStoryEventEvidence()
    if (evidence.issues.length > 0) {
      throw new Error(`Story event catalog verification failed:\n${evidence.issues.join('\n')}`)
    }
    log(
      `[release] Story event catalogs OK (${evidence.fileCount} file(s), ${evidence.eventCount} events, catalog ${evidence.catalogFingerprint?.slice(0, 12) ?? 'missing'})`,
    )
    return evidence
  }

  return Object.freeze({
    collectStoryEventEvidence,
    loadStoryEventCatalog,
    verifyStoryEventCatalogs,
  })
}

export async function collectProjectStoryEventEvidence(options = {}) {
  return createProjectStoryEventPolicy(options).collectStoryEventEvidence()
}

function resolveBoundaries(options) {
  const repositoryRoot = options.repositoryRoot
  const rustDirectory = options.rustDirectory
  for (const [name, value] of Object.entries({ repositoryRoot, rustDirectory })) {
    if (typeof value !== 'string' || value.length === 0) {
      throw new Error(`Project Story Event policy requires ${name}.`)
    }
  }

  const dataRoots = options.dataRoots ?? [
    { label: 'data', dir: path.join(repositoryRoot, 'data') },
    { label: 'rust-engine/data', dir: path.join(rustDirectory, 'data') },
  ]
  if (!Array.isArray(dataRoots) || dataRoots.length === 0) {
    throw new Error('Project Story Event policy requires at least one data root.')
  }
  const labels = new Set()
  for (const [index, dataRoot] of dataRoots.entries()) {
    if (!dataRoot || typeof dataRoot.label !== 'string' || dataRoot.label.length === 0) {
      throw new Error(`Project Story Event policy dataRoots[${index}] requires label.`)
    }
    if (typeof dataRoot.dir !== 'string' || dataRoot.dir.length === 0) {
      throw new Error(`Project Story Event policy dataRoots[${index}] requires dir.`)
    }
    if (labels.has(dataRoot.label)) {
      throw new Error(`Project Story Event policy data root label is duplicated: ${dataRoot.label}`)
    }
    labels.add(dataRoot.label)
  }

  return {
    repositoryRoot,
    rustDirectory,
    dataRoots: dataRoots.map((dataRoot) => ({ ...dataRoot })),
  }
}
