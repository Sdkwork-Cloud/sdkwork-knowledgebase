import type { KnowledgeDocumentVersion } from './knowledge-document-version';

export interface DocumentsVersionsResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
