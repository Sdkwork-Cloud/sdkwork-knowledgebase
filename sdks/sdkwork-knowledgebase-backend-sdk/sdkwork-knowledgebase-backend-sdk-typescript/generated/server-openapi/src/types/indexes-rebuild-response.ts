import type { OkfIndexDocument } from './okf-index-document';

export interface IndexesRebuildResponse {
  code: 0;
  data: unknown & { item: OkfIndexDocument; };
  /** Server-owned request correlation id. */
  traceId: string;
}
