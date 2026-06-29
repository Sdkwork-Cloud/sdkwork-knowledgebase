import type { KnowledgeOkfBundleFile } from './knowledge-okf-bundle-file';
import type { PageInfo } from './page-info';

export interface OkfBundleFilesListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
