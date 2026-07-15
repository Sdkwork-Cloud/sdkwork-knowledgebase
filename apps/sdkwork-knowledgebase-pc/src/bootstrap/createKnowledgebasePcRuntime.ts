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
  const session = createSessionStore(resolveSessionStorage(config.auth.tokenStorage));
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

function resolveSessionStorage(
  tokenStorage: KnowledgebasePcRuntime['config']['auth']['tokenStorage'],
): SessionStorageLike | undefined {
  if (typeof window === 'undefined') {
    return undefined;
  }
  if (tokenStorage === 'browser-local' || tokenStorage === 'browser-session') {
    migrateLegacyBrowserSession();
    return window.localStorage;
  }
  if (tokenStorage === 'os-secure-storage') {
    return createDesktopSecureSessionStorage();
  }
  return undefined;
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

function createDesktopSecureSessionStorage(): SessionStorageLike | undefined {
  const tauri = (globalThis as typeof globalThis & {
    __TAURI__?: { core?: { invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> } };
  }).__TAURI__;
  if (!tauri?.core?.invoke) {
    return undefined;
  }

  const memory = new Map<string, string>();
  void tauri.core!.invoke<Record<string, string>>('read_secure_session_snapshot')
    .then((snapshot) => {
      for (const [key, value] of Object.entries(snapshot ?? {})) {
        memory.set(key, value);
      }
    })
    .catch(() => undefined);

  return {
    getItem(key: string) {
      return memory.get(key) ?? null;
    },
    setItem(key: string, value: string) {
      memory.set(key, value);
      void tauri.core!.invoke('write_secure_session_value', { request: { key, value } }).catch(() => {
        memory.delete(key);
      });
    },
    removeItem(key: string) {
      memory.delete(key);
      void tauri.core!.invoke('remove_secure_session_value', { request: { key } }).catch(() => undefined);
    },
  };
}
