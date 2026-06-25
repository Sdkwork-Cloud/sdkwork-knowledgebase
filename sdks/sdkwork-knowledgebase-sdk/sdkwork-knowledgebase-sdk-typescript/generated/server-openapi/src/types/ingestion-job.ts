import type { IngestionJobState } from './ingestion-job-state';

export interface IngestionJob {
  id: number;
  spaceId: number;
  sourceType: string;
  idempotencyKey: string;
  state: IngestionJobState;
  errorMessage?: string | null;
}
