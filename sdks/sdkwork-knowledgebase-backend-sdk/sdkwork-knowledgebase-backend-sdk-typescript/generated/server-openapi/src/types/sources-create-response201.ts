import type { KnowledgeSource } from './knowledge-source';

export interface SourcesCreateResponse201 {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
