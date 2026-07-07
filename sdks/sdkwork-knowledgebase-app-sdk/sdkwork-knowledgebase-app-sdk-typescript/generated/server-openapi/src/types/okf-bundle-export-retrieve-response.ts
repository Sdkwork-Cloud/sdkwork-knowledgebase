import type { KnowledgeOkfBundleFile } from './knowledge-okf-bundle-file';

export interface OkfBundleExportRetrieveResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
