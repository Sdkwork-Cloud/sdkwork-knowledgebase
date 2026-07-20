import type {
  CreateKnowledgeEngineProviderBindingRequest,
  CreateKnowledgeEngineProviderCredentialReferenceRequest,
  CreateKnowledgeEngineProviderMigrationOperationRequest,
  KnowledgeEngineProviderBinding,
  KnowledgeEngineProviderBindingPage,
  KnowledgeEngineProviderCredentialReference,
  KnowledgeEngineProviderCredentialReferencePage,
  KnowledgeEngineProviderMigrationOperation,
  KnowledgeEngineProviderMigrationOperationPage,
  PageInfo,
  RotateKnowledgeEngineProviderCredentialReferenceRequest,
  SdkworkKnowledgebaseBackendClient,
  UpdateKnowledgeEngineProviderBindingRequest,
} from 'sdkwork-knowledgebase-pc-admin-core';
import {
  extractSdkWorkListItems,
  getKnowledgebaseBackendSdkClient,
  readStringField,
} from 'sdkwork-knowledgebase-pc-admin-core';

const PAGE_SIZE = 20;

export interface ProviderAdminSpace {
  id: string;
  name: string;
  knowledgeMode: string;
}

export interface ProviderAdminCursorPage<T> {
  items: T[];
  nextCursor?: string;
  hasMore: boolean;
}

export interface ProviderBindingActions {
  canUpdate: boolean;
  canTest: boolean;
  canActivate: boolean;
  canDisable: boolean;
}

function normalizePage<T>(page: { items: T[]; pageInfo: PageInfo }): ProviderAdminCursorPage<T> {
  return {
    items: page.items,
    nextCursor: page.pageInfo.nextCursor ?? undefined,
    hasMore: page.pageInfo.hasMore === true || Boolean(page.pageInfo.nextCursor),
  };
}

export function providerBindingActions(binding: KnowledgeEngineProviderBinding): ProviderBindingActions {
  const capabilities = new Set(binding.capabilitySnapshot);
  return {
    canUpdate: binding.lifecycleState === 'draft',
    canTest: ['draft', 'failed', 'degraded'].includes(binding.lifecycleState),
    canActivate: binding.lifecycleState === 'testing'
      && capabilities.has('health')
      && capabilities.has('search'),
    canDisable: binding.lifecycleState !== 'disabled',
  };
}

export class ProviderAdminService {
  constructor(private readonly resolveClient: () => SdkworkKnowledgebaseBackendClient = () => (
    getKnowledgebaseBackendSdkClient().client
  )) {}

  async listSpaces(cursor?: string): Promise<ProviderAdminCursorPage<ProviderAdminSpace>> {
    const payload = await this.resolveClient().knowledge.spaces.list({ cursor, pageSize: PAGE_SIZE });
    const pageInfo = (payload.pageInfo ?? {}) as PageInfo;
    return {
      items: extractSdkWorkListItems(payload).map((item) => ({
        id: readStringField(item, 'id'),
        name: readStringField(item, 'name'),
        knowledgeMode: readStringField(item, 'knowledgeMode'),
      })),
      nextCursor: pageInfo.nextCursor ?? undefined,
      hasMore: pageInfo.hasMore === true || Boolean(pageInfo.nextCursor),
    };
  }

  async listCredentials(cursor?: string): Promise<ProviderAdminCursorPage<KnowledgeEngineProviderCredentialReference>> {
    const page: KnowledgeEngineProviderCredentialReferencePage = await this.resolveClient()
      .knowledge.providerCredentialReferences.list({ cursor, pageSize: PAGE_SIZE });
    return normalizePage(page);
  }

  async createCredential(
    request: CreateKnowledgeEngineProviderCredentialReferenceRequest,
  ): Promise<KnowledgeEngineProviderCredentialReference> {
    return this.resolveClient().knowledge.providerCredentialReferences.create(request);
  }

  async rotateCredential(
    id: string,
    request: RotateKnowledgeEngineProviderCredentialReferenceRequest,
  ): Promise<void> {
    await this.resolveClient().knowledge.providerCredentialReferences.rotate(id, request);
  }

  async revokeCredential(id: string, expectedVersion: string): Promise<void> {
    await this.resolveClient().knowledge.providerCredentialReferences.revoke(id, { expectedVersion });
  }

  async listBindings(spaceId: string, cursor?: string): Promise<ProviderAdminCursorPage<KnowledgeEngineProviderBinding>> {
    const page: KnowledgeEngineProviderBindingPage = await this.resolveClient()
      .knowledge.spaces.providerBindings.list(spaceId, { cursor, pageSize: PAGE_SIZE });
    return normalizePage(page);
  }

  async createBinding(
    spaceId: string,
    request: CreateKnowledgeEngineProviderBindingRequest,
  ): Promise<KnowledgeEngineProviderBinding> {
    return this.resolveClient().knowledge.spaces.providerBindings.create(spaceId, request);
  }

  async updateBinding(
    spaceId: string,
    bindingId: string,
    request: UpdateKnowledgeEngineProviderBindingRequest,
  ): Promise<KnowledgeEngineProviderBinding> {
    return this.resolveClient().knowledge.spaces.providerBindings.update(spaceId, bindingId, request);
  }

  async testBinding(spaceId: string, binding: KnowledgeEngineProviderBinding): Promise<void> {
    await this.resolveClient().knowledge.spaces.providerBindings.test(
      spaceId,
      binding.id,
      { expectedVersion: binding.version },
    );
  }

  async activateBinding(spaceId: string, binding: KnowledgeEngineProviderBinding): Promise<void> {
    await this.resolveClient().knowledge.spaces.providerBindings.activate(
      spaceId,
      binding.id,
      { expectedVersion: binding.version },
    );
  }

  async disableBinding(spaceId: string, binding: KnowledgeEngineProviderBinding): Promise<void> {
    await this.resolveClient().knowledge.spaces.providerBindings.disable(
      spaceId,
      binding.id,
      { expectedVersion: binding.version },
    );
  }

  async listMigrations(
    spaceId: string,
    cursor?: string,
  ): Promise<ProviderAdminCursorPage<KnowledgeEngineProviderMigrationOperation>> {
    const page: KnowledgeEngineProviderMigrationOperationPage = await this.resolveClient()
      .knowledge.spaces.providerMigrations.list(spaceId, { cursor, pageSize: PAGE_SIZE });
    return normalizePage(page);
  }

  async createMigration(
    spaceId: string,
    request: CreateKnowledgeEngineProviderMigrationOperationRequest,
  ): Promise<KnowledgeEngineProviderMigrationOperation> {
    return this.resolveClient().knowledge.spaces.providerMigrations.create(spaceId, request);
  }

  async rollbackMigration(
    spaceId: string,
    migration: KnowledgeEngineProviderMigrationOperation,
  ): Promise<void> {
    await this.resolveClient().knowledge.spaces.providerMigrations.rollback(
      spaceId,
      migration.id,
      { expectedVersion: migration.version },
    );
  }
}

export const providerAdminService = new ProviderAdminService();
