import type { KnowledgeEngineProviderBinding } from './knowledge-engine-provider-binding';
import type { PageInfo } from './page-info';

/** One bounded cursor page of Provider bindings. */
export interface KnowledgeEngineProviderBindingPage {
  items: KnowledgeEngineProviderBinding[];
  pageInfo: PageInfo;
}
