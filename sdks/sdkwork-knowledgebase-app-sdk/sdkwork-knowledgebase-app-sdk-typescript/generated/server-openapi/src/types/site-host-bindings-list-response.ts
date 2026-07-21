import type { KnowledgeSiteHostBinding } from './knowledge-site-host-binding';
import type { PageInfo } from './page-info';

export interface SiteHostBindingsListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
