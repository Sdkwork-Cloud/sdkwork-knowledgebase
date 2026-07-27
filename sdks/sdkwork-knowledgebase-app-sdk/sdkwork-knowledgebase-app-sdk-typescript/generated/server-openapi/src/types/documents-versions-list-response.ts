import type { KnowledgeDocumentVersion } from './knowledge-document-version';
import type { PageInfo } from './page-info';

export interface DocumentsVersionsListResponse {
  code: 0;
  data: unknown & { items: KnowledgeDocumentVersion[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
