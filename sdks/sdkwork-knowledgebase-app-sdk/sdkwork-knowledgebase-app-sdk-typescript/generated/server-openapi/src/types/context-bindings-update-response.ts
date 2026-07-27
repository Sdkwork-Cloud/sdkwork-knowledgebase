import type { KnowledgeSpaceContextBinding } from './knowledge-space-context-binding';

export interface ContextBindingsUpdateResponse {
  code: 0;
  data: unknown & { item: KnowledgeSpaceContextBinding; };
  /** Server-owned request correlation id. */
  traceId: string;
}
