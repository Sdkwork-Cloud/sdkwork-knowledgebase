import type { IngestionJob } from './ingestion-job';

export interface IngestsRetrieveResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
