import type { KnowledgeEngineProviderErrorCategory } from './knowledge-engine-provider-error-category';
import type { KnowledgeEngineProviderMigrationState } from './knowledge-engine-provider-migration-state';

export interface KnowledgeEngineProviderMigrationOperation {
  id: string;
  uuid: string;
  tenantId: string;
  organizationId: string;
  spaceId: string;
  sourceBindingId: string;
  targetBindingId: string;
  operationState: KnowledgeEngineProviderMigrationState;
  requestedBy: string;
  attemptCount: number;
  cutoverAt?: string | null;
  observationUntil?: string | null;
  completedAt?: string | null;
  lastErrorCategory?: KnowledgeEngineProviderErrorCategory | null;
  createdAt: string;
  updatedAt: string;
  version: string;
}
