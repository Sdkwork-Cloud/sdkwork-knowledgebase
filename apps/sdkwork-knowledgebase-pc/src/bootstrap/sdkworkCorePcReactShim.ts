import { createRuntimeConfig } from 'sdkwork-knowledgebase-pc-core/config/runtimeConfig';
import { DEFAULT_SESSION_STORAGE_KEY } from 'sdkwork-knowledgebase-pc-core/session/sessionStore';

export interface PcReactRuntimeSession {
  accessToken?: string;
  authToken?: string;
  refreshToken?: string;
}

const SESSION_STORAGE_KEY = DEFAULT_SESSION_STORAGE_KEY;

function readStorage(): Storage | undefined {
  if (typeof window === 'undefined') {
    return undefined;
  }

  const tokenStorage = createRuntimeConfig(import.meta.env).auth.tokenStorage;
  if (tokenStorage === 'browser-local') {
    return window.localStorage;
  }
  if (tokenStorage === 'browser-session' || tokenStorage === 'os-secure-storage') {
    return window.sessionStorage;
  }
  return undefined;
}

function normalizeToken(value: unknown): string | undefined {
  const normalized = typeof value === 'string' ? value.trim() : '';
  return normalized.replace(/^Bearer\s+/i, '') || undefined;
}

function readStoredSession(): PcReactRuntimeSession {
  const storage = readStorage();
  if (!storage) {
    return {};
  }

  try {
    const raw = storage.getItem(SESSION_STORAGE_KEY);
    if (!raw) {
      return {};
    }
    const parsed = JSON.parse(raw) as PcReactRuntimeSession;
    return {
      accessToken: normalizeToken(parsed.accessToken),
      authToken: normalizeToken(parsed.authToken),
      refreshToken: normalizeToken(parsed.refreshToken),
    };
  } catch {
    return {};
  }
}

export function readPcReactRuntimeSession(): PcReactRuntimeSession {
  return readStoredSession();
}

export function persistPcReactRuntimeSession(
  tokens: PcReactRuntimeSession,
): PcReactRuntimeSession {
  const current = readStoredSession();
  const next = {
    accessToken: tokens.accessToken !== undefined
      ? normalizeToken(tokens.accessToken)
      : current.accessToken,
    authToken: tokens.authToken !== undefined
      ? normalizeToken(tokens.authToken)
      : current.authToken,
    refreshToken: tokens.refreshToken !== undefined
      ? normalizeToken(tokens.refreshToken)
      : current.refreshToken,
  };
  const storage = readStorage();

  if (storage) {
    if (!next.accessToken && !next.authToken && !next.refreshToken) {
      storage.removeItem(SESSION_STORAGE_KEY);
    } else {
      storage.setItem(SESSION_STORAGE_KEY, JSON.stringify(next));
    }
  }

  return next;
}

export function clearPcReactRuntimeSession(): void {
  readStorage()?.removeItem(SESSION_STORAGE_KEY);
}

export function resolveAppClientAccessToken(): string {
  return normalizeToken(readStoredSession().accessToken) ?? '';
}

export function getAppClientWithSession(): never {
  throw new Error(
    'Knowledgebase PC does not expose a generic app client. Use the injected appbase IAM runtime or Knowledgebase SDK service boundary.',
  );
}
