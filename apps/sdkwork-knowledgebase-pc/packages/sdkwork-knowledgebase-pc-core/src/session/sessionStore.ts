export interface SessionSnapshot {
  authToken?: string;
  accessToken?: string;
  refreshToken?: string;
  sessionId?: string;
  user?: SessionUserSnapshot;
  context?: SessionAppContextSnapshot;
  updatedAt?: string;
}

export interface SessionUserSnapshot {
  id: string;
  displayName?: string;
  avatarUrl?: string;
  email?: string;
}

export interface SessionAppContextSnapshot {
  tenantId: string;
  userId: string;
  organizationId?: string;
  sessionId?: string;
  appId?: string;
  environment?: string;
  deploymentMode?: string;
  authLevel?: string;
  dataScope?: string[];
  permissionScope?: string[];
  actorId?: string;
  actorKind?: string;
  deviceId?: string;
}

export interface SessionStore {
  getSnapshot(): SessionSnapshot;
  refreshSession(): SessionSnapshot;
  setSession(nextSession: SessionSnapshot): void;
  clearSession(): void;
  subscribe(listener: (snapshot: SessionSnapshot) => void): () => void;
}

export interface SessionStorageLike {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
  removeItem(key: string): void;
}

export const DEFAULT_SESSION_STORAGE_KEY = 'sdkwork-knowledgebase-pc-session';

function readInitialSession(
  storage: SessionStorageLike | undefined,
  storageKey: string,
): SessionSnapshot {
  if (!storage) {
    return {};
  }

  try {
    const raw = storage.getItem(storageKey);
    return raw ? (JSON.parse(raw) as SessionSnapshot) : {};
  } catch {
    return {};
  }
}

export function createSessionStore(
  storage?: SessionStorageLike,
  storageKey = DEFAULT_SESSION_STORAGE_KEY,
): SessionStore {
  let snapshot = readInitialSession(storage, storageKey);
  const listeners = new Set<(snapshot: SessionSnapshot) => void>();

  const emit = () => {
    for (const listener of listeners) {
      listener(snapshot);
    }
  };

  const persist = () => {
    if (!storage) {
      return;
    }
    if (!snapshot.authToken && !snapshot.accessToken && !snapshot.refreshToken) {
      storage.removeItem(storageKey);
      return;
    }
    storage.setItem(storageKey, JSON.stringify(snapshot));
  };

  return {
    getSnapshot: () => snapshot,
    refreshSession() {
      if (!storage) {
        return snapshot;
      }
      snapshot = readInitialSession(storage, storageKey);
      emit();
      return snapshot;
    },
    setSession(nextSession) {
      snapshot = {
        ...nextSession,
        updatedAt: new Date().toISOString(),
      };
      persist();
      emit();
    },
    clearSession() {
      snapshot = {};
      persist();
      emit();
    },
    subscribe(listener) {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
  };
}
