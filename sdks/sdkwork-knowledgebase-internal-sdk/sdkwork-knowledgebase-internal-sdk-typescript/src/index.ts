import {
  createClient as createGeneratedInternalClient,
  SdkworkKnowledgebaseInternalClient,
} from '../generated/server-openapi/src/index';
import type { SdkworkCustomConfig } from '../generated/server-openapi/src/types/common';

export { SdkworkKnowledgebaseInternalClient, createGeneratedInternalClient };
export type { SdkworkCustomConfig };
export * from '../generated/server-openapi/src/types';
export * from '../generated/server-openapi/src/api';
export * from '../generated/server-openapi/src/http';
export * from '../generated/server-openapi/src/auth';

export function createKnowledgebaseInternalClient(
  config: SdkworkCustomConfig,
): SdkworkKnowledgebaseInternalClient {
  return createGeneratedInternalClient(config);
}

export function createClient(
  config: SdkworkCustomConfig,
): SdkworkKnowledgebaseInternalClient {
  return createKnowledgebaseInternalClient(config);
}
