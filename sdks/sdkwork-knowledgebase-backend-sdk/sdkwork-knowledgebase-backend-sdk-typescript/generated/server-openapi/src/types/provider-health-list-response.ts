import type { KnowledgeProviderHealth } from './knowledge-provider-health';

export interface ProviderHealthListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
