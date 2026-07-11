import assert from 'node:assert/strict'
import { createHash } from 'node:crypto'
import { readFile } from 'node:fs/promises'
import vm from 'node:vm'
import ts from 'typescript'

const sourceUrl = new URL('../src/lib/rendererAssets.ts', import.meta.url)
const source = await readFile(sourceUrl, 'utf8')
const fixtureCharacterUrl = new URL('../../data/characters/renderer_fox.json', import.meta.url)
const fixtureModelUrl = new URL('../../data/assets/models/fox.glb', import.meta.url)
const fixtureLicenseUrl = new URL('../../data/assets/models/fox.LICENSE.txt', import.meta.url)
const fixtureSha256 = 'd97044e701822bac5a62696459b27d7b375aada5de8574ed4362edbba94771f7'

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
      live2d_model_path: 'live2d/hero.model3.json',
      model_3d_path: 'models/hero.glb',
      sprite_path: 'assets/sprites/hero_base.png',
    },
    { validatePaths: true, blockedPaths: ['asset:live2d/hero.model3.json'] },
  )),
  { mode: 'model3d', path: 'models/hero.glb', resolvedUrl: 'asset:models/hero.glb' },
)

assert.deepEqual(
  plain(selectCharacterRendererAsset(
    {
      live2d_model_path: 'live2d/hero.model3.json',
      model_3d_path: 'models/hero.glb',
      sprite_path: 'assets/sprites/hero_base.png',
    },
    { validatePaths: true, blockedPaths: ['live2d/hero.model3.json', 'asset:models/hero.glb'] },
  )),
  { mode: 'sprite', path: 'assets/sprites/hero_base.png', resolvedUrl: 'asset:assets/sprites/hero_base.png' },
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
assert.equal(rendererAssetValidationMessage('assets//hero.png', imageAssetExtensions), 'Path segments must be portable')
assert.equal(rendererAssetValidationMessage('assets/./hero.png', imageAssetExtensions), 'Path segments must be portable')
assert.equal(rendererAssetValidationMessage('assets/hero portrait.png', imageAssetExtensions), 'Path segments must be portable')
assert.equal(rendererAssetValidationMessage('assets/hero.bmp', imageAssetExtensions), 'Expected .png, .jpg, .jpeg, .webp, .svg')
assert.equal(rendererAssetValidationMessage('live2d/hero.model3.json', ['.json', '.model3.json']), null)

const fixtureCharacter = JSON.parse(await readFile(fixtureCharacterUrl, 'utf8'))
assert.equal(fixtureCharacter.id, 'renderer_fox')
assert.equal(fixtureCharacter.model_3d_path, 'assets/models/fox.glb')
assert.equal(selectCharacterRendererAsset(fixtureCharacter, { validatePaths: true }).mode, 'model3d')

const fixtureModel = await readFile(fixtureModelUrl)
assert.equal(fixtureModel.subarray(0, 4).toString('ascii'), 'glTF')
assert.equal(fixtureModel.readUInt32LE(4), 2)
assert.equal(fixtureModel.readUInt32LE(8), fixtureModel.length)
assert.equal(createHash('sha256').update(fixtureModel).digest('hex'), fixtureSha256)

const fixtureLicense = await readFile(fixtureLicenseUrl, 'utf8')
assert.match(fixtureLicense, /PixelMannen/)
assert.match(fixtureLicense, /tomkranis/)
assert.match(fixtureLicense, /CC BY 4\.0/)

console.log(`[renderer-assets] Selector and animated GLB fixture contract OK (${fixtureModel.length} bytes)`)
