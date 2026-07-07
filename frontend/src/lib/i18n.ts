import { ref, computed } from "vue";
import { invokeCommand } from "./tauri";

type NestedMessages = Record<string, any>;
const currentLocale = ref(localStorage.getItem("monogatari-locale") || "en");
const messages = ref<NestedMessages>({});
const loaded = ref(false);

function getNestedValue(obj: any, path: string): string | undefined {
  return path.split(".").reduce((current: any, key: string) => current?.[key], obj);
}

export async function loadI18n(locale?: string) {
  const lang = locale || currentLocale.value;
  try {
    const result = await invokeCommand<NestedMessages>("load_locale", { locale: lang }, {});
    messages.value = result;
    currentLocale.value = lang;
    localStorage.setItem("monogatari-locale", lang);
    loaded.value = true;
  } catch {
    try {
      const resp = await fetch("/locales/" + lang + ".json");
      if (resp.ok) messages.value = await resp.json();
    } catch {}
    currentLocale.value = lang;
    localStorage.setItem("monogatari-locale", lang);
    loaded.value = true;
  }
}

export function t(key: string, fallback?: string): string {
  const value = getNestedValue(messages.value, key);
  if (value !== undefined) return value;
  if (messages.value[key]) return messages.value[key];
  return fallback || key;
}

export function setLocale(locale: string) {
  loadI18n(locale);
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