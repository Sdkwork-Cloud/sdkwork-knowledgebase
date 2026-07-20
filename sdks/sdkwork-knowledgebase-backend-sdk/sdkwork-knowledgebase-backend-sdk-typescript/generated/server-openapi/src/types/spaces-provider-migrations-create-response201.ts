import type { KnowledgeEngineProviderMigrationOperation } from './knowledge-engine-provider-migration-operation';

export interface SpacesProviderMigrationsCreateResponse201 {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
