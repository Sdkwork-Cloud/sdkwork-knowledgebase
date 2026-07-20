import type { KnowledgeEngineProviderCredentialRotationState } from './knowledge-engine-provider-credential-rotation-state';

export interface KnowledgeEngineProviderCredentialReference {
  id: string;
  uuid: string;
  tenantId: string;
  organizationId: string;
  implementationId: string;
  displayName: string;
  rotationState: KnowledgeEngineProviderCredentialRotationState;
  lastRotatedAt?: string | null;
  createdBy: string;
  updatedBy: string;
  createdAt: string;
  updatedAt: string;
  version: string;
}
