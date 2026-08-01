import type { IngestionJob } from './ingestion-job';

export interface OkfCompileJobsCreateResponse201 {
  code: 0;
  data: unknown & { item: IngestionJob; };
  /** Server-owned request correlation id. */
  traceId: string;
}
