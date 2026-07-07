import type { OkfProfileDocument } from './okf-profile-document';

export interface OkfBundleProfileListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
