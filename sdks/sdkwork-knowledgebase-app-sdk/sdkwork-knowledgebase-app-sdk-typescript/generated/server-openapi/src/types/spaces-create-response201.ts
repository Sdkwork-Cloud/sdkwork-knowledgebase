import type { KnowledgeSpace } from './knowledge-space';

export interface SpacesCreateResponse201 {
  code: 0;
  data: unknown & { item: KnowledgeSpace; };
  /** Server-owned request correlation id. */
  traceId: string;
}
