import type { OkfBundleImportResult } from './okf-bundle-import-result';

export interface OkfBundleImportCreateResponse201 {
  code: 0;
  data: unknown & { item: OkfBundleImportResult; };
  /** Server-owned request correlation id. */
  traceId: string;
}
