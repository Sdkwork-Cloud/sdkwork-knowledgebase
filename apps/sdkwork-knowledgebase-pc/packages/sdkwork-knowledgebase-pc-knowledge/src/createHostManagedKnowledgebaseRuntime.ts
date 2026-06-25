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

  bindKnowledgebaseSessionStore(session);
  configureKnowledgebaseAppSdk(appSdkClient);
  configureKnowledgebaseDriveAppSdk(driveSdkClient);
  setKnowledgebaseApiEnabled(true);
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
