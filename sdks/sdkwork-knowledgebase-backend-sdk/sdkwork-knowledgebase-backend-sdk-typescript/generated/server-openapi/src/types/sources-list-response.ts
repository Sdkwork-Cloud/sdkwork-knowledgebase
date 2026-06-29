import type { KnowledgeSource } from './knowledge-source';
import type { PageInfo } from './page-info';

export interface SourcesListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
