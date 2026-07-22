import type { OkfIndexDocument } from './okf-index-document';

export interface OkfBundleIndexRebuildResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
