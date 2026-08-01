import type { KnowledgeProviderHealth } from './knowledge-provider-health';

export interface ProviderHealthListResponse {
  code: 0;
  data: unknown & { item: KnowledgeProviderHealth; };
  /** Server-owned request correlation id. */
  traceId: string;
}
