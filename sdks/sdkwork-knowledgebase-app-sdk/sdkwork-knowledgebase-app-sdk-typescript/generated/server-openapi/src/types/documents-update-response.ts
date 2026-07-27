import type { KnowledgeDocument } from './knowledge-document';

export interface DocumentsUpdateResponse {
  code: 0;
  data: unknown & { item: KnowledgeDocument; };
  /** Server-owned request correlation id. */
  traceId: string;
}
