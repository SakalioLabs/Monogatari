import { ref, computed } from 'vue';
import { invokeCommand } from './tauri';

type Messages = Record<string, string>;
const currentLocale = ref('en');
const messages = ref<Messages>({});
const loaded = ref(false);

export async function loadI18n(locale?: string) {
  const lang = locale || currentLocale.value;
  try {
    const result = await invokeCommand<Record<string, string>>('load_locale', { locale: lang }, {});
    messages.value = result;
    currentLocale.value = lang;
    loaded.value = true;
  } catch {
    messages.value = {};
    loaded.value = true;
  }
}

export function t(key: string, fallback?: string): string {
  return messages.value[key] || fallback || key;
}

export function useI18n() {
  return {
    locale: computed(() => currentLocale.value),
    t,
    loadI18n,
    loaded: computed(() => loaded.value),
  };
}
