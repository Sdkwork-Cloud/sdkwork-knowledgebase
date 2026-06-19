import {
  createHostAdapter,
  createKnowledgebaseAppSdkClient,
  createKnowledgebaseSessionTokenManager,
  createRuntimeConfig,
  createSessionStore,
  configureKnowledgebaseAppSdk,
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
  const iamRuntime = createKnowledgebaseIamRuntime({
    config,
    sdkClients: [appSdkClient],
    session,
    tokenManager,
  });

  bindKnowledgebaseSessionStore(session);
  configureKnowledgebaseAppSdk(appSdkClient);
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
