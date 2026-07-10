import { computed, readonly, ref } from 'vue'
import { invokeCommand } from './tauri'

type MessageTable = Record<string, string>
type LocaleDocument = { locale?: string; strings?: MessageTable } | MessageTable
type MessageParams = Record<string, string | number>

export interface SupportedLocale {
  code: string
  label: string
  shortLabel: string
}

export const supportedLocales: readonly SupportedLocale[] = [
  { code: 'en', label: 'English', shortLabel: 'EN' },
  { code: 'zh-CN', label: '简体中文', shortLabel: '中' },
  { code: 'ja-JP', label: '日本語', shortLabel: '日' },
  { code: 'ko-KR', label: '한국어', shortLabel: '한' },
]

const localeCodes = new Set(supportedLocales.map((item) => item.code))
const currentLocale = ref(resolveInitialLocale())
const messages = ref<MessageTable>({})
const loaded = ref(false)
let loadSequence = 0

function resolveInitialLocale(): string {
  const stored = localStorage.getItem('monogatari-locale')
  if (stored && localeCodes.has(stored)) return stored

  const browserLocale = typeof navigator === 'undefined' ? 'en' : navigator.language
  if (browserLocale.toLowerCase().startsWith('zh')) return 'zh-CN'
  if (browserLocale.toLowerCase().startsWith('ja')) return 'ja-JP'
  if (browserLocale.toLowerCase().startsWith('ko')) return 'ko-KR'
  return 'en'
}

function normalizeLocale(locale: string): string {
  return localeCodes.has(locale) ? locale : 'en'
}

function getMessageTable(document: LocaleDocument | null | undefined): MessageTable {
  if (!document || typeof document !== 'object') return {}
  const candidate = 'strings' in document ? document.strings : document
  if (!candidate || typeof candidate !== 'object') return {}

  return Object.fromEntries(
    Object.entries(candidate).filter((entry): entry is [string, string] => typeof entry[1] === 'string'),
  )
}

function hasMessages(document: LocaleDocument | null | undefined): boolean {
  return Object.keys(getMessageTable(document)).length > 0
}

async function fetchLocale(locale: string): Promise<LocaleDocument> {
  const base = import.meta.env.BASE_URL || '/'
  const normalizedBase = base.endsWith('/') ? base : `${base}/`
  const response = await fetch(`${normalizedBase}locales/${encodeURIComponent(locale)}.json`)
  if (!response.ok) return {}
  return await response.json() as LocaleDocument
}

async function loadLocaleDocument(locale: string): Promise<LocaleDocument> {
  try {
    const result = await invokeCommand<LocaleDocument>('load_locale', { locale })
    if (hasMessages(result)) return result
  } catch {
    // Browser builds and projects without the requested locale use public assets.
  }

  try {
    return await fetchLocale(locale)
  } catch {
    return {}
  }
}

function formatMessage(value: string, params?: MessageParams): string {
  if (!params) return value
  return value.replace(/\{([a-zA-Z0-9_]+)\}/g, (token, key: string) => (
    Object.prototype.hasOwnProperty.call(params, key) ? String(params[key]) : token
  ))
}

function applyDocumentLocale(locale: string) {
  document.documentElement.lang = locale
  document.documentElement.dir = 'ltr'
}

export async function loadI18n(locale?: string): Promise<void> {
  const targetLocale = normalizeLocale(locale || currentLocale.value)
  const requestId = ++loadSequence
  const [englishDocument, targetDocument] = await Promise.all([
    loadLocaleDocument('en'),
    targetLocale === 'en' ? Promise.resolve<LocaleDocument>({}) : loadLocaleDocument(targetLocale),
  ])

  if (requestId !== loadSequence) return

  messages.value = {
    ...getMessageTable(englishDocument),
    ...getMessageTable(targetDocument),
  }
  currentLocale.value = targetLocale
  localStorage.setItem('monogatari-locale', targetLocale)
  applyDocumentLocale(targetLocale)
  loaded.value = true
}

export function t(key: string, fallback?: string, params?: MessageParams): string {
  return formatMessage(messages.value[key] ?? fallback ?? key, params)
}

export function hasTranslation(key: string): boolean {
  return Object.prototype.hasOwnProperty.call(messages.value, key)
}

export async function setLocale(locale: string): Promise<void> {
  await loadI18n(locale)
}

export function useI18n() {
  return {
    locale: readonly(computed(() => currentLocale.value)),
    loaded: readonly(computed(() => loaded.value)),
    supportedLocales,
    t,
    hasTranslation,
    loadI18n,
    setLocale,
  }
}
