import type { OkfCandidateResult } from './okf-candidate-result';

export interface OkfCandidatesApproveResponse201 {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
