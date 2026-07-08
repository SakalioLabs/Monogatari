const CACHE_NAME = "monogatari-web-v0.9.5";
const SCOPE_PATH = new URL(self.registration.scope).pathname;
const BASE_PATH = SCOPE_PATH.endsWith("/") ? SCOPE_PATH.slice(0, -1) : SCOPE_PATH;
const APP_SHELL_PATHS = [
  "/",
  "/index.html",
  "/offline.html",
  "/favicon.svg",
  "/icons/app-icon.svg",
  "/icons/maskable-icon.svg",
  "/manifest.webmanifest",
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

  if (path.startsWith("/locales/") || path.startsWith("/icons/") || path === "/manifest.webmanifest" || path === "/favicon.svg") {
    event.respondWith(staleWhileRevalidate(request));
  }
});

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
