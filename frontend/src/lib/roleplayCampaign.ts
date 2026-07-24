import { hasTauriRuntime, invokeCommand } from './tauri'
import {
  startBrowserSceneRoleplay,
  type SceneRoleplayDefinition,
  type SceneRoleplaySession,
  type SceneRoleplaySnapshot,
} from './sceneRoleplay'

export const ROLEPLAY_CAMPAIGN_SCHEMA = 'monogatari-roleplay-campaign/v1'
export const ROLEPLAY_CAMPAIGN_SESSION_SCHEMA = 'monogatari-roleplay-campaign-session/v1'

export type RoleplayCampaignTarget =
  | { kind: 'entry'; entry_id: string }
  | { kind: 'complete' }

export interface RoleplayCampaignRoute {
  ending_id: string
  target: RoleplayCampaignTarget
}

export interface RoleplayCampaignEntry {
  id: string
  roleplay_id: string
  routes: RoleplayCampaignRoute[]
}

export interface RoleplayCampaignDefinition {
  schema: string
  id: string
  title: string
  start_entry_id: string
  entries: RoleplayCampaignEntry[]
}

export interface RoleplayCampaignCompletion {
  entry_id: string
  roleplay_id: string
  ending_id: string
  total_turns: number
  scores: Record<string, number>
  observed_evidence: string[]
  relationships: Record<string, number>
}

export interface RoleplayCampaignSession {
  schema: string
  campaign_id: string
  current_entry_id: string | null
  status: 'active' | 'completed'
  relationships: Record<string, number>
  completed_entries: RoleplayCampaignCompletion[]
}

export interface RoleplayCampaignAdvance {
  completed: RoleplayCampaignCompletion
  target: RoleplayCampaignTarget
  status: 'active' | 'completed'
  current_entry_id: string | null
  relationships: Record<string, number>
}

export interface RoleplayCampaignRuntimeSnapshot {
  schema: string
  definition: RoleplayCampaignDefinition
  session: RoleplayCampaignSession
  active_roleplay: SceneRoleplaySnapshot | null
  last_advance: RoleplayCampaignAdvance | null
}

interface WebProjectManifest {
  schema: string
  campaign_files?: string[]
}

export async function loadRoleplayCampaigns(
  roleplays: SceneRoleplayDefinition[] = [],
): Promise<RoleplayCampaignDefinition[]> {
  if (hasTauriRuntime()) {
    return invokeCommand<RoleplayCampaignDefinition[]>('list_roleplay_campaigns')
  }
  const manifestResponse = await fetch(projectUrl('project-assets.json'), { cache: 'no-cache' })
  if (!manifestResponse.ok) throw new Error(`Project manifest returned HTTP ${manifestResponse.status}`)
  const manifest = await manifestResponse.json() as WebProjectManifest
  if (manifest.schema !== 'monogatari-web-project-assets/v1') {
    throw new Error(`Unsupported project manifest: ${String(manifest.schema)}`)
  }
  const campaigns = await Promise.all((manifest.campaign_files || []).map(async (path) => {
    const response = await fetch(projectUrl(path), { cache: 'no-cache' })
    if (!response.ok) throw new Error(`${path} returned HTTP ${response.status}`)
    return response.json() as Promise<RoleplayCampaignDefinition>
  }))
  return campaigns
    .map(campaign => validateRoleplayCampaign(campaign, roleplays))
    .sort((left, right) => left.id.localeCompare(right.id))
}

export function startBrowserRoleplayCampaign(
  definition: RoleplayCampaignDefinition,
  roleplays: SceneRoleplayDefinition[],
  relationships: Record<string, number> = {},
): RoleplayCampaignRuntimeSnapshot {
  validateRoleplayCampaign(definition, roleplays)
  const normalizedRelationships = normalizeRelationships(relationships)
  const session: RoleplayCampaignSession = {
    schema: ROLEPLAY_CAMPAIGN_SESSION_SCHEMA,
    campaign_id: definition.id,
    current_entry_id: definition.start_entry_id,
    status: 'active',
    relationships: normalizedRelationships,
    completed_entries: [],
  }
  return {
    schema: 'monogatari-roleplay-campaign-runtime-snapshot/v1',
    definition,
    session,
    active_roleplay: startEntryRoleplay(definition, session, roleplays),
    last_advance: null,
  }
}

export function advanceBrowserRoleplayCampaign(
  current: RoleplayCampaignRuntimeSnapshot,
  roleplays: SceneRoleplayDefinition[],
  completedRoleplay: SceneRoleplaySession,
): RoleplayCampaignRuntimeSnapshot {
  const definition = validateRoleplayCampaign(current.definition, roleplays)
  validateCampaignSession(definition, current.session)
  if (current.session.status !== 'active' || !current.session.current_entry_id) {
    throw new Error(`Roleplay campaign "${definition.id}" is already completed.`)
  }
  const entry = campaignEntry(definition, current.session.current_entry_id)
  if (completedRoleplay.roleplay_id !== entry.roleplay_id || completedRoleplay.status !== 'completed' || !completedRoleplay.ending_id) {
    throw new Error(`Completed roleplay does not match campaign entry "${entry.id}".`)
  }
  const route = entry.routes.find(candidate => candidate.ending_id === completedRoleplay.ending_id)
  if (!route) throw new Error(`Roleplay ending "${completedRoleplay.ending_id}" has no campaign route.`)

  const completion: RoleplayCampaignCompletion = {
    entry_id: entry.id,
    roleplay_id: entry.roleplay_id,
    ending_id: completedRoleplay.ending_id,
    total_turns: completedRoleplay.total_turns,
    scores: finiteValues(completedRoleplay.scores, 'score'),
    observed_evidence: uniqueIds(completedRoleplay.observed_evidence),
    relationships: normalizeRelationships(completedRoleplay.relationships || {}),
  }
  const relationships = {
    ...current.session.relationships,
    ...completion.relationships,
  }
  const targetEntryId = route.target.kind === 'entry' ? route.target.entry_id : null
  const status = route.target.kind === 'entry' ? 'active' : 'completed'
  const session: RoleplayCampaignSession = {
    ...current.session,
    current_entry_id: targetEntryId,
    status,
    relationships,
    completed_entries: [...current.session.completed_entries, completion],
  }
  validateCampaignSession(definition, session)
  const advance: RoleplayCampaignAdvance = {
    completed: completion,
    target: route.target,
    status,
    current_entry_id: targetEntryId,
    relationships,
  }
  return {
    schema: current.schema,
    definition,
    session,
    active_roleplay: status === 'active' ? startEntryRoleplay(definition, session, roleplays) : null,
    last_advance: advance,
  }
}

export function validateRoleplayCampaign(
  definition: RoleplayCampaignDefinition,
  roleplays: SceneRoleplayDefinition[] = [],
): RoleplayCampaignDefinition {
  if (definition.schema !== ROLEPLAY_CAMPAIGN_SCHEMA) throw new Error(`Unsupported roleplay campaign schema: ${definition.schema}`)
  portableId(definition.id, 'campaign id')
  if (!definition.title?.trim()) throw new Error('Roleplay campaign title is required.')
  if (!definition.entries?.length || definition.entries.length > 256) throw new Error(`Roleplay campaign "${definition.id}" has invalid entries.`)
  const entries = new Map<string, RoleplayCampaignEntry>()
  const usedRoleplays = new Set<string>()
  for (const entry of definition.entries) {
    portableId(entry.id, 'entry id')
    portableId(entry.roleplay_id, 'roleplay id')
    if (entries.has(entry.id)) throw new Error(`Duplicate campaign entry "${entry.id}".`)
    if (usedRoleplays.has(entry.roleplay_id)) throw new Error(`Roleplay "${entry.roleplay_id}" is assigned more than once.`)
    if (!entry.routes?.length || entry.routes.length > 64) throw new Error(`Campaign entry "${entry.id}" has invalid routes.`)
    const endings = new Set<string>()
    for (const route of entry.routes) {
      portableId(route.ending_id, 'ending id')
      if (endings.has(route.ending_id)) throw new Error(`Campaign entry "${entry.id}" repeats ending "${route.ending_id}".`)
      endings.add(route.ending_id)
      if (route.target.kind === 'entry') portableId(route.target.entry_id, 'target entry id')
      else if (route.target.kind !== 'complete') throw new Error(`Campaign entry "${entry.id}" has an invalid target.`)
    }
    entries.set(entry.id, entry)
    usedRoleplays.add(entry.roleplay_id)
  }
  if (!entries.has(definition.start_entry_id)) throw new Error(`Campaign start entry "${definition.start_entry_id}" is unavailable.`)
  for (const entry of definition.entries) {
    for (const route of entry.routes) {
      if (route.target.kind === 'entry' && !entries.has(route.target.entry_id)) {
        throw new Error(`Campaign entry "${entry.id}" targets missing entry "${route.target.entry_id}".`)
      }
    }
  }
  validateCampaignGraph(definition, entries)

  if (roleplays.length) {
    const byId = new Map(roleplays.map(roleplay => [roleplay.id, roleplay]))
    for (const entry of definition.entries) {
      const roleplay = byId.get(entry.roleplay_id)
      if (!roleplay) throw new Error(`Campaign entry "${entry.id}" references missing roleplay "${entry.roleplay_id}".`)
      const possibleEndings = roleplayEndingIds(roleplay)
      const routedEndings = new Set(entry.routes.map(route => route.ending_id))
      for (const endingId of possibleEndings) {
        if (!routedEndings.has(endingId)) throw new Error(`Campaign entry "${entry.id}" does not route ending "${endingId}".`)
      }
      for (const endingId of routedEndings) {
        if (!possibleEndings.has(endingId)) throw new Error(`Campaign entry "${entry.id}" routes unavailable ending "${endingId}".`)
      }
    }
  }
  return definition
}

function startEntryRoleplay(
  definition: RoleplayCampaignDefinition,
  session: RoleplayCampaignSession,
  roleplays: SceneRoleplayDefinition[],
): SceneRoleplaySnapshot {
  const entry = campaignEntry(definition, session.current_entry_id || '')
  const roleplay = roleplays.find(candidate => candidate.id === entry.roleplay_id)
  if (!roleplay) throw new Error(`Scene roleplay "${entry.roleplay_id}" is unavailable.`)
  return startBrowserSceneRoleplay(roleplay, session.relationships)
}

function validateCampaignSession(
  definition: RoleplayCampaignDefinition,
  session: RoleplayCampaignSession,
): void {
  if (session.schema !== ROLEPLAY_CAMPAIGN_SESSION_SCHEMA || session.campaign_id !== definition.id) {
    throw new Error('Roleplay campaign session does not match its definition.')
  }
  let expectedEntryId: string | null = definition.start_entry_id
  let expectedStatus: 'active' | 'completed' = 'active'
  const seen = new Set<string>()
  for (const completion of session.completed_entries) {
    if (!expectedEntryId || completion.entry_id !== expectedEntryId || seen.has(completion.entry_id)) {
      throw new Error('Campaign completion history does not match the authored route.')
    }
    seen.add(completion.entry_id)
    const entry = campaignEntry(definition, expectedEntryId)
    if (completion.roleplay_id !== entry.roleplay_id) throw new Error('Campaign completion uses the wrong roleplay.')
    const route = entry.routes.find(candidate => candidate.ending_id === completion.ending_id)
    if (!route) throw new Error(`Roleplay ending "${completion.ending_id}" has no campaign route.`)
    expectedEntryId = route.target.kind === 'entry' ? route.target.entry_id : null
    expectedStatus = route.target.kind === 'entry' ? 'active' : 'completed'
  }
  if (session.current_entry_id !== expectedEntryId || session.status !== expectedStatus) {
    throw new Error('Campaign cursor does not match its completion history.')
  }
  normalizeRelationships(session.relationships)
}

function validateCampaignGraph(
  definition: RoleplayCampaignDefinition,
  entries: Map<string, RoleplayCampaignEntry>,
): void {
  const visiting = new Set<string>()
  const visited = new Set<string>()
  let hasCompletion = false
  const visit = (entryId: string) => {
    if (visited.has(entryId)) return
    if (visiting.has(entryId)) throw new Error(`Roleplay campaign contains a cycle at "${entryId}".`)
    visiting.add(entryId)
    for (const route of campaignEntry(definition, entryId).routes) {
      if (route.target.kind === 'entry') visit(route.target.entry_id)
      else hasCompletion = true
    }
    visiting.delete(entryId)
    visited.add(entryId)
  }
  visit(definition.start_entry_id)
  if (visited.size !== entries.size) throw new Error('Roleplay campaign contains unreachable entries.')
  if (!hasCompletion) throw new Error('Roleplay campaign has no completion route.')
}

function roleplayEndingIds(roleplay: SceneRoleplayDefinition): Set<string> {
  const endings = new Set<string>([roleplay.exhaustion_ending_id])
  for (const node of roleplay.nodes) {
    if (node.timeout_target.kind === 'ending') endings.add(node.timeout_target.ending_id)
    for (const transition of node.transitions) {
      if (transition.target.kind === 'ending') endings.add(transition.target.ending_id)
    }
  }
  return endings
}

function campaignEntry(definition: RoleplayCampaignDefinition, entryId: string): RoleplayCampaignEntry {
  const entry = definition.entries.find(candidate => candidate.id === entryId)
  if (!entry) throw new Error(`Campaign entry "${entryId}" is unavailable.`)
  return entry
}

function normalizeRelationships(values: Record<string, number>): Record<string, number> {
  return Object.fromEntries(Object.entries(finiteValues(values, 'relationship')).map(([id, value]) => {
    portableId(id, 'relationship character id')
    if (value < -1 || value > 1) throw new Error(`Relationship "${id}" must be between -1 and 1.`)
    return [id, value]
  }))
}

function finiteValues(values: Record<string, number>, label: string): Record<string, number> {
  if (Object.keys(values).length > 256) throw new Error(`Campaign ${label} summary is too large.`)
  return Object.fromEntries(Object.entries(values).map(([id, value]) => {
    portableId(id, `${label} id`)
    if (!Number.isFinite(value)) throw new Error(`Campaign ${label} "${id}" is not finite.`)
    return [id, value]
  }))
}

function uniqueIds(values: string[]): string[] {
  const unique = new Set(values)
  if (unique.size !== values.length || unique.size > 256) throw new Error('Campaign evidence summary is invalid.')
  for (const value of values) portableId(value, 'evidence id')
  return [...values]
}

function portableId(value: string, label: string): void {
  if (!/^[A-Za-z0-9_-]{1,128}$/.test(value || '')) throw new Error(`Invalid ${label}: "${value}".`)
}

function projectUrl(relativePath: string): string {
  const base = import.meta.env.BASE_URL || '/'
  const baseUrl = base === './' ? new URL('./', window.location.href) : new URL(base, window.location.origin)
  return new URL(relativePath.replace(/^\/+/, ''), baseUrl).toString()
}
