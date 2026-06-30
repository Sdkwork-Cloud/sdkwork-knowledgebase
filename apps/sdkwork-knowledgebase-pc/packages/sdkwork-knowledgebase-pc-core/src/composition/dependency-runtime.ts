import type { SdkworkDependencySdkBaseUrls } from '../config/runtimeConfig.js';
import { sdkworkKnowledgebasePcSdkInventory } from './sdk-inventory.js';

export function listDependencySdkWorkspaces(): string[] {
  return [...sdkworkKnowledgebasePcSdkInventory];
}

export function buildDependencySdkBaseUrls(input: {
  appApiBaseUrl: string;
  iamAppApiBaseUrl: string;
  driveAppApiBaseUrl: string;
}): Record<string, SdkworkDependencySdkBaseUrls> {
  const result: Record<string, SdkworkDependencySdkBaseUrls> = {};

  for (const workspace of sdkworkKnowledgebasePcSdkInventory) {
    if (workspace === 'sdkwork-iam-app-sdk') {
      result[workspace] = { appApiBaseUrl: input.iamAppApiBaseUrl };
      continue;
    }
    if (workspace === 'sdkwork-drive-app-sdk') {
      result[workspace] = { appApiBaseUrl: input.driveAppApiBaseUrl };
      continue;
    }
    result[workspace] = { appApiBaseUrl: input.appApiBaseUrl };
  }

  return result;
}
