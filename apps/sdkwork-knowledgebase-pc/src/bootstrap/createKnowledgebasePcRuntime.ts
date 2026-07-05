import {
  createHostAdapter,
  createKnowledgebaseAppSdkClient,
  createKnowledgebaseBackendSdkClient,
  createKnowledgebaseDriveAppSdkClient,
  createKnowledgebaseSessionTokenManager,
  createRuntimeConfig,
  createSessionStore,
  configureKnowledgebaseAppSdk,
  configureKnowledgebaseBackendSdk,
  configureKnowledgebaseDriveAppSdk,
  bindKnowledgebaseSessionStore,
  setKnowledgebaseApiEnabled,
  setKnowledgebaseBackendApiEnabled,
  isKnowledgebaseAppApiConfigured,
  isKnowledgebaseBackendApiConfigured,
  type KnowledgebasePcRuntime,
  type SessionStorageLike,
} from 'sdkwork-knowledgebase-pc-core';

import { createKnowledgebaseIamRuntime } from './knowledgebaseIamRuntime';

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
      backend: backendSdkClient,
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
  if (tokenStorage === 'browser-local') {
    return window.localStorage;
  }
  if (tokenStorage === 'os-secure-storage') {
    return createDesktopSecureSessionStorage();
  }
  if (tokenStorage === 'browser-session') {
    return window.sessionStorage;
  }
  return undefined;
}

function createDesktopSecureSessionStorage(): SessionStorageLike | undefined {
  const tauri = (globalThis as typeof globalThis & {
    __TAURI__?: { core?: { invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> } };
  }).__TAURI__;
  if (!tauri?.core?.invoke) {
    return window.sessionStorage;
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
