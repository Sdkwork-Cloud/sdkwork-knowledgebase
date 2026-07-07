import type { KnowledgeSpace } from './knowledge-space';

export interface SpacesUpdateResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
