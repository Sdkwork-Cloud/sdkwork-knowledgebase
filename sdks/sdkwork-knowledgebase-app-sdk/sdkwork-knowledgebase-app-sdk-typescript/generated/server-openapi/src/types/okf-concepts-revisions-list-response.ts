import type { KnowledgeOkfConceptRevisionList } from './knowledge-okf-concept-revision-list';

export interface OkfConceptsRevisionsListResponse {
  code: 0;
  data: unknown & KnowledgeOkfConceptRevisionList;
  /** Server-owned request correlation id. */
  traceId: string;
}
