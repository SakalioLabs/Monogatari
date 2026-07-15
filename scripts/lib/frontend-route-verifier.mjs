export const expectedFrontendRoutes = Object.freeze([
  { path: '/', name: 'home', component: 'HomeView.vue', navKey: 'nav.dashboard' },
  { path: '/title', name: 'title', component: 'TitleScreenView.vue', sidebar: false },
  { path: '/game', name: 'game', component: 'GameView.vue', navKey: 'nav.story' },
  { path: '/chat', name: 'chat', component: 'ChatView.vue', navKey: 'nav.chat' },
  { path: '/editor', name: 'editor', component: 'WorkflowEditor.vue', navKey: 'nav.workflow' },
  { path: '/character-editor', name: 'character-editor', component: 'CharacterEditorView.vue', navKey: 'nav.editor' },
  { path: '/assets', name: 'assets', component: 'SceneAssetsView.vue', navKey: 'nav.assets' },
  { path: '/settings', name: 'settings', component: 'SettingsView.vue', navKey: 'nav.settings' },
  { path: '/characters', name: 'characters', component: 'CharacterGalleryView.vue', navKey: 'nav.characters' },
  { path: '/group-chat', name: 'group-chat', component: 'GroupChatView.vue', navKey: 'nav.group' },
  { path: '/analytics', name: 'analytics', component: 'AnalyticsView.vue', navKey: 'nav.analytics' },
  { path: '/quality', name: 'quality', component: 'QualitySuiteView.vue', navKey: 'nav.quality' },
  { path: '/marketplace', name: 'marketplace', component: 'MarketplaceView.vue', navKey: 'nav.marketplace' },
  { path: '/plugins', name: 'plugins', component: 'PluginView.vue', navKey: 'nav.plugins' },
  { path: '/audio', name: 'audio', component: 'AudioView.vue', navKey: 'nav.audio' },
  { path: '/knowledge', name: 'knowledge', component: 'KnowledgeBaseView.vue', navKey: 'nav.knowledge' },
  { path: '/dialogue-editor', name: 'dialogue-editor', component: 'DialogueEditorView.vue', navKey: 'nav.dialogues' },
  { path: '/story-events', name: 'story-events', component: 'StoryEventEditorView.vue', navKey: 'nav.events' },
  { path: '/endings', name: 'endings', component: 'EndingEditorView.vue', navKey: 'nav.endings' },
  { path: '/scene-editor', name: 'scene-editor', component: 'SceneEditorView.vue', navKey: 'nav.scenes' },
  { path: '/cg-gallery', name: 'cg-gallery', component: 'CGGalleryView.vue', navKey: 'nav.cg-gallery' },
  { path: '/backlog', name: 'backlog', component: 'BacklogView.vue', navKey: 'nav.backlog' },
].map(Object.freeze))

export function parseFrontendRoutes(source) {
  if (typeof source !== 'string') return []

  const directImports = new Map()
  const importPattern = /import\s+([A-Za-z_$][\w$]*)\s+from\s+(['"])\.\.\/views\/([^'"]+\.vue)\2/g
  for (const match of source.matchAll(importPattern)) {
    directImports.set(match[1], match[3])
  }

  const routeBodies = extractArrayObjectBodies(source, /\broutes\s*:/g)
  return routeBodies.flatMap((body) => {
    const routePath = stringProperty(body, 'path')
    const name = stringProperty(body, 'name')
    const directComponent = /\bcomponent\s*:\s*([A-Za-z_$][\w$]*)/.exec(body)?.[1]
    const lazyComponent = /\bcomponent\s*:\s*\(\s*\)\s*=>\s*import\s*\(\s*(['"])\.\.\/views\/([^'"]+\.vue)\1\s*\)/.exec(body)?.[2]
    const component = lazyComponent ?? directImports.get(directComponent) ?? directComponent
    return routePath && name && component ? [{ path: routePath, name, component }] : []
  })
}

export function parseSidebarNavItems(source) {
  if (typeof source !== 'string') return []

  const itemBodies = extractArrayObjectBodies(
    source,
    /\bconst\s+navItems\s*=\s*computed(?:<[^>]+>)?\s*\(\s*\(\s*\)\s*=>/g,
  )
  return itemBodies.flatMap((body) => {
    const itemPath = stringProperty(body, 'path')
    const labelKey = translatedProperty(body, 'label')
    if (!itemPath || !labelKey) return []
    return [{
      path: itemPath,
      labelKey,
      badgeKey: translatedProperty(body, 'badge'),
      badgeLiteral: stringProperty(body, 'badge'),
    }]
  })
}

export function frontendRouteCoverageEvidence({
  routerSource,
  appSource,
  localeMessages,
  availableComponents,
} = {}) {
  const issues = []
  if (typeof routerSource !== 'string') issues.push('frontend router source must be a string')
  if (typeof appSource !== 'string') issues.push('App.vue source must be a string')
  if (!isRecord(localeMessages)) issues.push('English locale messages must be an object')

  const componentNames = toStringSet(availableComponents)
  if (!componentNames) issues.push('frontend view component catalog must be an iterable of filenames')

  const routes = parseFrontendRoutes(routerSource)
  const navItems = parseSidebarNavItems(appSource)
  const localeKeys = new Set(isRecord(localeMessages) ? Object.keys(localeMessages) : [])
  const expectedRoutesByPath = new Map(expectedFrontendRoutes.map((route) => [route.path, route]))
  const routesByPath = new Map(routes.map((route) => [route.path, route]))
  const navByPath = new Map(navItems.map((item) => [item.path, item]))
  const sidebarRoutes = expectedFrontendRoutes.filter((route) => route.sidebar !== false)

  for (const duplicate of duplicateValues(routes.map((route) => route.path))) {
    issues.push(`frontend router has duplicate path ${duplicate}`)
  }
  for (const duplicate of duplicateValues(routes.map((route) => route.name))) {
    issues.push(`frontend router has duplicate name ${duplicate}`)
  }
  for (const duplicate of duplicateValues(navItems.map((item) => item.path))) {
    issues.push(`sidebar nav has duplicate path ${duplicate}`)
  }

  if (routes.length !== expectedFrontendRoutes.length) {
    issues.push(`frontend router must expose ${expectedFrontendRoutes.length} routes, found ${routes.length}`)
  }
  if (navItems.length !== sidebarRoutes.length) {
    issues.push(`sidebar nav must expose ${sidebarRoutes.length} items, found ${navItems.length}`)
  }

  for (const expected of expectedFrontendRoutes) {
    const route = routesByPath.get(expected.path)
    const nav = navByPath.get(expected.path)
    if (!route) {
      issues.push(`missing frontend route ${expected.path}`)
      continue
    }
    if (route.name !== expected.name) {
      issues.push(`route ${expected.path} must be named ${expected.name}, found ${route.name}`)
    }
    if (route.component !== expected.component) {
      issues.push(`route ${expected.path} must load ${expected.component}, found ${route.component}`)
    }
    if (componentNames && !componentNames.has(expected.component)) {
      issues.push(`route ${expected.path} component file is missing: ${expected.component}`)
    }

    if (expected.sidebar === false) {
      if (nav) issues.push(`route ${expected.path} should not appear in the sidebar nav`)
      continue
    }
    if (!nav) {
      issues.push(`route ${expected.path} is missing from the sidebar nav`)
    } else if (nav.labelKey !== expected.navKey) {
      issues.push(`sidebar nav ${expected.path} must use ${expected.navKey}, found ${nav.labelKey}`)
    }
  }

  for (const route of routes) {
    if (!expectedRoutesByPath.has(route.path)) {
      issues.push(`unexpected frontend route ${route.path}`)
    }
    if (!route.component.endsWith('.vue')) {
      issues.push(`route ${route.path} must resolve to a Vue single-file component`)
    }
  }

  for (const item of navItems) {
    const expected = expectedRoutesByPath.get(item.path)
    if (!expected) {
      issues.push(`sidebar nav item ${item.path} does not match any expected route`)
    } else if (expected.sidebar === false) {
      issues.push(`sidebar nav item ${item.path} targets a full-screen route`)
    }
    if (!localeKeys.has(item.labelKey)) {
      issues.push(`sidebar nav ${item.path} label key is missing from data/locales/en.json: ${item.labelKey}`)
    }
    if (item.badgeLiteral) {
      issues.push(`sidebar nav ${item.path} badge must use t('badge.*') instead of a literal`)
    }
    if (item.badgeKey && !item.badgeKey.startsWith('badge.')) {
      issues.push(`sidebar nav ${item.path} badge key must use the badge.* namespace`)
    }
    if (item.badgeKey && !localeKeys.has(item.badgeKey)) {
      issues.push(`sidebar nav ${item.path} badge key is missing from data/locales/en.json: ${item.badgeKey}`)
    }
  }

  if (typeof appSource === 'string' && !/route\.name\s*!==\s*(['"])game\1\s*&&\s*route\.name\s*!==\s*(['"])title\2/.test(appSource)) {
    issues.push('App.vue must keep game and title as full-screen routes without the sidebar')
  }

  return { issues, routes, navItems }
}

function extractArrayObjectBodies(source, markerPattern) {
  markerPattern.lastIndex = 0
  const marker = markerPattern.exec(source)
  if (!marker) return []
  const arrayStart = findCodeCharacter(source, '[', marker.index + marker[0].length)
  if (arrayStart < 0) return []
  const arrayEnd = findBalancedEnd(source, arrayStart, '[', ']')
  if (arrayEnd < 0) return []

  const bodies = []
  let cursor = arrayStart + 1
  while (cursor < arrayEnd) {
    const objectStart = findCodeCharacter(source, '{', cursor, arrayEnd)
    if (objectStart < 0) break
    const objectEnd = findBalancedEnd(source, objectStart, '{', '}', arrayEnd)
    if (objectEnd < 0) return []
    bodies.push(source.slice(objectStart + 1, objectEnd))
    cursor = objectEnd + 1
  }
  return bodies
}

function findCodeCharacter(source, target, start, limit = source.length) {
  const state = lexicalState()
  for (let index = start; index < limit; index += 1) {
    if (advanceLexicalState(source, index, state)) {
      index = state.skipTo
      continue
    }
    if (!state.quote && source[index] === target) return index
  }
  return -1
}

function findBalancedEnd(source, start, open, close, limit = source.length) {
  const state = lexicalState()
  let depth = 0
  for (let index = start; index < limit; index += 1) {
    if (advanceLexicalState(source, index, state)) {
      index = state.skipTo
      continue
    }
    if (state.quote) continue
    if (source[index] === open) depth += 1
    if (source[index] === close) {
      depth -= 1
      if (depth === 0) return index
    }
  }
  return -1
}

function lexicalState() {
  return { quote: null, escaped: false, skipTo: -1 }
}

function advanceLexicalState(source, index, state) {
  const char = source[index]
  const next = source[index + 1]
  state.skipTo = index

  if (state.quote) {
    if (state.escaped) {
      state.escaped = false
    } else if (char === '\\') {
      state.escaped = true
    } else if (char === state.quote) {
      state.quote = null
    }
    return true
  }
  if (char === "'" || char === '"' || char === '`') {
    state.quote = char
    return true
  }
  if (char === '/' && next === '/') {
    const lineEnd = source.indexOf('\n', index + 2)
    state.skipTo = lineEnd < 0 ? source.length : lineEnd
    return true
  }
  if (char === '/' && next === '*') {
    const commentEnd = source.indexOf('*/', index + 2)
    state.skipTo = commentEnd < 0 ? source.length : commentEnd + 1
    return true
  }
  return false
}

function stringProperty(source, property) {
  const escaped = escapeRegExp(property)
  return new RegExp(`\\b${escaped}\\s*:\\s*(['"])([^'"]+)\\1`).exec(source)?.[2]
}

function translatedProperty(source, property) {
  const escaped = escapeRegExp(property)
  return new RegExp(`\\b${escaped}\\s*:\\s*t\\(\\s*(['"])([^'"]+)\\1`).exec(source)?.[2]
}

function duplicateValues(values) {
  const seen = new Set()
  const duplicates = new Set()
  for (const value of values) {
    if (seen.has(value)) duplicates.add(value)
    seen.add(value)
  }
  return [...duplicates]
}

function toStringSet(value) {
  if (!value || typeof value === 'string' || typeof value[Symbol.iterator] !== 'function') return null
  const strings = [...value]
  return strings.every((item) => typeof item === 'string') ? new Set(strings) : null
}

function isRecord(value) {
  return Boolean(value) && typeof value === 'object' && !Array.isArray(value)
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}
