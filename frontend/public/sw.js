const CACHE_NAME = "monogatari-web-v__APP_VERSION__-__BUILD_FINGERPRINT__";
const SCOPE_PATH = new URL(self.registration.scope).pathname;
const BASE_PATH = SCOPE_PATH.endsWith("/") ? SCOPE_PATH.slice(0, -1) : SCOPE_PATH;
const PROJECT_ASSET_MANIFEST_PATH = "/project-assets.json";
const INFERENCE_RUNTIME_PATH = "/inference-runtime.json";
const APP_SHELL_PATHS = [
  "/",
  "/index.html",
  "/offline.html",
  "/offline-i18n.js",
  "/favicon.svg",
  "/icons/app-icon.svg",
  "/icons/maskable-icon.svg",
  "/manifest.webmanifest",
  PROJECT_ASSET_MANIFEST_PATH,
  INFERENCE_RUNTIME_PATH,
  "/locales/en.json",
  "/locales/zh-CN.json",
  "/locales/zh.json",
  "/locales/ja-JP.json",
  "/locales/ja.json",
  "/locales/ko-KR.json",
  "/locales/ko.json"
];
const APP_SHELL = APP_SHELL_PATHS.map(withBase);

function withBase(path) {
  const normalized = path.startsWith("/") ? path : `/${path}`;
  if (!BASE_PATH || BASE_PATH === "/") return normalized;
  if (normalized === "/") return `${BASE_PATH}/`;
  return `${BASE_PATH}${normalized}`;
}

function routePath(pathname) {
  if (!BASE_PATH || BASE_PATH === "/") return pathname;
  if (pathname === BASE_PATH) return "/";
  if (pathname.startsWith(`${BASE_PATH}/`)) return pathname.slice(BASE_PATH.length) || "/";
  return pathname;
}

self.addEventListener("install", (event) => {
  event.waitUntil(
    caches
      .open(CACHE_NAME)
      .then((cache) => cache.addAll(APP_SHELL))
      .then(() => cacheProjectAssets())
      .then(() => self.skipWaiting())
  );
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((names) => Promise.all(names.filter((name) => name !== CACHE_NAME).map((name) => caches.delete(name))))
      .then(() => self.clients.claim())
  );
});

self.addEventListener("fetch", (event) => {
  const request = event.request;
  if (request.method !== "GET") return;

  const url = new URL(request.url);
  if (url.origin !== self.location.origin) return;
  const path = routePath(url.pathname);

  if (request.mode === "navigate") {
    event.respondWith(networkFirstNavigation(request));
    return;
  }

  if (path.startsWith("/assets/")) {
    event.respondWith(cacheFirst(request));
    return;
  }

  if (path.startsWith("/events/") || path.startsWith("/scenes/") || path.startsWith("/dialogue/") || path.startsWith("/roleplays/") || path.startsWith("/endings/") || path.startsWith("/characters/") || path.startsWith("/knowledge/")) {
    event.respondWith(staleWhileRevalidate(request));
    return;
  }

  if (path.startsWith("/locales/") || path.startsWith("/icons/") || path === "/manifest.webmanifest" || path === "/favicon.svg") {
    event.respondWith(staleWhileRevalidate(request));
  }
});

async function cacheProjectAssets() {
  try {
    const manifestResponse = await fetch(withBase(PROJECT_ASSET_MANIFEST_PATH), { cache: "no-cache" });
    if (!manifestResponse.ok) return;

    const manifest = await manifestResponse.json();
    if (manifest.schema !== "monogatari-web-project-assets/v1" || !Array.isArray(manifest.assets)) return;

    const projectAssets = [
      ...manifest.assets,
      ...(Array.isArray(manifest.event_catalogs) ? manifest.event_catalogs : []),
      ...(Array.isArray(manifest.scene_files) ? manifest.scene_files : []),
      ...(Array.isArray(manifest.dialogue_files) ? manifest.dialogue_files : []),
      ...(Array.isArray(manifest.roleplay_files) ? manifest.roleplay_files : []),
      ...(Array.isArray(manifest.ending_files) ? manifest.ending_files : []),
      ...(Array.isArray(manifest.character_files) ? manifest.character_files : []),
      ...(Array.isArray(manifest.knowledge_files) ? manifest.knowledge_files : [])
    ]
      .filter((assetPath) => typeof assetPath === "string" && ["/assets/", "/events/", "/scenes/", "/dialogue/", "/roleplays/", "/endings/", "/characters/", "/knowledge/"].some((prefix) => assetPath.startsWith(prefix)))
      .map(withBase);
    if (projectAssets.length === 0) return;

    const cache = await caches.open(CACHE_NAME);
    await cache.addAll(projectAssets);
  } catch {
    // Project assets still use cache-first runtime caching when the manifest is unavailable.
  }
}

async function networkFirstNavigation(request) {
  try {
    const response = await fetch(request);
    const cache = await caches.open(CACHE_NAME);
    cache.put(request, response.clone());
    return response;
  } catch {
    return (await caches.match(request)) || (await caches.match(withBase("/index.html"))) || caches.match(withBase("/offline.html"));
  }
}

async function cacheFirst(request) {
  const cached = await caches.match(request);
  if (cached) return cached;

  const response = await fetch(request);
  const cache = await caches.open(CACHE_NAME);
  cache.put(request, response.clone());
  return response;
}

async function staleWhileRevalidate(request) {
  const cached = await caches.match(request);
  const network = fetch(request)
    .then(async (response) => {
      const cache = await caches.open(CACHE_NAME);
      cache.put(request, response.clone());
      return response;
    })
    .catch(() => cached);

  return cached || network;
}
