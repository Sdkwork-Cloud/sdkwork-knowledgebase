import type { KnowledgeSpaceContextBinding } from './knowledge-space-context-binding';

export interface ContextBindingsRetrieveResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
