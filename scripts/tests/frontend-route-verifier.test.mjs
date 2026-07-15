import assert from 'node:assert/strict'
import { readFile, readdir } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'

import {
  expectedFrontendRoutes,
  frontendRouteCoverageEvidence,
  parseFrontendRoutes,
  parseSidebarNavItems,
} from '../lib/frontend-route-verifier.mjs'

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..')

test('parses direct and lazy routes plus translated sidebar metadata', () => {
  const routerSource = `
    import HomeView from '../views/HomeView.vue'
    const router = createRouter({ routes: [
      { path: '/', name: 'home', component: HomeView },
      { path: "/game", name: "game", component: () => import("../views/GameView.vue") },
    ] })
  `
  assert.deepEqual(parseFrontendRoutes(routerSource), [
    { path: '/', name: 'home', component: 'HomeView.vue' },
    { path: '/game', name: 'game', component: 'GameView.vue' },
  ])

  const appSource = `
    const navItems = computed<NavItem[]>(() => [
      { path: '/', label: t('nav.dashboard'), badge: t('badge.ready') },
      { path: "/game", label: t("nav.story") },
    ])
  `
  assert.deepEqual(parseSidebarNavItems(appSource), [
    { path: '/', labelKey: 'nav.dashboard', badgeKey: 'badge.ready', badgeLiteral: undefined },
    { path: '/game', labelKey: 'nav.story', badgeKey: undefined, badgeLiteral: undefined },
  ])
})

test('checked-in router, shell, views, and English locale satisfy one shared policy', async () => {
  const frontendRoot = path.join(repositoryRoot, 'frontend')
  const [routerSource, appSource, locale, viewEntries] = await Promise.all([
    readFile(path.join(frontendRoot, 'src', 'router', 'index.ts'), 'utf8'),
    readFile(path.join(frontendRoot, 'src', 'App.vue'), 'utf8'),
    readFile(path.join(repositoryRoot, 'data', 'locales', 'en.json'), 'utf8').then(JSON.parse),
    readdir(path.join(frontendRoot, 'src', 'views'), { withFileTypes: true }),
  ])
  const evidence = frontendRouteCoverageEvidence({
    routerSource,
    appSource,
    localeMessages: locale.strings,
    availableComponents: viewEntries.filter((entry) => entry.isFile()).map((entry) => entry.name),
  })

  assert.deepEqual(evidence.issues, [])
  assert.equal(evidence.routes.length, expectedFrontendRoutes.length)
  assert.equal(evidence.navItems.length, expectedFrontendRoutes.filter((route) => route.sidebar !== false).length)
})

test('malformed route objects fail closed without borrowing fields from neighbors', () => {
  const malformedRouter = `
    import HomeView from '../views/HomeView.vue'
    createRouter({ routes: [
      { path: '/' },
      { name: 'home', component: HomeView },
    ] })
  `
  assert.deepEqual(parseFrontendRoutes(malformedRouter), [])

  const evidence = frontendRouteCoverageEvidence({
    routerSource: malformedRouter,
    appSource: 'const navItems = computed(() => [])',
    localeMessages: {},
    availableComponents: 'HomeView.vue',
  })
  assert(evidence.issues.includes('frontend view component catalog must be an iterable of filenames'))
  assert(evidence.issues.includes(`frontend router must expose ${expectedFrontendRoutes.length} routes, found 0`))
  assert(evidence.issues.includes('missing frontend route /'))
})

test('duplicate routes and missing component files remain independently actionable', async () => {
  const routerPath = path.join(repositoryRoot, 'frontend', 'src', 'router', 'index.ts')
  const appPath = path.join(repositoryRoot, 'frontend', 'src', 'App.vue')
  const localePath = path.join(repositoryRoot, 'data', 'locales', 'en.json')
  const [routerSource, appSource, locale] = await Promise.all([
    readFile(routerPath, 'utf8'),
    readFile(appPath, 'utf8'),
    readFile(localePath, 'utf8').then(JSON.parse),
  ])
  const duplicateRouter = routerSource.replace(
    'routes: [',
    "routes: [{ path: '/', name: 'home-copy', component: HomeView },",
  )
  const components = expectedFrontendRoutes
    .map((route) => route.component)
    .filter((component) => component !== 'HomeView.vue')
  const evidence = frontendRouteCoverageEvidence({
    routerSource: duplicateRouter,
    appSource,
    localeMessages: locale.strings,
    availableComponents: components,
  })

  assert(evidence.issues.includes('frontend router has duplicate path /'))
  assert(evidence.issues.includes(`frontend router must expose ${expectedFrontendRoutes.length} routes, found ${expectedFrontendRoutes.length + 1}`))
  assert(evidence.issues.includes('route / component file is missing: HomeView.vue'))
})

test('sidebar policy rejects full-screen exposure, literal badges, and missing locale keys', async () => {
  const frontendRoot = path.join(repositoryRoot, 'frontend')
  const [routerSource, appSource, locale] = await Promise.all([
    readFile(path.join(frontendRoot, 'src', 'router', 'index.ts'), 'utf8'),
    readFile(path.join(frontendRoot, 'src', 'App.vue'), 'utf8'),
    readFile(path.join(repositoryRoot, 'data', 'locales', 'en.json'), 'utf8').then(JSON.parse),
  ])
  const invalidApp = appSource
    .replace(
      'const navItems = computed<NavItem[]>(() => [',
      "const navItems = computed<NavItem[]>(() => [{ path: '/title', label: t('nav.unknown'), badge: 'Beta' },",
    )
    .replace("route.name !== 'game' && route.name !== 'title'", "route.name !== 'game'")
  const evidence = frontendRouteCoverageEvidence({
    routerSource,
    appSource: invalidApp,
    localeMessages: locale.strings,
    availableComponents: expectedFrontendRoutes.map((route) => route.component),
  })

  assert(evidence.issues.includes('route /title should not appear in the sidebar nav'))
  assert(evidence.issues.includes('sidebar nav item /title targets a full-screen route'))
  assert(evidence.issues.includes('sidebar nav /title label key is missing from data/locales/en.json: nav.unknown'))
  assert(evidence.issues.includes("sidebar nav /title badge must use t('badge.*') instead of a literal"))
  assert(evidence.issues.includes('App.vue must keep game and title as full-screen routes without the sidebar'))
})

test('release runner delegates route policy while retaining filesystem orchestration', async () => {
  const source = await readFile(path.join(repositoryRoot, 'scripts', 'verify-release.mjs'), 'utf8')
  assert(source.includes("from './lib/frontend-route-verifier.mjs'"))
  assert(source.includes('frontendRouteCoverageEvidence({'))
  assert(source.includes("readdir(path.join(frontendDir, 'src', 'views'), { withFileTypes: true })"))
  assert(!source.includes('function parseFrontendRoutes'))
  assert(!source.includes('const expectedFrontendRoutes = ['))
})
