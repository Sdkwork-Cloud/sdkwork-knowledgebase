import type { KnowledgeRetrievalProfile } from './knowledge-retrieval-profile';

export interface RetrievalProfilesUpdateResponse {
  code: 0;
  data: unknown & { item: KnowledgeRetrievalProfile; };
  /** Server-owned request correlation id. */
  traceId: string;
}
