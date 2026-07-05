import {
  bindKnowledgebaseSessionStore,
  configureKnowledgebaseAppSdk,
  configureKnowledgebaseDriveAppSdk,
  createHostAdapter,
  createKnowledgebaseAppSdkClient,
  createKnowledgebaseDriveAppSdkClient,
  createKnowledgebaseSessionTokenManager,
  createRuntimeConfig,
  createSessionStore,
  setKnowledgebaseApiEnabled,
  isKnowledgebaseAppApiConfigured,
  type KnowledgebasePcRuntime,
} from 'sdkwork-knowledgebase-pc-core';
import {
  configureKnowledgebaseBackendSdk,
  createKnowledgebaseBackendSdkClient,
  isKnowledgebaseBackendApiConfigured,
  setKnowledgebaseBackendApiEnabled,
} from 'sdkwork-knowledgebase-pc-admin-core';

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
      drive: driveSdkClient,
    },
    session,
    host: createHostAdapter(),
  };
}
