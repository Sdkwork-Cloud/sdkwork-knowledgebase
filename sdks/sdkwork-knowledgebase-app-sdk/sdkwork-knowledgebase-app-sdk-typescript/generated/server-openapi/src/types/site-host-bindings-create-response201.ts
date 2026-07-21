import type { KnowledgeSiteHostBinding } from './knowledge-site-host-binding';

export interface SiteHostBindingsCreateResponse201 {
  code: 0;
  data: unknown & KnowledgeSiteHostBinding;
  /** Server-owned request correlation id. */
  traceId: string;
}
