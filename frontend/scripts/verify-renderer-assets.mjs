import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import vm from 'node:vm'
import ts from 'typescript'

const sourceUrl = new URL('../src/lib/rendererAssets.ts', import.meta.url)
const source = await readFile(sourceUrl, 'utf8')

const { outputText, diagnostics = [] } = ts.transpileModule(source, {
  fileName: 'rendererAssets.ts',
  reportDiagnostics: true,
  compilerOptions: {
    esModuleInterop: true,
    module: ts.ModuleKind.CommonJS,
    target: ts.ScriptTarget.ES2022,
  },
})

const blockingDiagnostics = diagnostics.filter((diagnostic) => diagnostic.category === ts.DiagnosticCategory.Error)
if (blockingDiagnostics.length > 0) {
  throw new Error(
    blockingDiagnostics
      .map((diagnostic) => ts.flattenDiagnosticMessageText(diagnostic.messageText, '\n'))
      .join('\n'),
  )
}

const module = { exports: {} }
const sandbox = {
  module,
  exports: module.exports,
  require(id) {
    if (id === './assets') {
      return {
        resolveAssetUrl(path) {
          return path ? `asset:${String(path).trim()}` : null
        },
      }
    }
    throw new Error(`Unexpected renderer asset test import: ${id}`)
  },
}

vm.runInNewContext(outputText, sandbox, { filename: 'rendererAssets.ts' })

const {
  cleanRendererPathMap,
  imageAssetExtensions,
  rendererAssetValidationMessage,
  selectCharacterRendererAsset,
} = module.exports

function plain(value) {
  return JSON.parse(JSON.stringify(value))
}

assert.deepEqual(
  plain(cleanRendererPathMap({
    ' happy ': ' assets/sprites/happy.png ',
    sad: '',
    ' ': 'assets/sprites/invalid.png',
  })),
  { happy: 'assets/sprites/happy.png' },
)

assert.equal(
  selectCharacterRendererAsset({
    live2d_model_path: ' live2d/hero.model3.json ',
    model_3d_path: 'models/hero.glb',
    sprite_path: 'assets/sprites/hero.png',
  }).mode,
  'live2d',
)

assert.deepEqual(
  plain(selectCharacterRendererAsset({
    model_3d_path: 'models/hero.glb',
    sprite_path: 'assets/sprites/hero.png',
  })),
  { mode: 'model3d', path: 'models/hero.glb', resolvedUrl: 'asset:models/hero.glb' },
)

assert.deepEqual(
  plain(selectCharacterRendererAsset(
    {
      emotion: 'sad',
      sprite_paths: {
        happy: 'assets/sprites/hero_happy.png',
        sad: 'assets/sprites/hero_sad.png',
        neutral: 'assets/sprites/hero_neutral.png',
      },
      sprite_path: 'assets/sprites/hero_base.png',
      portrait_path: 'assets/portraits/hero.png',
    },
    { expression: 'happy' },
  )),
  {
    mode: 'sprite',
    path: 'assets/sprites/hero_happy.png',
    resolvedUrl: 'asset:assets/sprites/hero_happy.png',
  },
)

assert.deepEqual(
  plain(selectCharacterRendererAsset({
    emotion: 'sad',
    sprite_paths: { neutral: 'assets/sprites/hero_neutral.png' },
    sprite_path: 'assets/sprites/hero_base.png',
    portrait_path: 'assets/portraits/hero.png',
  })),
  {
    mode: 'sprite',
    path: 'assets/sprites/hero_neutral.png',
    resolvedUrl: 'asset:assets/sprites/hero_neutral.png',
  },
)

assert.deepEqual(
  plain(selectCharacterRendererAsset({
    sprite_path: 'assets/sprites/hero_base.png',
    portrait_path: 'assets/portraits/hero.png',
  })),
  {
    mode: 'sprite',
    path: 'assets/sprites/hero_base.png',
    resolvedUrl: 'asset:assets/sprites/hero_base.png',
  },
)

assert.deepEqual(
  plain(selectCharacterRendererAsset({
    portrait_path: 'assets/portraits/hero.png',
  })),
  {
    mode: 'sprite',
    path: 'assets/portraits/hero.png',
    resolvedUrl: 'asset:assets/portraits/hero.png',
  },
)

assert.deepEqual(plain(selectCharacterRendererAsset(null)), {
  mode: 'placeholder',
  path: null,
  resolvedUrl: null,
})

assert.deepEqual(
  plain(selectCharacterRendererAsset(
    {
      live2d_model_path: 'https://cdn.example.com/hero.model3.json',
      model_3d_path: 'models/hero.glb',
    },
    { validatePaths: true },
  )),
  { mode: 'model3d', path: 'models/hero.glb', resolvedUrl: 'asset:models/hero.glb' },
)

assert.deepEqual(
  plain(selectCharacterRendererAsset(
    {
      live2d_model_path: 'C:/assets/hero.model3.json',
      model_3d_path: '../models/hero.glb',
      sprite_path: '/assets/sprites/hero.png',
    },
    { validatePaths: true },
  )),
  { mode: 'placeholder', path: null, resolvedUrl: null },
)

assert.equal(rendererAssetValidationMessage('https://cdn.example.com/hero.png', imageAssetExtensions), 'Use a project-relative path')
assert.equal(rendererAssetValidationMessage('/assets/hero.png', imageAssetExtensions), 'Absolute paths are not portable')
assert.equal(rendererAssetValidationMessage('../assets/hero.png', imageAssetExtensions), 'Parent traversal is not allowed')
assert.equal(rendererAssetValidationMessage('assets/hero.bmp', imageAssetExtensions), 'Expected .png, .jpg, .jpeg, .webp, .svg')
assert.equal(rendererAssetValidationMessage('live2d/hero.model3.json', ['.json', '.model3.json']), null)

console.log('[renderer-assets] Selector contract OK')
