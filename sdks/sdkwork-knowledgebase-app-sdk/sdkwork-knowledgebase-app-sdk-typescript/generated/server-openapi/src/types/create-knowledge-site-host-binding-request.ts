import type { KnowledgeSiteHostBindingType } from './knowledge-site-host-binding-type';

export interface CreateKnowledgeSiteHostBindingRequest {
  bindingType: KnowledgeSiteHostBindingType;
  host: string;
  canonical: boolean;
  expectedSiteVersion: string;
}
