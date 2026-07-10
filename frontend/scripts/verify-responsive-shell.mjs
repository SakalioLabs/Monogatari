import { readdir, readFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const frontendDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const distDir = path.join(frontendDir, 'dist')
const issues = []

const [appSource, globalCss, indexHtml, fallbackHtml, manifest, distCss] = await Promise.all([
  readText(path.join(frontendDir, 'src', 'App.vue')),
  readText(path.join(frontendDir, 'src', 'styles', 'main.css')),
  readText(path.join(distDir, 'index.html')),
  readText(path.join(distDir, '404.html')),
  readJson(path.join(distDir, 'manifest.webmanifest')),
  readDistCss(),
])

for (const [name, html] of [
  ['dist/index.html', indexHtml],
  ['dist/404.html', fallbackHtml],
]) {
  requireIncludes(html, 'viewport-fit=cover', `${name} must preserve safe-area viewport metadata`)
  requireIncludes(html, 'rel="manifest"', `${name} must preserve the PWA manifest link`)
}

if (manifest.display !== 'standalone') {
  issues.push('dist/manifest.webmanifest display must remain standalone')
}

const viewportProfiles = [
  {
    name: 'mobile',
    width: 375,
    sourceChecks: [
      [globalCss, '@media (max-width: 480px)', 'src/styles/main.css must keep the 375px mobile breakpoint'],
      [appSource, '@media (max-width: 860px)', 'App.vue must switch to the compact shell before tablet width'],
      [appSource, '.app-frame, .app-frame.sidebar-collapsed { display: block; }', 'compact shell must remove the desktop grid track'],
      [appSource, 'position: fixed; inset: 0 auto 0 0;', 'mobile sidebar must become an off-canvas drawer'],
      [appSource, 'transform: translateX(-102%);', 'mobile sidebar must remain off canvas until opened'],
      [appSource, '.app-main { padding-bottom: calc(60px + env(safe-area-inset-bottom, 0px)); }', 'mobile main area must clear bottom navigation'],
      [appSource, 'height: calc(56px + env(safe-area-inset-bottom, 0px));', 'mobile navigation must reserve safe-area height'],
      [appSource, 'grid-template-columns: repeat(5, minmax(0, 1fr));', 'mobile navigation must keep five stable action tracks'],
    ],
    distChecks: [
      '@media (width<=860px)',
      'height:calc(56px + env(safe-area-inset-bottom, 0px))',
      'padding-bottom:calc(60px + env(safe-area-inset-bottom, 0px))',
      'grid-template-columns:repeat(5,minmax(0,1fr))',
      'translate(-102%)',
    ],
  },
  {
    name: 'tablet',
    width: 768,
    sourceChecks: [
      [appSource, 'grid-template-columns: var(--sidebar-width) minmax(0, 1fr);', 'desktop shell must keep a stable sidebar and shrinkable workspace'],
      [appSource, 'min-height: 100svh;', 'App.vue root must use small viewport height for tablet/mobile WebViews'],
      [appSource, '.app-main { min-width: 0; min-height: 0; overflow: auto; }', 'App.vue main content must shrink without horizontal overflow'],
      [appSource, '.sidebar-nav { flex: 1; min-height: 0; overflow-y: auto;', 'sidebar navigation must scroll vertically without growing the shell'],
      [appSource, '.app-workspace { display: grid; min-width: 0; min-height: 100svh;', 'workspace must constrain its content track'],
    ],
    distChecks: [
      'grid-template-columns:var(--sidebar-width) minmax(0,1fr)',
      'min-height:100svh',
      'overflow-y:auto',
      'overflow:auto',
    ],
  },
]

for (const profile of viewportProfiles) {
  for (const [source, needle, message] of profile.sourceChecks) {
    requireNormalizedIncludes(source, needle, `${profile.name} ${profile.width}px: ${message}`)
  }
  for (const needle of profile.distChecks) {
    requireNormalizedIncludes(distCss, needle, `${profile.name} ${profile.width}px: built CSS must include ${needle}`)
  }
}

if (issues.length > 0) {
  throw new Error(`Responsive shell verification failed:\n${issues.join('\n')}`)
}

console.log('[responsive-shell] OK: built Web/PWA shell verified for 375px mobile and 768px tablet layout signals')

async function readText(file) {
  return readFile(file, 'utf8')
}

async function readJson(file) {
  return JSON.parse(await readText(file))
}

async function readDistCss() {
  let entries
  try {
    entries = await readdir(path.join(distDir, 'assets'), { withFileTypes: true })
  } catch (error) {
    throw new Error(`Unable to read dist CSS assets. Run npm run build:web before verify:responsive-shell. ${error.message}`)
  }

  const cssFiles = entries
    .filter((entry) => entry.isFile() && entry.name.endsWith('.css'))
    .map((entry) => path.join(distDir, 'assets', entry.name))
    .sort()

  if (cssFiles.length === 0) {
    throw new Error('dist/assets must contain built CSS files before responsive shell verification')
  }

  return (await Promise.all(cssFiles.map(readText))).join('\n')
}

function requireIncludes(source, needle, message) {
  if (!source.includes(needle)) {
    issues.push(message)
  }
}

function requireNormalizedIncludes(source, needle, message) {
  if (!normalize(source).includes(normalize(needle))) {
    issues.push(message)
  }
}

function normalize(value) {
  return value.replace(/\s+/g, '')
}
