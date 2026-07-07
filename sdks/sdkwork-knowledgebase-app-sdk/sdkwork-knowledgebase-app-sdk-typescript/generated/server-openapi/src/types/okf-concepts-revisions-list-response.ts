import type { KnowledgeOkfConceptRevision } from './knowledge-okf-concept-revision';
import type { PageInfo } from './page-info';

export interface OkfConceptsRevisionsListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
