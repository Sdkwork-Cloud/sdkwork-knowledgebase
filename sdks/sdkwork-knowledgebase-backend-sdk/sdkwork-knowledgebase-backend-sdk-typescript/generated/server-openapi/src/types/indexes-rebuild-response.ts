import type { OkfIndexDocument } from './okf-index-document';

export interface IndexesRebuildResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
