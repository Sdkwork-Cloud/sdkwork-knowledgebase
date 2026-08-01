import type { KnowledgeOkfBundleFile } from './knowledge-okf-bundle-file';
import type { PageInfo } from './page-info';

export interface OkfBundleFilesListResponse {
  code: 0;
  data: unknown & { items: KnowledgeOkfBundleFile[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
