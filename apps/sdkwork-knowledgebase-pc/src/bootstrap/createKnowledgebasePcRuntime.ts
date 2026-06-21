import {
  createHostAdapter,
  createKnowledgebaseAppSdkClient,
  createKnowledgebaseDriveAppSdkClient,
  createKnowledgebaseSessionTokenManager,
  createRuntimeConfig,
  createSessionStore,
  configureKnowledgebaseAppSdk,
  configureKnowledgebaseDriveAppSdk,
  bindKnowledgebaseSessionStore,
  setKnowledgebaseApiEnabled,
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
  const driveSdkClient = createKnowledgebaseDriveAppSdkClient({
    config,
    tokenManager,
  });
  const iamRuntime = createKnowledgebaseIamRuntime({
    config,
    sdkClients: [appSdkClient, driveSdkClient],
    session,
    tokenManager,
  });

  bindKnowledgebaseSessionStore(session);
  configureKnowledgebaseAppSdk(appSdkClient);
  configureKnowledgebaseDriveAppSdk(driveSdkClient);
  setKnowledgebaseApiEnabled(
    config.auth.tokenManagerMode !== 'test'
    && Boolean(config.appApiBaseUrl || config.sdkBaseUrls.appApiBaseUrl),
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
  if (tokenStorage === 'browser-local') {
    return window.localStorage;
  }
  if (tokenStorage === 'browser-session' || tokenStorage === 'os-secure-storage') {
    // Desktop secure storage is not wired yet; keep session in browser session storage.
    return window.sessionStorage;
  }
  return undefined;
}
