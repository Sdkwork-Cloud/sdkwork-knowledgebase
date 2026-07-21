import type { KnowledgeSitePublishMode } from './knowledge-site-publish-mode';
import type { KnowledgeSiteState } from './knowledge-site-state';
import type { KnowledgeSiteVisibility } from './knowledge-site-visibility';

export interface KnowledgeSite {
  id: string;
  uuid: string;
  tenantId: string;
  organizationId: string;
  spaceId: string;
  title: string;
  visibility: KnowledgeSiteVisibility;
  homepageConceptId?: string | null;
  themeId: string;
  publishMode: KnowledgeSitePublishMode;
  lifecycleState: KnowledgeSiteState;
  canonicalHostBindingId?: string | null;
  currentReleaseId?: string | null;
  createdAt: string;
  updatedAt: string;
  version: string;
}
