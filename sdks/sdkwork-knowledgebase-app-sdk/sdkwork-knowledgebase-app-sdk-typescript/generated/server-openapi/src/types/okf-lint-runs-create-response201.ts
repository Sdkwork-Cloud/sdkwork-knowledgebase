import type { OkfQualityRun } from './okf-quality-run';

export interface OkfLintRunsCreateResponse201 {
  code: 0;
  data: unknown & { item: OkfQualityRun; };
  /** Server-owned request correlation id. */
  traceId: string;
}
