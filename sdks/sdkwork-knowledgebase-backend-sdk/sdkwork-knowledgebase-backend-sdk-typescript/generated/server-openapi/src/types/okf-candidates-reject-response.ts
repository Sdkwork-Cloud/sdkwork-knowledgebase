import type { OkfCandidateResult } from './okf-candidate-result';

export interface OkfCandidatesRejectResponse {
  code: 0;
  data: unknown & { item: OkfCandidateResult; };
  /** Server-owned request correlation id. */
  traceId: string;
}
