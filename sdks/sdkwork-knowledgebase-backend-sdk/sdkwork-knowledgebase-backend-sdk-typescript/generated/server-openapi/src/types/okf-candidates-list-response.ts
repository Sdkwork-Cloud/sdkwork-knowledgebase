import type { OkfCandidateResult } from './okf-candidate-result';
import type { PageInfo } from './page-info';

export interface OkfCandidatesListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
