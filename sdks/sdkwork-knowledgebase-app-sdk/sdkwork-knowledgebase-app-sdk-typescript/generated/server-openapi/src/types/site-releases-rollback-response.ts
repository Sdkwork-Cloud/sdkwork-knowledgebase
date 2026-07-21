import type { KnowledgeSite } from './knowledge-site';

export interface SiteReleasesRollbackResponse {
  code: 0;
  data: unknown & KnowledgeSite;
  /** Server-owned request correlation id. */
  traceId: string;
}
