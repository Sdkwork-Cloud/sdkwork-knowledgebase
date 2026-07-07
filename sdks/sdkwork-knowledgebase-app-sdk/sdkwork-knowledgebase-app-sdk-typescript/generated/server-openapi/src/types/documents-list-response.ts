import type { KnowledgeDocument } from './knowledge-document';
import type { PageInfo } from './page-info';

export interface DocumentsListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
