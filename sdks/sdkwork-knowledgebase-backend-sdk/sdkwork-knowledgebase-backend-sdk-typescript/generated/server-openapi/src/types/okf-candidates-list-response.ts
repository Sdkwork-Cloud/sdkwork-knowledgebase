import type { OkfCandidateResult } from './okf-candidate-result';
import type { PageInfo } from './page-info';

export interface OkfCandidatesListResponse {
  code: 0;
  data: unknown & { items: OkfCandidateResult[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
