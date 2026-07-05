import {
  bindKnowledgebaseSessionStore,
  configureKnowledgebaseAppSdk,
  configureKnowledgebaseBackendSdk,
  configureKnowledgebaseDriveAppSdk,
  createHostAdapter,
  createKnowledgebaseAppSdkClient,
  createKnowledgebaseBackendSdkClient,
  createKnowledgebaseDriveAppSdkClient,
  createKnowledgebaseSessionTokenManager,
  createRuntimeConfig,
  createSessionStore,
  setKnowledgebaseApiEnabled,
  setKnowledgebaseBackendApiEnabled,
  isKnowledgebaseAppApiConfigured,
  isKnowledgebaseBackendApiConfigured,
  type KnowledgebasePcRuntime,
} from 'sdkwork-knowledgebase-pc-core';

import { getKnowledgebasePcSdkPorts } from './sdkPorts';
import { syncHostSessionIntoKnowledgebaseStore } from './sessionBridge';

export function createHostManagedKnowledgebaseRuntime(): KnowledgebasePcRuntime {
  const config = createRuntimeConfig({
    ...import.meta.env,
    VITE_SDKWORK_KNOWLEDGEBASE_TOKEN_STORAGE: 'memory',
    VITE_SDKWORK_KNOWLEDGEBASE_TOKEN_MANAGER_MODE: 'appbase-global',
  });
  const session = createSessionStore();
  const tokenManager = createKnowledgebaseSessionTokenManager(session);
  const ports = getKnowledgebasePcSdkPorts();
  const appSdkClient = createKnowledgebaseAppSdkClient({
    config,
    sdkClient: ports.getKnowledgebaseClient(),
    tokenManager,
  });
  const driveSdkClient = createKnowledgebaseDriveAppSdkClient({
    config,
    sdkClient: ports.getDriveClient(),
    tokenManager,
  });

  const backendSdkClient = createKnowledgebaseBackendSdkClient({
    config,
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
  syncHostSessionIntoKnowledgebaseStore(session);

  return {
    config,
    sdk: {
      app: appSdkClient,
      backend: backendSdkClient,
      drive: driveSdkClient,
    },
    session,
    host: createHostAdapter(),
  };
}
