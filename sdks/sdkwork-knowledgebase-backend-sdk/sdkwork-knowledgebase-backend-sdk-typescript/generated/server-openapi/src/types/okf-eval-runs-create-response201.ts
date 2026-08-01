import type { OkfQualityRun } from './okf-quality-run';

export interface OkfEvalRunsCreateResponse201 {
  code: 0;
  data: unknown & { item: OkfQualityRun; };
  /** Server-owned request correlation id. */
  traceId: string;
}
