function hasTauriRuntime(): boolean {
  return typeof window !== 'undefined'
    && ('__TAURI_INTERNALS__' in window || '__TAURI__' in window)
}

function serviceWorkerScope(): URL {
  const baseUrl = import.meta.env.BASE_URL || '/'
  if (baseUrl === './') return new URL('./', window.location.href)

  return new URL(baseUrl, window.location.origin)
}

export function registerPwa() {
  if (!import.meta.env.PROD) return
  if (hasTauriRuntime()) return
  if (!('serviceWorker' in navigator)) return

  window.addEventListener('load', () => {
    const scopeUrl = serviceWorkerScope()

    navigator.serviceWorker
      .register(new URL('sw.js', scopeUrl), { scope: scopeUrl.pathname })
      .catch((error) => console.warn('Monogatari service worker registration failed:', error))
  })
}
