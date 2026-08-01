import type { KnowledgeEngineProviderBinding } from './knowledge-engine-provider-binding';

export interface SpacesProviderBindingsRetrieveResponse {
  code: 0;
  data: unknown & { item: KnowledgeEngineProviderBinding; };
  /** Server-owned request correlation id. */
  traceId: string;
}
