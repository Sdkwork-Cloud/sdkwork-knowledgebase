import type { OkfQualityRun } from './okf-quality-run';

export interface OkfLintRunsCreateResponse201 {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
