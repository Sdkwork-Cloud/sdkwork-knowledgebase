import {
  createClient as createGeneratedKnowledgebaseAppClient,
  SdkworkKnowledgebaseAppClient,
} from '../generated/server-openapi/src/index';
import type { SdkworkAppConfig } from '../generated/server-openapi/src/types/common';

export { SdkworkKnowledgebaseAppClient, createGeneratedKnowledgebaseAppClient };
export type { SdkworkAppConfig };
export * from '../generated/server-openapi/src/types';
export * from '../generated/server-openapi/src/api';
export * from '../generated/server-openapi/src/http';
export * from '../generated/server-openapi/src/auth';

export function createKnowledgebaseAppClient(config: SdkworkAppConfig): SdkworkKnowledgebaseAppClient {
  return createGeneratedKnowledgebaseAppClient(config);
}

export function createClient(config: SdkworkAppConfig): SdkworkKnowledgebaseAppClient {
  return createKnowledgebaseAppClient(config);
}
