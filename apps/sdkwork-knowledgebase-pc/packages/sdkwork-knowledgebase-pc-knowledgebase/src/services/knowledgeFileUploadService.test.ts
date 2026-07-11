import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  configureKnowledgebaseAppSdk,
  configureKnowledgebaseDriveAppSdk,
  setKnowledgebaseApiEnabled,
} from 'sdkwork-knowledgebase-pc-core';

import { uploadKnowledgebaseFiles } from './knowledgeFileUploadService';

afterEach(() => {
  setKnowledgebaseApiEnabled(false);
});

describe('uploadKnowledgebaseFiles', () => {
  it('uploads root files under the knowledge browser files root', async () => {
    const browserList = vi.fn(async () => ({
      spaceId: '42',
      driveSpaceId: 'drv-kb-001',
      parentId: 'node-raw-root',
      view: 'files',
      pageSize: 1,
      items: [],
      pageInfo: {
        mode: 'cursor',
        hasMore: false,
        nextCursor: null,
      },
    }));
    const upload = vi.fn(async () => ({
      uploadItem: {
        nodeId: 'node-uploaded-file',
      },
    }));
    const driveImportCreate = vi.fn(async (body: { title: string }) => ({
      document: {
        id: 'doc-1',
        title: body.title,
      },
    }));

    configureKnowledgebaseAppSdk({
      client: {
        knowledge: {
          spaces: {
            retrieve: async () => ({
              id: '42',
              uuid: 'kb-42',
              name: 'Research',
              driveSpaceId: 'drv-kb-001',
              status: 'active',
              okfBundleInitialized: true,
              knowledgeMode: 'okf_bundle',
            }),
            browser: {
              list: browserList,
            },
          },
          driveImports: {
            create: driveImportCreate,
          },
        },
      } as never,
      setTokenManager() {
        // Test fake; token propagation is covered by pc-core SDK bootstrap tests.
      },
    });
    configureKnowledgebaseDriveAppSdk({
      client: {
        uploader: {
          upload,
        },
        drive: {
          nodes: {
            downloadUrls: {
              retrieve: async () => ({
                downloadUrl: null,
                signedSourceUrl: null,
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

    const file = {
      name: 'guide.pdf',
      type: 'application/pdf',
      size: 2048,
      lastModified: 1,
    } as File;

    await expect(uploadKnowledgebaseFiles([file], '42')).resolves.toHaveLength(1);

    expect(browserList).toHaveBeenCalledWith('42', {
      view: 'files',
      parentId: null,
      cursor: null,
      pageSize: 1,
    });
    expect(upload).toHaveBeenCalledWith(expect.objectContaining({
      spaceId: 'drv-kb-001',
      parentNodeId: 'node-raw-root',
      originalFileName: 'guide.pdf',
    }));
    expect(driveImportCreate).toHaveBeenCalledWith(expect.objectContaining({
      driveNodeId: 'node-uploaded-file',
      driveSpaceId: 'drv-kb-001',
      title: 'guide.pdf',
    }));
  });

  it('uses Drive uploader for root text files so original sources stay visible', async () => {
    const browserList = vi.fn(async () => ({
      spaceId: '42',
      driveSpaceId: 'drv-kb-001',
      parentId: 'node-raw-root',
      view: 'files',
      pageSize: 1,
      items: [],
      pageInfo: {
        mode: 'cursor',
        hasMore: false,
        nextCursor: null,
      },
    }));
    const upload = vi.fn(async () => ({
      uploadItem: {
        nodeId: 'node-uploaded-md',
      },
    }));
    const driveImportCreate = vi.fn(async (body: { title: string }) => ({
      document: {
        id: 'doc-md',
        title: body.title,
      },
    }));
    const ingestCreate = vi.fn(async () => ({
      id: 'ingest-md',
      state: 'succeeded',
    }));

    configureKnowledgebaseAppSdk({
      client: {
        knowledge: {
          spaces: {
            retrieve: async () => ({
              id: '42',
              uuid: 'kb-42',
              name: 'Research',
              driveSpaceId: 'drv-kb-001',
              status: 'active',
              okfBundleInitialized: true,
              knowledgeMode: 'okf_bundle',
            }),
            browser: {
              list: browserList,
            },
          },
          driveImports: {
            create: driveImportCreate,
          },
          ingests: {
            create: ingestCreate,
          },
        },
      } as never,
      setTokenManager() {
        // Test fake; token propagation is covered by pc-core SDK bootstrap tests.
      },
    });
    configureKnowledgebaseDriveAppSdk({
      client: {
        uploader: {
          upload,
        },
        drive: {
          nodes: {
            downloadUrls: {
              retrieve: async () => ({
                downloadUrl: null,
                signedSourceUrl: null,
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

    const file = {
      name: 'notes.md',
      type: 'text/markdown',
      size: 64,
      lastModified: 1,
      text: async () => '# Notes',
    } as File;

    await expect(uploadKnowledgebaseFiles([file], '42')).resolves.toHaveLength(1);

    expect(upload).toHaveBeenCalledWith(expect.objectContaining({
      parentNodeId: 'node-raw-root',
      uploadProfileCode: 'text',
      originalFileName: 'notes.md',
    }));
    expect(driveImportCreate).toHaveBeenCalledWith(expect.objectContaining({
      driveNodeId: 'node-uploaded-md',
      title: 'notes.md',
    }));
    expect(ingestCreate).not.toHaveBeenCalled();
  });
});
