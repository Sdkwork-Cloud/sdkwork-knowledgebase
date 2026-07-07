import type { OkfCandidateResult } from './okf-candidate-result';

export interface OkfCandidatesApproveResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
