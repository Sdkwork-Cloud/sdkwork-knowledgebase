import type { KnowledgeDocument } from './knowledge-document';

export interface DocumentsCreateResponse201 {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
