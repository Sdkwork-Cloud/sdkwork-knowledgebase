import type { KnowledgeOkfBundleFile } from './knowledge-okf-bundle-file';

export interface OkfProfileUpdateResponse {
  code: 0;
  data: unknown & { item: KnowledgeOkfBundleFile; };
  /** Server-owned request correlation id. */
  traceId: string;
}
