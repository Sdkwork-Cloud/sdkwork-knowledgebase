import type { KnowledgeSiteRelease } from './knowledge-site-release';
import type { PageInfo } from './page-info';

export interface SiteReleasesListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
