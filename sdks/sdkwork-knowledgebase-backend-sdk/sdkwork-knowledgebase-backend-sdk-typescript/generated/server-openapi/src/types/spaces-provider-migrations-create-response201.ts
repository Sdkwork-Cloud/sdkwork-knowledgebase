import type { KnowledgeEngineProviderMigrationOperation } from './knowledge-engine-provider-migration-operation';

export interface SpacesProviderMigrationsCreateResponse201 {
  code: 0;
  data: unknown & { item: KnowledgeEngineProviderMigrationOperation; };
  /** Server-owned request correlation id. */
  traceId: string;
}
