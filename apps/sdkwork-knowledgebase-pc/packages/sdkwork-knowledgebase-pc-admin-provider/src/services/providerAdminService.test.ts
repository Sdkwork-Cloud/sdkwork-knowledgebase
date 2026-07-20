import { describe, expect, it, vi } from 'vitest';
import type {
  KnowledgeEngineProviderBinding,
  SdkworkKnowledgebaseBackendClient,
} from 'sdkwork-knowledgebase-pc-admin-core';
import { ProviderAdminService, providerBindingActions } from './providerAdminService';

function binding(
  lifecycleState: KnowledgeEngineProviderBinding['lifecycleState'],
  capabilities: KnowledgeEngineProviderBinding['capabilitySnapshot'] = [],
): KnowledgeEngineProviderBinding {
  return {
    id: '1', uuid: 'binding-1', tenantId: '1', organizationId: '1', spaceId: '1',
    implementationId: 'engine.knowledge.external.dify', remoteResourceType: 'dataset',
    remoteResourceId: 'remote-1', lifecycleState, capabilitySnapshot: capabilities,
    capabilitySnapshotVersion: '1', createdBy: 'operator', updatedBy: 'operator',
    createdAt: '2026-07-20T00:00:00Z', updatedAt: '2026-07-20T00:00:00Z', version: '2',
  };
}

describe('ProviderAdminService', () => {
  it('enables Binding actions only for valid lifecycle and capability states', () => {
    expect(providerBindingActions(binding('draft'))).toEqual({
      canUpdate: true, canTest: true, canActivate: false, canDisable: true,
    });
    expect(providerBindingActions(binding('testing', ['health', 'search']))).toEqual({
      canUpdate: false, canTest: false, canActivate: true, canDisable: true,
    });
    expect(providerBindingActions(binding('testing', ['health']))).toEqual({
      canUpdate: false, canTest: false, canActivate: false, canDisable: true,
    });
    expect(providerBindingActions(binding('disabled'))).toEqual({
      canUpdate: false, canTest: false, canActivate: false, canDisable: false,
    });
  });

  it('uses bounded SDK cursor pages and version-fenced commands', async () => {
    const list = vi.fn().mockResolvedValue({
      items: [binding('draft')],
      pageInfo: { mode: 'cursor', nextCursor: 'next', hasMore: true },
    });
    const testBinding = vi.fn().mockResolvedValue({ accepted: true });
    const client = {
      knowledge: {
        spaces: {
          providerBindings: { list, test: testBinding },
        },
      },
    } as unknown as SdkworkKnowledgebaseBackendClient;
    const service = new ProviderAdminService(() => client);

    const page = await service.listBindings('space-1', 'cursor-1');
    expect(page.nextCursor).toBe('next');
    expect(list).toHaveBeenCalledWith('space-1', { cursor: 'cursor-1', pageSize: 20 });
    await service.testBinding('space-1', page.items[0]);
    expect(testBinding).toHaveBeenCalledWith('space-1', '1', { expectedVersion: '2' });
  });
});
