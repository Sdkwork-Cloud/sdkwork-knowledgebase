import type { KnowledgeSite } from './knowledge-site';

export interface SitesRetrieveResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
