import type { KnowledgeWikiPublication } from './knowledge-wiki-publication';

export interface WikiPublicationsRetrieveResponse {
  code: 0;
  data: unknown & { item: KnowledgeWikiPublication; };
  /** Server-owned request correlation id. */
  traceId: string;
}
