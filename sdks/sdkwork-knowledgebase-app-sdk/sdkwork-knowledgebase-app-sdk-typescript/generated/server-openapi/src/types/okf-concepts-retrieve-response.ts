import type { OkfConceptSummary } from './okf-concept-summary';

export interface OkfConceptsRetrieveResponse {
  code: 0;
  data: unknown & { item: OkfConceptSummary; };
  /** Server-owned request correlation id. */
  traceId: string;
}
