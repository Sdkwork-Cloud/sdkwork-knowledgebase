import {
  isDevSameOriginApiEnabled,
  KnowledgebaseErrorCodes,
  throwKnowledgebaseError,
  type KnowledgebaseRuntimeConfig,
} from 'sdkwork-knowledgebase-pc-core';
import type { KnowledgebaseBackendSdkClient } from '../sdk/knowledgebaseBackendSdkClient';

let backendSdkClient: KnowledgebaseBackendSdkClient | null = null;
let backendApiEnabled = false;

export function configureKnowledgebaseBackendSdk(client: KnowledgebaseBackendSdkClient): void {
  backendSdkClient = client;
}

export function getKnowledgebaseBackendSdkClient(): KnowledgebaseBackendSdkClient {
  if (!backendSdkClient) {
    throwKnowledgebaseError(KnowledgebaseErrorCodes.API_UNAVAILABLE_SDK);
  }
  return backendSdkClient;
}

export function setKnowledgebaseBackendApiEnabled(enabled: boolean): void {
  backendApiEnabled = enabled;
}

export function isKnowledgebaseBackendApiAvailable(): boolean {
  return backendApiEnabled && backendSdkClient !== null;
}

export function isKnowledgebaseBackendApiConfigured(
  config: KnowledgebaseRuntimeConfig,
  env: Record<string, string | undefined> = import.meta.env as Record<string, string | undefined>,
): boolean {
  return Boolean(config.backendApiBaseUrl || config.appApiBaseUrl || config.sdkBaseUrls.appApiBaseUrl)
    || isDevSameOriginApiEnabled(config, env);
}

export function requireKnowledgebaseBackendApi(operation: string): void {
  if (!isKnowledgebaseBackendApiAvailable()) {
    throwKnowledgebaseError(KnowledgebaseErrorCodes.API_UNAVAILABLE, {
      cause: operation,
    });
  }
}

const KNOWLEDGE_ADMIN_SCOPES = new Set([
  'knowledge.platform.manage',
  'knowledge.admin',
  'knowledge.*',
]);

export function canAccessKnowledgebaseAdminConsole(permissionScope?: string[]): boolean {
  if (!permissionScope?.length) {
    return false;
  }
  return permissionScope.some((scope) => KNOWLEDGE_ADMIN_SCOPES.has(scope));
}
