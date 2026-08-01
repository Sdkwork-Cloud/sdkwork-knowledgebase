import type { KnowledgeRetrievalTrace } from './knowledge-retrieval-trace';

export interface RetrievalTracesRetrieveResponse {
  code: 0;
  data: unknown & { item: KnowledgeRetrievalTrace; };
  /** Server-owned request correlation id. */
  traceId: string;
}
