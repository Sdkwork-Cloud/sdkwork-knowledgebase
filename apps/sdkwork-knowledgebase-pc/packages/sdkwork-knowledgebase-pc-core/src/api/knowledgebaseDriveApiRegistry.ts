import type { KnowledgebaseDriveAppSdkClient } from '../sdk/driveAppSdkClient';
import { KnowledgebaseErrorCodes } from '../errors/knowledgebaseErrorCodes';
import { throwKnowledgebaseError } from '../errors/knowledgebaseAppError';
import { isKnowledgebaseApiAvailable } from './knowledgebaseApiRegistry';

let driveSdkClient: KnowledgebaseDriveAppSdkClient | null = null;

export function configureKnowledgebaseDriveAppSdk(client: KnowledgebaseDriveAppSdkClient): void {
  driveSdkClient = client;
}

export function getKnowledgebaseDriveAppSdkClient(): KnowledgebaseDriveAppSdkClient {
  if (!driveSdkClient) {
    throwKnowledgebaseError(KnowledgebaseErrorCodes.API_UNAVAILABLE_DRIVE);
  }
  return driveSdkClient;
}

export function isKnowledgebaseDriveApiAvailable(): boolean {
  return isKnowledgebaseApiAvailable() && driveSdkClient !== null;
}
