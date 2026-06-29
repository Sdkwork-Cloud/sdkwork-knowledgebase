import type { KnowledgeIndex } from './knowledge-index';

export interface IndexesCreateResponse201 {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
