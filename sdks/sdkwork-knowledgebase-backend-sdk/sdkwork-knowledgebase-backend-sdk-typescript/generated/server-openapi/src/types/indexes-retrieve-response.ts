import type { KnowledgeIndex } from './knowledge-index';

export interface IndexesRetrieveResponse {
  code: 0;
  data: unknown & { item: KnowledgeIndex; };
  /** Server-owned request correlation id. */
  traceId: string;
}
