import type { OkfConceptSummary } from './okf-concept-summary';

export interface OkfConceptsUpdateResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
