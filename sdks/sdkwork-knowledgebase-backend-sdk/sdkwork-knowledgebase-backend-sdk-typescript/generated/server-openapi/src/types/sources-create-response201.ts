import type { KnowledgeSource } from './knowledge-source';

export interface SourcesCreateResponse201 {
  code: 0;
  data: unknown & { item: KnowledgeSource; };
  /** Server-owned request correlation id. */
  traceId: string;
}
