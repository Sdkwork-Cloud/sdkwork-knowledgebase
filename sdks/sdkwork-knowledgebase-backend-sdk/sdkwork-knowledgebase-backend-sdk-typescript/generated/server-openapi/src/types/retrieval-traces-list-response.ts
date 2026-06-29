import type { KnowledgeRetrievalTrace } from './knowledge-retrieval-trace';
import type { PageInfo } from './page-info';

export interface RetrievalTracesListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
