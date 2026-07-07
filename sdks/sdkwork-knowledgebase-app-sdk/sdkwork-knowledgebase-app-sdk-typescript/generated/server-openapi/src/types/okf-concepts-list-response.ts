import type { OkfConceptSummary } from './okf-concept-summary';
import type { PageInfo } from './page-info';

export interface OkfConceptsListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
