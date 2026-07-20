import type { KnowledgeEngineProviderMigrationOperationPage } from './knowledge-engine-provider-migration-operation-page';

export interface SpacesProviderMigrationsListResponse {
  code: 0;
  data: unknown & KnowledgeEngineProviderMigrationOperationPage;
  /** Server-owned request correlation id. */
  traceId: string;
}
