export const APP_LANGUAGE_STORAGE_KEY = 'app-language';

const SUPPORTED_LANGUAGES = ['zh', 'en'] as const;

export type AppLanguage = (typeof SUPPORTED_LANGUAGES)[number];

export function normalizeAppLanguage(value: string | null | undefined): AppLanguage {
  const raw = (value ?? '').trim().toLowerCase();
  if (!raw) {
    return 'zh';
  }
  if (raw === 'en' || raw.startsWith('en-')) {
    return 'en';
  }
  if (raw === 'zh' || raw.startsWith('zh-')) {
    return 'zh';
  }
  return 'zh';
}

export function readStoredAppLanguage(): AppLanguage | null {
  if (typeof window === 'undefined') {
    return null;
  }

  try {
    const stored = window.localStorage.getItem(APP_LANGUAGE_STORAGE_KEY);
    if (stored) {
      return normalizeAppLanguage(stored);
    }
    const legacy = window.localStorage.getItem('i18nextLng');
    if (legacy) {
      return normalizeAppLanguage(legacy);
    }
  } catch {
    // Ignore storage read errors and fall back to defaults.
  }

  return null;
}

export function persistAppLanguage(language: string): void {
  if (typeof window === 'undefined') {
    return;
  }

  const normalized = normalizeAppLanguage(language);
  try {
    window.localStorage.setItem(APP_LANGUAGE_STORAGE_KEY, normalized);
    window.localStorage.setItem('i18nextLng', normalized);
  } catch {
    // Ignore storage write errors.
  }
}

export function resolveInitialAppLanguage(): AppLanguage {
  return readStoredAppLanguage() ?? normalizeAppLanguage(
    typeof navigator !== 'undefined' ? navigator.language : 'zh',
  );
}

export function resolveKnowledgebaseAuthLocaleFromAppLanguage(
  language: string | null | undefined,
): string | null {
  const normalized = normalizeAppLanguage(language);
  return normalized === 'en' ? 'en-US' : 'zh-CN';
}
