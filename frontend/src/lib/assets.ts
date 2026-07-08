import { convertFileSrc } from '@tauri-apps/api/core'

function hasTauriRuntime(): boolean {
  return typeof window !== 'undefined'
    && ('__TAURI_INTERNALS__' in window || '__TAURI__' in window)
}

function baseUrl(): URL {
  const base = import.meta.env.BASE_URL || '/'
  if (base === './') return new URL('./', window.location.href)

  return new URL(base, window.location.origin)
}

function isExternalAsset(path: string): boolean {
  return /^(https?:|data:|blob:|asset:)/i.test(path)
}

function isAbsoluteFilePath(path: string): boolean {
  return /^[a-zA-Z]:[\\/]/.test(path) || path.startsWith('\\\\') || path.startsWith('//')
}

export function resolveAssetUrl(path: string | null | undefined): string | null {
  if (!path) return null

  const trimmed = path.trim()
  if (!trimmed) return null
  if (isExternalAsset(trimmed)) return trimmed

  if (isAbsoluteFilePath(trimmed)) {
    return hasTauriRuntime() ? convertFileSrc(trimmed) : trimmed.replace(/\\/g, '/')
  }

  const normalized = trimmed.replace(/\\/g, '/').replace(/^\/+/, '')
  return new URL(normalized, baseUrl()).toString()
}
