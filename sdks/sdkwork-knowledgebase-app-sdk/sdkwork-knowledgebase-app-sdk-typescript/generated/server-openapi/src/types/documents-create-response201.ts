import type { KnowledgeDocument } from './knowledge-document';

export interface DocumentsCreateResponse201 {
  code: 0;
  data: unknown & { item: KnowledgeDocument; };
  /** Server-owned request correlation id. */
  traceId: string;
}
