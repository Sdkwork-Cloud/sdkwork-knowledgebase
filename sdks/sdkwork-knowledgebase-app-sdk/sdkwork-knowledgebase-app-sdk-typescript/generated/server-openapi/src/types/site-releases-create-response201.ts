import type { KnowledgeSitePublicationResult } from './knowledge-site-publication-result';

export interface SiteReleasesCreateResponse201 {
  code: 0;
  data: unknown & KnowledgeSitePublicationResult;
  /** Server-owned request correlation id. */
  traceId: string;
}
