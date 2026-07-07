import type { KnowledgeSpaceContextBinding } from './knowledge-space-context-binding';

export interface SpacesContextBindingsResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
