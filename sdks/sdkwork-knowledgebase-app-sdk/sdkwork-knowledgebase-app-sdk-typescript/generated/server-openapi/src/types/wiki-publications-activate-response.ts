import type { KnowledgeWikiPublication } from './knowledge-wiki-publication';

export interface WikiPublicationsActivateResponse {
  code: 0;
  data: unknown & { item: KnowledgeWikiPublication; };
  /** Server-owned request correlation id. */
  traceId: string;
}
