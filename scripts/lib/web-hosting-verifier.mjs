const sharedCspFragments = [
  "default-src 'self'",
  "script-src 'self'",
  "style-src 'self' 'unsafe-inline'",
  "img-src 'self' asset: http://asset.localhost data: blob:",
  "media-src 'self' asset: http://asset.localhost data: blob:",
  "font-src 'self' data:",
  "connect-src 'self' asset: http://asset.localhost blob: https: http://localhost:* http://127.0.0.1:* ws://localhost:* ws://127.0.0.1:*",
  "worker-src 'self' blob:",
  "object-src 'none'",
  "base-uri 'self'",
  "form-action 'none'",
]

export const requiredTauriCspFragments = Object.freeze([
  ...sharedCspFragments,
  "frame-ancestors 'none'",
])
export const requiredWebCspFragments = Object.freeze([
  ...sharedCspFragments,
  "script-src 'self' 'wasm-unsafe-eval'",
  "frame-src 'none'",
])
export const requiredWebHeaderCspFragments = Object.freeze([
  ...requiredWebCspFragments,
  "frame-ancestors 'none'",
])

const requiredPermissionsPolicyFragments = [
  'camera=()',
  'microphone=()',
  'geolocation=()',
  'payment=()',
  'usb=()',
  'serial=()',
  'bluetooth=()',
]
const requiredAzureStaticWebAppFallbackExcludes = [
  '/assets/*',
  '/events/*',
  '/models/*',
  '/icons/*',
  '/locales/*',
  '/manifest.webmanifest',
  '/sw.js',
  '/offline.html',
  '/offline-i18n.js',
  '/project-assets.json',
  '/inference-runtime.json',
  '/favicon.svg',
]
const requiredStaticRedirectPassthroughs = [
  ['/assets/*', '/assets/:splat'],
  ['/events/*', '/events/:splat'],
  ['/models/*', '/models/:splat'],
  ['/icons/*', '/icons/:splat'],
  ['/locales/*', '/locales/:splat'],
  ['/manifest.webmanifest', '/manifest.webmanifest'],
  ['/sw.js', '/sw.js'],
  ['/offline.html', '/offline.html'],
  ['/offline-i18n.js', '/offline-i18n.js'],
  ['/project-assets.json', '/project-assets.json'],
  ['/inference-runtime.json', '/inference-runtime.json'],
  ['/favicon.svg', '/favicon.svg'],
]
const requiredVercelSecurityHeaders = [
  'Content-Security-Policy',
  'X-Content-Type-Options',
  'Referrer-Policy',
  'Permissions-Policy',
]

export function extractHtmlCsp(html) {
  if (typeof html !== 'string') return null

  for (const match of html.matchAll(/<meta\b[^>]*>/gi)) {
    const tag = match[0]
    if (!/\bhttp-equiv\s*=\s*["']Content-Security-Policy["']/i.test(tag)) continue
    return tag.match(/\bcontent\s*=\s*(["'])([\s\S]*?)\1/i)?.[2] ?? null
  }
  return null
}

export function verifyCspPolicy(csp, requiredFragments, label, issues = [], options = {}) {
  if (typeof csp !== 'string') {
    issues.push(`${label} must be a string`)
    return issues
  }
  for (const fragment of requiredFragments) {
    if (!csp.includes(fragment)) {
      issues.push(`${label} must include ${fragment}`)
    }
  }
  if (csp.includes("'unsafe-eval'")) {
    issues.push(`${label} must not allow unsafe-eval`)
  }
  if (/default-src\s+\*/.test(csp)) {
    issues.push(`${label} must not use default-src *`)
  }
  for (const fragment of options.forbiddenFragments ?? []) {
    if (csp.includes(fragment)) {
      issues.push(`${label} must not include ${fragment}`)
    }
  }
  return issues
}

export function verifyStaticHostingHeaders(source, issues = []) {
  if (typeof source !== 'string') {
    issues.push('_headers must be a text document')
    return issues
  }
  if (!source.includes('/*')) {
    issues.push('_headers must define a /* route for static-hosting security headers')
  }

  const csp = extractStaticHeader(source, 'Content-Security-Policy')
  if (!csp) {
    issues.push('_headers must include Content-Security-Policy')
  } else {
    verifyCspPolicy(csp, requiredWebHeaderCspFragments, '_headers Content-Security-Policy', issues)
  }

  const contentTypeOptions = extractStaticHeader(source, 'X-Content-Type-Options')
  if (contentTypeOptions !== 'nosniff') {
    issues.push('_headers must include X-Content-Type-Options: nosniff')
  }
  const referrerPolicy = extractStaticHeader(source, 'Referrer-Policy')
  if (referrerPolicy !== 'no-referrer') {
    issues.push('_headers must include Referrer-Policy: no-referrer')
  }

  const permissionsPolicy = extractStaticHeader(source, 'Permissions-Policy')
  if (!permissionsPolicy) {
    issues.push('_headers must include Permissions-Policy')
  } else {
    appendPermissionsPolicyIssues(permissionsPolicy, '_headers Permissions-Policy', issues)
  }
  return issues
}

export function verifyStaticHostingRedirects(source, issues = []) {
  if (typeof source !== 'string') {
    issues.push('_redirects must be a text document')
    return issues
  }
  const rules = source
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line && !line.startsWith('#'))
    .map((line) => line.split(/\s+/))

  if (rules.length === 0) {
    issues.push('_redirects must define static-hosting redirect rules')
    return issues
  }

  for (const parts of rules) {
    const [from, to, status] = parts
    if (!from || !to || !status) {
      issues.push(`_redirects rule must include source, destination, and status: ${parts.join(' ')}`)
      continue
    }
    if (parts.length !== 3) {
      issues.push(`_redirects rule must not include extra fields: ${parts.join(' ')}`)
    }
    if (status !== '200') {
      issues.push(`_redirects rule must use 200 rewrite status: ${parts.join(' ')}`)
    }
    if (/^https?:\/\//i.test(to)) {
      issues.push(`_redirects rule must not target an external URL: ${parts.join(' ')}`)
    }
  }

  for (const [from, to] of requiredStaticRedirectPassthroughs) {
    if (!rules.some((rule) => rule[0] === from && rule[1] === to && rule[2] === '200')) {
      issues.push(`_redirects must include passthrough rule: ${from} ${to} 200`)
    }
  }

  const fallbackIndex = rules.findIndex((rule) => rule[0] === '/*' && rule[1] === '/index.html' && rule[2] === '200')
  if (fallbackIndex < 0) {
    issues.push('_redirects must include SPA fallback rule: /* /index.html 200')
  } else if (fallbackIndex !== rules.length - 1) {
    issues.push('_redirects SPA fallback rule must be the final rule')
  }
  return issues
}

export function verifyAzureStaticWebAppConfig(config, issues = []) {
  if (!isRecord(config)) {
    issues.push('staticwebapp.config.json must be a JSON object')
    return issues
  }

  if (config.navigationFallback?.rewrite !== '/index.html') {
    issues.push('staticwebapp.config.json navigationFallback.rewrite must be /index.html')
  }
  const fallbackExcludes = config.navigationFallback?.exclude
  if (!Array.isArray(fallbackExcludes)) {
    issues.push('staticwebapp.config.json navigationFallback.exclude must be an array')
  } else {
    for (const route of requiredAzureStaticWebAppFallbackExcludes) {
      if (!fallbackExcludes.includes(route)) {
        issues.push(`staticwebapp.config.json navigationFallback.exclude must include ${route}`)
      }
    }
  }
  if (config.responseOverrides?.['404']?.rewrite !== '/404.html') {
    issues.push('staticwebapp.config.json responseOverrides.404.rewrite must be /404.html')
  }

  const headers = config.globalHeaders
  if (!isRecord(headers)) {
    issues.push('staticwebapp.config.json globalHeaders must be an object')
    return issues
  }

  const csp = headers['content-security-policy'] ?? headers['Content-Security-Policy']
  if (!csp) {
    issues.push('staticwebapp.config.json globalHeaders must include content-security-policy')
  } else {
    verifyCspPolicy(csp, requiredWebHeaderCspFragments, 'staticwebapp.config.json content-security-policy', issues)
  }
  const contentTypeOptions = headers['x-content-type-options'] ?? headers['X-Content-Type-Options']
  if (contentTypeOptions !== 'nosniff') {
    issues.push('staticwebapp.config.json globalHeaders must include x-content-type-options: nosniff')
  }
  const referrerPolicy = headers['referrer-policy'] ?? headers['Referrer-Policy']
  if (referrerPolicy !== 'no-referrer') {
    issues.push('staticwebapp.config.json globalHeaders must include referrer-policy: no-referrer')
  }
  const permissionsPolicy = headers['permissions-policy'] ?? headers['Permissions-Policy']
  if (!permissionsPolicy) {
    issues.push('staticwebapp.config.json globalHeaders must include permissions-policy')
  } else {
    appendPermissionsPolicyIssues(
      permissionsPolicy,
      'staticwebapp.config.json permissions-policy',
      issues,
    )
  }
  return issues
}

export function verifyVercelConfig(config, issues = []) {
  if (!isRecord(config)) {
    issues.push('vercel.json must be a JSON object')
    return issues
  }
  if (config.$schema !== 'https://openapi.vercel.sh/vercel.json') {
    issues.push('vercel.json must declare the Vercel config schema')
  }

  const rewrites = config.rewrites
  if (!Array.isArray(rewrites)) {
    issues.push('vercel.json rewrites must be an array')
  } else if (!rewrites.some((rewrite) => rewrite?.source === '/(.*)' && rewrite?.destination === '/index.html')) {
    issues.push('vercel.json rewrites must route /(.*) to /index.html for SPA fallback')
  }
  for (const rewrite of Array.isArray(rewrites) ? rewrites : []) {
    if (typeof rewrite?.destination === 'string' && /^https?:\/\//i.test(rewrite.destination)) {
      issues.push('vercel.json rewrites must not point SPA fallback to an external URL')
    }
  }

  const globalHeadersRule = Array.isArray(config.headers)
    ? config.headers.find((rule) => rule?.source === '/(.*)' && Array.isArray(rule?.headers))
    : null
  if (!globalHeadersRule) {
    issues.push('vercel.json headers must include a /(.*) security header rule')
    return issues
  }

  const headerMap = new Map(
    globalHeadersRule.headers
      .filter((header) => typeof header?.key === 'string' && typeof header?.value === 'string')
      .map((header) => [header.key.toLowerCase(), header.value]),
  )
  for (const header of requiredVercelSecurityHeaders) {
    if (!headerMap.has(header.toLowerCase())) {
      issues.push(`vercel.json headers must include ${header}`)
    }
  }

  const csp = headerMap.get('content-security-policy')
  if (csp) {
    verifyCspPolicy(csp, requiredWebHeaderCspFragments, 'vercel.json Content-Security-Policy', issues)
  }
  if (headerMap.get('x-content-type-options') !== 'nosniff') {
    issues.push('vercel.json headers must include X-Content-Type-Options: nosniff')
  }
  if (headerMap.get('referrer-policy') !== 'no-referrer') {
    issues.push('vercel.json headers must include Referrer-Policy: no-referrer')
  }
  const permissionsPolicy = headerMap.get('permissions-policy')
  if (permissionsPolicy) {
    appendPermissionsPolicyIssues(permissionsPolicy, 'vercel.json Permissions-Policy', issues)
  }
  return issues
}

function appendPermissionsPolicyIssues(policy, label, issues) {
  if (typeof policy !== 'string') {
    issues.push(`${label} must be a string`)
    return
  }
  for (const fragment of requiredPermissionsPolicyFragments) {
    if (!policy.includes(fragment)) {
      issues.push(`${label} must include ${fragment}`)
    }
  }
}

function extractStaticHeader(source, headerName) {
  const escapedHeaderName = escapeRegExp(headerName)
  return source.match(new RegExp(`^\\s*${escapedHeaderName}\\s*:\\s*(.+)$`, 'im'))?.[1]?.trim() ?? null
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

function isRecord(value) {
  return Boolean(value) && typeof value === 'object' && !Array.isArray(value)
}
