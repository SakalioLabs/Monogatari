import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'

import {
  extractHtmlCsp,
  requiredWebHeaderCspFragments,
  verifyAzureStaticWebAppConfig,
  verifyCspPolicy,
  verifyStaticHostingHeaders,
  verifyStaticHostingRedirects,
  verifyVercelConfig,
} from '../lib/web-hosting-verifier.mjs'

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..')
const csp = requiredWebHeaderCspFragments.join('; ')
const permissions = 'camera=(), microphone=(), geolocation=(), payment=(), usb=(), serial=(), bluetooth=()'
const headers = `/*
  Content-Security-Policy: ${csp}
  X-Content-Type-Options: nosniff
  Referrer-Policy: no-referrer
  Permissions-Policy: ${permissions}
`
const redirects = `/assets/* /assets/:splat 200
/events/* /events/:splat 200
/models/* /models/:splat 200
/icons/* /icons/:splat 200
/locales/* /locales/:splat 200
/manifest.webmanifest /manifest.webmanifest 200
/sw.js /sw.js 200
/offline.html /offline.html 200
/offline-i18n.js /offline-i18n.js 200
/project-assets.json /project-assets.json 200
/inference-runtime.json /inference-runtime.json 200
/favicon.svg /favicon.svg 200
/* /index.html 200
`
const azure = {
  navigationFallback: {
    rewrite: '/index.html',
    exclude: [
      '/assets/*', '/events/*', '/models/*', '/icons/*', '/locales/*',
      '/manifest.webmanifest', '/sw.js', '/offline.html', '/offline-i18n.js',
      '/project-assets.json', '/inference-runtime.json', '/favicon.svg',
    ],
  },
  responseOverrides: { 404: { rewrite: '/404.html' } },
  globalHeaders: {
    'content-security-policy': csp,
    'x-content-type-options': 'nosniff',
    'referrer-policy': 'no-referrer',
    'permissions-policy': permissions,
  },
}
const vercel = {
  $schema: 'https://openapi.vercel.sh/vercel.json',
  headers: [{
    source: '/(.*)',
    headers: [
      { key: 'Content-Security-Policy', value: csp },
      { key: 'X-Content-Type-Options', value: 'nosniff' },
      { key: 'Referrer-Policy', value: 'no-referrer' },
      { key: 'Permissions-Policy', value: permissions },
    ],
  }],
  rewrites: [{ source: '/(.*)', destination: '/index.html' }],
}

test('extracts CSP metadata independent of attribute order', () => {
  const html = `<meta content="${csp}" data-owner="web" http-equiv="Content-Security-Policy">`
  assert.equal(extractHtmlCsp(html), csp)
  assert.equal(extractHtmlCsp('<main></main>'), null)
  assert.equal(extractHtmlCsp(null), null)
})

test('accepts complete static hosting contracts across providers', () => {
  assert.deepEqual(verifyStaticHostingHeaders(headers), [])
  assert.deepEqual(verifyStaticHostingRedirects(redirects), [])
  assert.deepEqual(verifyAzureStaticWebAppConfig(azure), [])
  assert.deepEqual(verifyVercelConfig(vercel), [])
})

test('rejects unsafe CSP and incomplete security headers', () => {
  const issues = verifyCspPolicy(
    "default-src *; script-src 'self' 'unsafe-eval'",
    requiredWebHeaderCspFragments,
    'test CSP',
  )
  assert(issues.includes('test CSP must not allow unsafe-eval'))
  assert(issues.includes('test CSP must not use default-src *'))
  assert(issues.some((issue) => issue.includes("object-src 'none'")))

  const headerIssues = verifyStaticHostingHeaders('/*\n  Referrer-Policy: origin')
  assert(headerIssues.includes('_headers must include Content-Security-Policy'))
  assert(headerIssues.includes('_headers must include X-Content-Type-Options: nosniff'))
  assert(headerIssues.includes('_headers must include Permissions-Policy'))
})

test('rejects external rewrites and requires the SPA fallback to be last', () => {
  const invalid = `${redirects} /proxy https://example.test 301`
  const issues = verifyStaticHostingRedirects(invalid)
  assert(issues.some((issue) => issue.includes('must not target an external URL')))
  assert(issues.some((issue) => issue.includes('must use 200 rewrite status')))
  assert(issues.includes('_redirects SPA fallback rule must be the final rule'))
})

test('provider configs fail closed for malformed and external destinations', () => {
  assert.deepEqual(verifyAzureStaticWebAppConfig(null), [
    'staticwebapp.config.json must be a JSON object',
  ])
  assert.deepEqual(verifyVercelConfig([]), ['vercel.json must be a JSON object'])

  const malformedAzure = structuredClone(azure)
  malformedAzure.globalHeaders['permissions-policy'] = {}
  assert(verifyAzureStaticWebAppConfig(malformedAzure).includes(
    'staticwebapp.config.json permissions-policy must be a string',
  ))

  const malformedVercel = structuredClone(vercel)
  malformedVercel.rewrites = {}
  assert(verifyVercelConfig(malformedVercel).includes('vercel.json rewrites must be an array'))

  const invalidVercel = structuredClone(vercel)
  invalidVercel.rewrites = [{ source: '/(.*)', destination: 'https://example.test' }]
  const issues = verifyVercelConfig(invalidVercel)
  assert(issues.includes('vercel.json rewrites must route /(.*) to /index.html for SPA fallback'))
  assert(issues.includes('vercel.json rewrites must not point SPA fallback to an external URL'))
})

test('release runner delegates hosting policy without redeclaring it', async () => {
  const source = await readFile(path.join(repositoryRoot, 'scripts', 'verify-release.mjs'), 'utf8')
  assert(source.includes("from './lib/web-hosting-verifier.mjs'"))
  assert(!source.includes('function verifyStaticHostingHeaders'))
  assert(!source.includes('const requiredStaticRedirectPassthroughs'))
})
