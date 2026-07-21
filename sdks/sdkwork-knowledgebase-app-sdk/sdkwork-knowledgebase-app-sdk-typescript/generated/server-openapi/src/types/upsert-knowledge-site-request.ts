import type { KnowledgeSitePublishMode } from './knowledge-site-publish-mode';
import type { KnowledgeSiteVisibility } from './knowledge-site-visibility';

export interface UpsertKnowledgeSiteRequest {
  spaceId: string;
  title: string;
  visibility: KnowledgeSiteVisibility;
  homepageConceptId?: string | null;
  themeId: string;
  publishMode: KnowledgeSitePublishMode;
  expectedVersion?: string | null;
}
