import {
  createHostAdapter,
  createKnowledgebaseAppSdkClient,
  createKnowledgebaseDriveAppSdkClient,
  createKnowledgebaseSessionTokenManager,
  createRuntimeConfig,
  createSessionStore,
  DEFAULT_SESSION_STORAGE_KEY,
  configureKnowledgebaseAppSdk,
  configureKnowledgebaseDriveAppSdk,
  bindKnowledgebaseSessionStore,
  setKnowledgebaseApiEnabled,
  isKnowledgebaseAppApiConfigured,
  invokeDesktopCommand,
  isTauriDesktopRuntime,
  type KnowledgebasePcRuntime,
  type SessionStorageLike,
} from 'sdkwork-knowledgebase-pc-core';
import {
  configureKnowledgebaseBackendSdk,
  createKnowledgebaseBackendSdkClient,
  isKnowledgebaseBackendApiConfigured,
  setKnowledgebaseBackendApiEnabled,
} from 'sdkwork-knowledgebase-pc-admin-core';

import { createKnowledgebaseIamRuntime } from './knowledgebaseIamRuntime';
import { primePcReactRuntimeSessionCache } from './sdkworkCorePcReactShim';

export function createKnowledgebasePcRuntime(): KnowledgebasePcRuntime {
  const config = createRuntimeConfig(import.meta.env);
  const resolvedStorage = resolveSessionStorage(config.auth.tokenStorage);
  const session = createSessionStore(resolvedStorage.storage);
  void resolvedStorage.hydrated?.then(() => {
    const current = session.getSnapshot();
    if (!current.authToken && !current.accessToken && !current.refreshToken) {
      session.refreshSession();
    }
  });
  const tokenManager = createKnowledgebaseSessionTokenManager(session);
  const appSdkClient = createKnowledgebaseAppSdkClient({
    config,
    tokenManager,
  });
  const backendSdkClient = createKnowledgebaseBackendSdkClient({
    config,
    tokenManager,
  });
  const driveSdkClient = createKnowledgebaseDriveAppSdkClient({
    config,
    tokenManager,
  });
  const iamRuntime = createKnowledgebaseIamRuntime({
    config,
    sdkClients: [appSdkClient, backendSdkClient, driveSdkClient],
    session,
    tokenManager,
  });

  primePcReactRuntimeSessionCache(session.getSnapshot());
  session.subscribe((snapshot) => {
    primePcReactRuntimeSessionCache(snapshot);
  });

  bindKnowledgebaseSessionStore(session);
  configureKnowledgebaseAppSdk(appSdkClient);
  configureKnowledgebaseBackendSdk(backendSdkClient);
  configureKnowledgebaseDriveAppSdk(driveSdkClient);
  setKnowledgebaseApiEnabled(
    config.auth.tokenManagerMode !== 'test'
    && isKnowledgebaseAppApiConfigured(config),
  );
  setKnowledgebaseBackendApiEnabled(
    config.auth.tokenManagerMode !== 'test'
    && isKnowledgebaseBackendApiConfigured(config),
  );

  return {
    config,
    auth: {
      iamRuntime,
    },
    sdk: {
      app: appSdkClient,
      drive: driveSdkClient,
    },
    session,
    host: createHostAdapter(),
  };
}

interface ResolvedSessionStorage {
  storage?: SessionStorageLike;
  hydrated?: Promise<void>;
}

function resolveSessionStorage(
  tokenStorage: KnowledgebasePcRuntime['config']['auth']['tokenStorage'],
): ResolvedSessionStorage {
  if (typeof window === 'undefined') {
    return {};
  }
  if (tokenStorage === 'browser-local' || tokenStorage === 'browser-session') {
    migrateLegacyBrowserSession();
    return { storage: window.localStorage };
  }
  if (tokenStorage === 'os-secure-storage') {
    return createDesktopSecureSessionStorage() ?? {};
  }
  return {};
}

function migrateLegacyBrowserSession(): void {
  const legacySession = window.sessionStorage.getItem(DEFAULT_SESSION_STORAGE_KEY);
  if (legacySession && !window.localStorage.getItem(DEFAULT_SESSION_STORAGE_KEY)) {
    window.localStorage.setItem(DEFAULT_SESSION_STORAGE_KEY, legacySession);
  }
  if (legacySession) {
    window.sessionStorage.removeItem(DEFAULT_SESSION_STORAGE_KEY);
  }
}

function createDesktopSecureSessionStorage(): ResolvedSessionStorage | undefined {
  if (!isTauriDesktopRuntime()) {
    return undefined;
  }

  const memory = new Map<string, string>();
  let mutationVersion = 0;
  const hydrated = invokeDesktopCommand<string | null>('read_secure_session_value', {
    request: { key: DEFAULT_SESSION_STORAGE_KEY },
  })
    .then((value) => {
      if (value && mutationVersion === 0) {
        memory.set(DEFAULT_SESSION_STORAGE_KEY, value);
      }
    })
    .catch(() => undefined)
    .then(() => undefined);

  return {
    hydrated,
    storage: {
      getItem(key: string) {
        return memory.get(key) ?? null;
      },
      setItem(key: string, value: string) {
        mutationVersion += 1;
        memory.set(key, value);
        void invokeDesktopCommand('write_secure_session_value', { request: { key, value } }).catch(() => {
          memory.delete(key);
        });
      },
      removeItem(key: string) {
        mutationVersion += 1;
        memory.delete(key);
        void invokeDesktopCommand('remove_secure_session_value', { request: { key } }).catch(() => undefined);
      },
    },
  };
}
