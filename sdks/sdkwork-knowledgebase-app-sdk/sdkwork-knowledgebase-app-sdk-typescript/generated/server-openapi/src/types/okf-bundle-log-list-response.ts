import type { OkfLogDocument } from './okf-log-document';

export interface OkfBundleLogListResponse {
  code: 0;
  data: unknown & { item: OkfLogDocument; };
  /** Server-owned request correlation id. */
  traceId: string;
}
