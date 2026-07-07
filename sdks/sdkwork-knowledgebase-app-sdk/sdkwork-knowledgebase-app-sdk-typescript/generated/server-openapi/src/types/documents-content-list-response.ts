import type { KnowledgeDocumentContent } from './knowledge-document-content';

export interface DocumentsContentListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
