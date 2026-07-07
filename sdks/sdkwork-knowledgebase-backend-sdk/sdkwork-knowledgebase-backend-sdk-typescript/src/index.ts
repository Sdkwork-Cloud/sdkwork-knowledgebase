import {
  createClient as createGeneratedBackendClient,
  SdkworkKnowledgebaseBackendClient,
} from '../generated/server-openapi/src/index';
import type { SdkworkBackendConfig } from '../generated/server-openapi/src/types/common';

export { SdkworkKnowledgebaseBackendClient, createGeneratedBackendClient };
export type { SdkworkBackendConfig };
export * from '../generated/server-openapi/src/types';
export * from '../generated/server-openapi/src/api';
export * from '../generated/server-openapi/src/http';
export * from '../generated/server-openapi/src/auth';

export function createClient(
  config: SdkworkBackendConfig,
): SdkworkKnowledgebaseBackendClient {
  return createGeneratedBackendClient(config);
}
