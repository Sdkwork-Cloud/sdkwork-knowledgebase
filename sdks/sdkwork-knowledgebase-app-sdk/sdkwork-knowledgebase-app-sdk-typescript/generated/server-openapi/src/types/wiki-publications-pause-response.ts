import type { KnowledgeWikiPublication } from './knowledge-wiki-publication';

export interface WikiPublicationsPauseResponse {
  code: 0;
  data: unknown & { item: KnowledgeWikiPublication; };
  /** Server-owned request correlation id. */
  traceId: string;
}
