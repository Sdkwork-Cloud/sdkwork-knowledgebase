import type { KnowledgeSite } from './knowledge-site';

export interface SitesUpdateResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
