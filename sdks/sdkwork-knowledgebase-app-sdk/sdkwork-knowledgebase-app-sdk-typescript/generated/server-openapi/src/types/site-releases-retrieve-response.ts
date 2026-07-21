import type { KnowledgeSiteRelease } from './knowledge-site-release';

export interface SiteReleasesRetrieveResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
