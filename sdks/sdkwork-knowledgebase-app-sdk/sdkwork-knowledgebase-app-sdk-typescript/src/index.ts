import {
  createClient as createGeneratedKnowledgebaseAppClient,
  SdkworkAppClient,
} from '../generated/server-openapi/src/index';
import type { SdkworkAppConfig } from '../generated/server-openapi/src/types/common';

export { SdkworkAppClient, createGeneratedKnowledgebaseAppClient };
export type { SdkworkAppConfig };
export * from '../generated/server-openapi/src/types';
export * from '../generated/server-openapi/src/api';
export * from '../generated/server-openapi/src/http';
export * from '../generated/server-openapi/src/auth';

export type SdkworkKnowledgebaseAppClient = SdkworkAppClient;

export function createKnowledgebaseAppClient(config: SdkworkAppConfig): SdkworkKnowledgebaseAppClient {
  return createGeneratedKnowledgebaseAppClient(config);
}

export function createClient(config: SdkworkAppConfig): SdkworkKnowledgebaseAppClient {
  return createKnowledgebaseAppClient(config);
}
