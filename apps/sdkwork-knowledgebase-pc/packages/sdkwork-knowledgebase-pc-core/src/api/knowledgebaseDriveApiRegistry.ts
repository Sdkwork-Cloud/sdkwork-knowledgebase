import type { KnowledgebaseDriveAppSdkClient } from '../sdk/driveAppSdkClient';
import { isKnowledgebaseApiAvailable } from './knowledgebaseApiRegistry';

let driveSdkClient: KnowledgebaseDriveAppSdkClient | null = null;

export function configureKnowledgebaseDriveAppSdk(client: KnowledgebaseDriveAppSdkClient): void {
  driveSdkClient = client;
}

export function getKnowledgebaseDriveAppSdkClient(): KnowledgebaseDriveAppSdkClient {
  if (!driveSdkClient) {
    throw new Error('Knowledgebase Drive app SDK is not configured.');
  }
  return driveSdkClient;
}

export function isKnowledgebaseDriveApiAvailable(): boolean {
  return isKnowledgebaseApiAvailable() && driveSdkClient !== null;
}
