import type { KnowledgeDocument } from './knowledge-document';

export interface DocumentsRetrieveResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
