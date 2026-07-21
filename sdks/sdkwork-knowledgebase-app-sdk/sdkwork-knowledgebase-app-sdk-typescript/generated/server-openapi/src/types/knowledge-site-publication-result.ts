import type { KnowledgeSite } from './knowledge-site';
import type { KnowledgeSiteRelease } from './knowledge-site-release';

export interface KnowledgeSitePublicationResult {
  site: KnowledgeSite;
  release: KnowledgeSiteRelease;
  publicUrl: string;
}
