import type { KnowledgeSource } from './knowledge-source';
import type { PageInfo } from './page-info';

export interface SourcesListResponse {
  code: 0;
  data: unknown & { items: KnowledgeSource[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
