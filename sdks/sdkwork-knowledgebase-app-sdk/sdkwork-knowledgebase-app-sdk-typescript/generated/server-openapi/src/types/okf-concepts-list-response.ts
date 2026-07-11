import type { OkfConceptSummaryList } from './okf-concept-summary-list';

export interface OkfConceptsListResponse {
  code: 0;
  data: unknown & OkfConceptSummaryList;
  /** Server-owned request correlation id. */
  traceId: string;
}
