import type { IngestionJob } from './ingestion-job';

export interface UploadSessionsCompleteResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
