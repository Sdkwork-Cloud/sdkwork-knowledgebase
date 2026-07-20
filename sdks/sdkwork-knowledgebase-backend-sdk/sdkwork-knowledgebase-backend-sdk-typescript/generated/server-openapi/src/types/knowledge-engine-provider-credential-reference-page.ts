import type { KnowledgeEngineProviderCredentialReference } from './knowledge-engine-provider-credential-reference';
import type { PageInfo } from './page-info';

/** One bounded cursor page of Provider credential references. */
export interface KnowledgeEngineProviderCredentialReferencePage {
  items: KnowledgeEngineProviderCredentialReference[];
  pageInfo: PageInfo;
}
