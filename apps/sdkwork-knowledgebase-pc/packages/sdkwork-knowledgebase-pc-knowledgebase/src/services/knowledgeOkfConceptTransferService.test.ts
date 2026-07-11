import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  configureKnowledgebaseAppSdk,
  setKnowledgebaseApiEnabled,
} from 'sdkwork-knowledgebase-pc-core';

import { copyOkfConcept } from './knowledgeOkfConceptTransferService';

afterEach(() => {
  setKnowledgebaseApiEnabled(false);
});

describe('knowledge OKF concept transfer service', () => {
  it('resolves copied concepts through the OKF bundle browser view', async () => {
    let upsertedLogicalPath = '';
    const browserList = vi.fn(async () => ({
      spaceId: '43',
      driveSpaceId: 'drv-kb-043',
      parentId: 'node-okf-root',
      view: 'okf_bundle',
      pageSize: 100,
      items: [
        {
          id: 'node-copied-concept',
          nodeType: 'okf_concept',
          name: 'customers-copy.md',
          path: upsertedLogicalPath,
          conceptId: '9001',
          updatedAt: '2026-07-09T00:00:00.000Z',
          permissions: {
            canRead: true,
            canUpload: false,
            canRename: false,
            canMove: false,
            canDelete: false,
            canReview: false,
            canPublish: false,
          },
        },
      ],
      pageInfo: {
        mode: 'cursor',
        hasMore: false,
        nextCursor: null,
      },
    }));

    configureKnowledgebaseAppSdk({
      client: {
        knowledge: {
          spaces: {
            browser: {
              list: browserList,
            },
          },
          okf: {
            concepts: {
              retrieve: vi.fn(async () => ({
                conceptId: 'customers',
                title: 'Customers',
                tags: ['crm'],
              })),
              update: vi.fn(async (body: { conceptId: string }) => {
                upsertedLogicalPath = `okf/${body.conceptId}.md`;
                return {
                  conceptId: body.conceptId,
                  logicalPath: upsertedLogicalPath,
                };
              }),
            },
          },
        },
      } as never,
      setTokenManager() {
        // Test fake; token propagation is covered by pc-core SDK bootstrap tests.
      },
    });
    setKnowledgebaseApiEnabled(true);

    const copied = await copyOkfConcept(
      '42',
      '91',
      '43',
      async () => '---\ntitle: Customers\n---\n\n# Customers',
    );

    expect(copied.id).toBe('okf:43:9001');
    expect(browserList).toHaveBeenCalledWith('43', {
      view: 'okf_bundle',
      parentId: null,
      cursor: null,
      pageSize: 100,
    });
  });
});
