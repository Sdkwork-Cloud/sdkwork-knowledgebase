import type { KnowledgeEngineCapability } from './knowledge-engine-capability';
import type { KnowledgeEngineProviderBindingState } from './knowledge-engine-provider-binding-state';
import type { KnowledgeEngineProviderErrorCategory } from './knowledge-engine-provider-error-category';

export interface KnowledgeEngineProviderBinding {
  id: string;
  uuid: string;
  tenantId: string;
  organizationId: string;
  spaceId: string;
  implementationId: string;
  remoteResourceType: string;
  remoteResourceId: string;
  credentialReferenceId?: string | null;
  lifecycleState: KnowledgeEngineProviderBindingState;
  capabilitySnapshot: KnowledgeEngineCapability[];
  capabilitySnapshotVersion: string;
  lastTestedAt?: string | null;
  activatedAt?: string | null;
  disabledAt?: string | null;
  lastErrorCategory?: KnowledgeEngineProviderErrorCategory | null;
  createdBy: string;
  updatedBy: string;
  createdAt: string;
  updatedAt: string;
  version: string;
}
