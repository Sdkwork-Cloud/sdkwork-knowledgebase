import type { KnowledgebaseAppSdkClient } from '../sdk/knowledgebaseAppSdkClient';

let appSdkClient: KnowledgebaseAppSdkClient | null = null;
let apiEnabled = false;

export function configureKnowledgebaseAppSdk(client: KnowledgebaseAppSdkClient): void {
  appSdkClient = client;
}

export function getKnowledgebaseAppSdkClient(): KnowledgebaseAppSdkClient {
  if (!appSdkClient) {
    throw new Error('Knowledgebase app SDK client is not configured.');
  }
  return appSdkClient;
}

export function setKnowledgebaseApiEnabled(enabled: boolean): void {
  apiEnabled = enabled;
}

export function isKnowledgebaseApiAvailable(): boolean {
  return apiEnabled && appSdkClient !== null;
}
