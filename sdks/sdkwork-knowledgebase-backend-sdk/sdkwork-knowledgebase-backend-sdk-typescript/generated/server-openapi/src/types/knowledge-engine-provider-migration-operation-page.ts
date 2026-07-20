import type { KnowledgeEngineProviderMigrationOperation } from './knowledge-engine-provider-migration-operation';
import type { PageInfo } from './page-info';

/** One bounded cursor page of Provider migration operations. */
export interface KnowledgeEngineProviderMigrationOperationPage {
  items: KnowledgeEngineProviderMigrationOperation[];
  pageInfo: PageInfo;
}
