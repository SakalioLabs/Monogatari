import { ref, computed } from "vue";
import { invokeCommand } from "./tauri";

type NestedMessages = Record<string, any>;
const currentLocale = ref(localStorage.getItem("monogatari-locale") || "en");
const messages = ref<NestedMessages>({});
const loaded = ref(false);

function getMessageTable(obj: any): NestedMessages {
  if (obj?.strings && typeof obj.strings === "object") {
    return obj.strings;
  }
  return obj || {};
}

function hasMessages(obj: any): boolean {
  return Object.keys(getMessageTable(obj)).length > 0;
}

function getNestedValue(obj: any, path: string): string | undefined {
  const table = getMessageTable(obj);
  const direct = table[path];
  if (typeof direct === "string") return direct;

  const nested = path.split(".").reduce((current: any, key: string) => current?.[key], table);
  return typeof nested === "string" ? nested : undefined;
}

async function fetchLocale(lang: string): Promise<NestedMessages> {
  const base = import.meta.env.BASE_URL || "/";
  const normalizedBase = base.endsWith("/") ? base : `${base}/`;
  const localeUrl = `${normalizedBase}locales/${encodeURIComponent(lang)}.json`;
  const resp = await fetch(localeUrl);
  if (!resp.ok) return {};
  return await resp.json();
}

export async function loadI18n(locale?: string) {
  const lang = locale || currentLocale.value;
  let nextMessages: NestedMessages = {};

  try {
    const result = await invokeCommand<NestedMessages>("load_locale", { locale: lang });
    if (hasMessages(result)) nextMessages = result;
  } catch {}

  if (!hasMessages(nextMessages)) {
    try {
      const result = await fetchLocale(lang);
      if (hasMessages(result)) nextMessages = result;
    } catch {}
  }

  if (!hasMessages(nextMessages) && lang !== "en") {
    try {
      const result = await fetchLocale("en");
      if (hasMessages(result)) nextMessages = result;
    } catch {}
  }

  messages.value = nextMessages;
  currentLocale.value = lang;
  localStorage.setItem("monogatari-locale", lang);
  loaded.value = true;
}

export function t(key: string, fallback?: string): string {
  const value = getNestedValue(messages.value, key);
  if (value !== undefined) return value;
  if (messages.value[key]) return messages.value[key];
  return fallback || key;
}

export function setLocale(locale: string) {
  void loadI18n(locale);
}

export function useI18n() {
  return {
    locale: computed(() => currentLocale.value),
    t,
    loadI18n,
    setLocale,
    loaded: computed(() => loaded.value),
  };
}
