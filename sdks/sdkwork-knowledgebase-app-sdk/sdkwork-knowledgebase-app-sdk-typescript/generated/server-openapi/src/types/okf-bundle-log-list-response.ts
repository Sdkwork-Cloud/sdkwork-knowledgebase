import type { OkfLogDocument } from './okf-log-document';

export interface OkfBundleLogListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
