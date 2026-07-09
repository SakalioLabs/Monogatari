import { resolveAssetUrl } from './assets'

export type RendererAssetMode = 'live2d' | 'model3d' | 'sprite' | 'placeholder'

export interface CharacterRendererSource {
  live2d_model_path?: string | null
  model_3d_path?: string | null
  portrait_path?: string | null
  sprite_path?: string | null
  sprite_paths?: Record<string, unknown> | null
  emotion?: string | null
}

export interface RendererAssetSpec {
  key: 'live2d_model_path' | 'model_3d_path' | 'portrait_path' | 'sprite_path'
  label: string
  extensions: string[]
}

export interface RendererAssetChoice {
  mode: RendererAssetMode
  path: string | null
  resolvedUrl: string | null
}

export const imageAssetExtensions = ['.png', '.jpg', '.jpeg', '.webp', '.svg']

export const rendererAssetSpecs: RendererAssetSpec[] = [
  { key: 'live2d_model_path', label: 'Live2D', extensions: ['.json', '.model3.json'] },
  { key: 'model_3d_path', label: '3D', extensions: ['.glb', '.gltf'] },
  { key: 'portrait_path', label: 'Portrait', extensions: imageAssetExtensions },
  { key: 'sprite_path', label: 'Fallback sprite', extensions: imageAssetExtensions },
]

export function cleanRendererPathMap(value: unknown): Record<string, string> {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return {}

  return Object.fromEntries(
    Object.entries(value as Record<string, unknown>)
      .map(([slot, path]) => [slot.trim(), String(path ?? '').trim()])
      .filter(([slot, path]) => slot.length > 0 && path.length > 0)
  )
}

export function rendererAssetValidationMessage(path: string, extensions: string[]): string | null {
  const normalized = path.trim().replace(/\\/g, '/')
  if (!normalized) return null
  if (/^[a-zA-Z]:\//.test(normalized) || normalized.startsWith('/')) return 'Absolute paths are not portable'
  if (/^[a-zA-Z][a-zA-Z0-9+.-]*:/.test(normalized)) return 'Use a project-relative path'
  const segments = normalized.split('/')
  if (segments.includes('..')) return 'Parent traversal is not allowed'
  if (segments.some(segment => !segment || segment === '.')) return 'Path segments must be portable'
  if (segments.some(segment => !/^[A-Za-z0-9._-]+$/.test(segment))) return 'Path segments must be portable'

  const extension = rendererAssetExtension(normalized)
  if (!extensions.includes(extension)) return `Expected ${extensions.join(', ')}`
  return null
}

export function selectCharacterRendererAsset(
  character: CharacterRendererSource | null | undefined,
  options: { expression?: string | null; validatePaths?: boolean } = {},
): RendererAssetChoice {
  if (!character) return placeholderChoice()

  const expression = options.expression?.trim() || character.emotion?.trim() || 'neutral'
  const live2d = assetChoice('live2d', character.live2d_model_path, rendererAssetSpecs[0].extensions, options.validatePaths)
  if (live2d) return live2d

  const model3d = assetChoice('model3d', character.model_3d_path, rendererAssetSpecs[1].extensions, options.validatePaths)
  if (model3d) return model3d

  const sprites = cleanRendererPathMap(character.sprite_paths)
  const spritePath = sprites[expression]
    || sprites[character.emotion?.trim() || '']
    || sprites.neutral
    || character.sprite_path
    || character.portrait_path
  const sprite = assetChoice('sprite', spritePath, imageAssetExtensions, options.validatePaths)
  if (sprite) return sprite

  return placeholderChoice()
}

function assetChoice(
  mode: RendererAssetMode,
  path: string | null | undefined,
  extensions: string[],
  validatePaths = false,
): RendererAssetChoice | null {
  const trimmed = path?.trim()
  if (!trimmed) return null
  if (validatePaths && rendererAssetValidationMessage(trimmed, extensions)) return null

  return {
    mode,
    path: trimmed,
    resolvedUrl: resolveAssetUrl(trimmed),
  }
}

function placeholderChoice(): RendererAssetChoice {
  return {
    mode: 'placeholder',
    path: null,
    resolvedUrl: null,
  }
}

function rendererAssetExtension(path: string): string {
  const lower = path.toLowerCase()
  if (lower.endsWith('.model3.json')) return '.model3.json'
  const match = lower.match(/\.[^./?#]+(?=([?#].*)?$)/)
  return match?.[0] || ''
}
