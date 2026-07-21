import type { KnowledgeSiteHostBindingState } from './knowledge-site-host-binding-state';
import type { KnowledgeSiteHostBindingType } from './knowledge-site-host-binding-type';

export interface KnowledgeSiteHostBinding {
  id: string;
  uuid: string;
  siteId: string;
  bindingType: KnowledgeSiteHostBindingType;
  normalizedHost: string;
  canonical: boolean;
  lifecycleState: KnowledgeSiteHostBindingState;
  webServerSiteId?: string | null;
  webServerDomainId?: string | null;
  webServerDeploymentId?: string | null;
  createdAt: string;
  updatedAt: string;
  version: string;
}
