import { invoke as tauriInvoke } from '@tauri-apps/api/core'

type CommandArgs = Record<string, unknown>

function hasTauriRuntime(): boolean {
  return typeof window !== 'undefined'
    && ('__TAURI_INTERNALS__' in window || '__TAURI__' in window)
}

export async function invokeCommand<T>(
  command: string,
  args?: CommandArgs,
  fallback?: T | (() => T)
): Promise<T> {
  if (hasTauriRuntime()) {
    return tauriInvoke<T>(command, args)
  }

  if (fallback !== undefined) {
    return typeof fallback === 'function' ? (fallback as () => T)() : fallback
  }

  throw new Error(`Tauri runtime unavailable for command: ${command}`)
}
