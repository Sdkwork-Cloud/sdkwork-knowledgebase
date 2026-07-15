import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  bindKnowledgebaseSessionStore,
  configureKnowledgebaseAppSdk,
  readLocalDocumentContent,
  setKnowledgebaseApiEnabled,
  type SessionSnapshot,
  type SessionStore,
} from 'sdkwork-knowledgebase-pc-core';

import {
  createDocument,
  getDocumentContent,
  saveDocumentContent,
} from './knowledgebaseDocumentApiBridge';
import { activateEphemeralFixedKnowledgebaseWorkspace } from '../workspaceMode';

const EMPTY_SESSION_STORE: SessionStore = createStaticSessionStore({});

function createStaticSessionStore(snapshot: SessionSnapshot): SessionStore {
  return {
    getSnapshot: () => snapshot,
    refreshSession: () => snapshot,
    setSession() {
      // Test fake; session mutation is covered by pc-core session tests.
    },
    clearSession() {
      // Test fake; session clearing is covered by pc-core session tests.
    },
    subscribe() {
      return () => {
        // Test fake; listener lifecycle is not relevant for document bridge orchestration.
      };
    },
  };
}

function installWindowStorage(tenantId: string, spaceId: string): void {
  const localStorage = new Map<string, string>();
  const sessionStorage = new Map<string, string>();
  localStorage.set(
    `sdkwork.knowledgebase.spaces.v1.${tenantId}`,
    JSON.stringify([
      {
        spaceId,
        kbType: 'team',
        createdAt: '2026-07-09T00:00:00.000Z',
      },
    ]),
  );

  Object.defineProperty(globalThis, 'window', {
    configurable: true,
    value: {
      localStorage: storageFromMap(localStorage),
      sessionStorage: storageFromMap(sessionStorage),
    },
  });
}

function storageFromMap(values: Map<string, string>): Storage {
  return {
    get length() {
      return values.size;
    },
    clear() {
      values.clear();
    },
    getItem(key: string) {
      return values.get(key) ?? null;
    },
    key(index: number) {
      return Array.from(values.keys())[index] ?? null;
    },
    removeItem(key: string) {
      values.delete(key);
    },
    setItem(key: string, value: string) {
      values.set(key, value);
    },
  };
}

function bindTenantContext(tenantId: string): void {
  bindKnowledgebaseSessionStore(
    createStaticSessionStore({
      context: {
        tenantId,
        userId: 'user-1',
      },
    }),
  );
}

afterEach(() => {
  bindKnowledgebaseSessionStore(EMPTY_SESSION_STORE);
  setKnowledgebaseApiEnabled(false);
  Reflect.deleteProperty(globalThis, 'window');
});

describe('KnowledgebaseDocumentApiBridge.createDocument', () => {
  it('uses ingest-created document identity for new documents with content', async () => {
    const documentsCreate = vi.fn();
    const ingestsCreate = vi.fn(async () => ({
      id: 'ingest-1',
      state: 'succeeded',
    }));
    const browserList = vi.fn(async () => ({
      spaceId: '42',
      parentId: null,
      view: 'files',
      pageSize: 100,
      items: [
        {
          id: 'browser-doc-991',
          nodeType: 'document',
          name: 'Research Note',
          path: '/Research Note.md',
          documentId: '991',
          mimeType: 'text/markdown',
          updatedAt: '2026-07-09T00:00:00.000Z',
          permissions: {
            canRead: true,
            canWrite: true,
            canDelete: true,
          },
        },
      ],
      pageInfo: {
        mode: 'cursor',
        hasMore: false,
        nextCursor: null,
      },
    }));

    bindTenantContext('tenant-1');
    installWindowStorage('tenant-1', '42');
    configureKnowledgebaseAppSdk({
      client: {
        knowledge: {
          documents: {
            create: documentsCreate,
          },
          ingests: {
            create: ingestsCreate,
          },
          spaces: {
            browser: {
              list: browserList,
            },
          },
        },
      } as never,
      setTokenManager() {
        // Test fake; token propagation is covered by pc-core SDK bootstrap tests.
      },
    });
    setKnowledgebaseApiEnabled(true);

    const created = await createDocument({
      kbId: '42',
      title: 'Research Note',
      type: 'markdown',
      content: '# Research Note',
    });

    expect(documentsCreate).not.toHaveBeenCalled();
    expect(ingestsCreate).toHaveBeenCalledWith({
      spaceId: '42',
      title: 'Research Note',
      payloadMarkdown: '# Research Note',
      idempotencyKey: expect.stringMatching(/^pc-create-42-/),
    });
    expect(browserList).toHaveBeenCalledWith('42', {
      view: 'files',
      parentId: null,
      cursor: null,
      pageSize: 100,
    });
    expect(created.id).toBe('991');
    expect(created.title).toBe('Research Note');
    expect(created.kbId).toBe('42');
    expect(readLocalDocumentContent('tenant-1', '991')).toBe('# Research Note');
  });

  it('does not access browser storage for an active fixed group workspace', async () => {
    const localStorage = createStorageSpy();
    const sessionStorage = createStorageSpy();
    const directContentLookup = vi.fn(async () => ({
      documentId: '991',
      contentMarkdown: '# Group-only document',
      contentVersion: 'group-v1',
    }));
    const directDocumentLookup = vi.fn();
    Object.defineProperty(globalThis, 'window', {
      configurable: true,
      value: { localStorage, sessionStorage },
    });
    bindTenantContext('tenant-1');
    configureKnowledgebaseAppSdk({
      client: {
        knowledge: {
          documents: {
            retrieve: directDocumentLookup,
            content: {
              list: directContentLookup,
            },
          },
          ingests: {
            create: async () => ({ id: 'ingest-group-1', state: 'succeeded' }),
          },
          retrievals: {
            create: async () => ({
              hits: [
                {
                  documentId: '991',
                  title: 'Group source',
                  content: '# Group-only document',
                },
              ],
            }),
          },
          spaces: {
            browser: {
              list: async () => ({
                spaceId: '420',
                parentId: null,
                view: 'files',
                pageSize: 100,
                items: [
                  {
                    id: 'browser-doc-991',
                    nodeType: 'document',
                    name: 'Group source',
                    path: '/Group source.md',
                    documentId: '991',
                    mimeType: 'text/markdown',
                    updatedAt: '2026-07-13T00:00:00.000Z',
                    permissions: {
                      canRead: true,
                      canWrite: true,
                      canDelete: true,
                    },
                  },
                  {
                    id: 'browser-doc-992',
                    nodeType: 'document',
                    name: 'Group draft',
                    path: '/Group draft.md',
                    documentId: '992',
                    mimeType: 'text/markdown',
                    updatedAt: '2026-07-13T00:00:00.000Z',
                    permissions: {
                      canRead: true,
                      canWrite: true,
                      canDelete: true,
                    },
                  },
                ],
                pageInfo: {
                  mode: 'cursor',
                  hasMore: false,
                  nextCursor: null,
                },
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

    const releaseWorkspace = activateEphemeralFixedKnowledgebaseWorkspace('420');
    try {
      await expect(getDocumentContent('991')).resolves.toBe('# Group-only document');
      await expect(saveDocumentContent('991', '# Updated group document')).resolves.toBe(true);
      await expect(createDocument({
        kbId: '420',
        title: 'Group draft',
        type: 'markdown',
        content: '# New group draft',
      })).resolves.toMatchObject({ id: '992', kbId: '420' });
    } finally {
      releaseWorkspace();
    }

    expect(directContentLookup).not.toHaveBeenCalled();
    expect(directDocumentLookup).not.toHaveBeenCalled();
    expectStorageUnused(localStorage);
    expectStorageUnused(sessionStorage);
  });
});

function createStorageSpy(): Storage {
  return {
    get length() {
      return 0;
    },
    clear: vi.fn(),
    getItem: vi.fn(() => null),
    key: vi.fn(() => null),
    removeItem: vi.fn(),
    setItem: vi.fn(),
  };
}

function expectStorageUnused(storage: Storage): void {
  expect(storage.getItem).not.toHaveBeenCalled();
  expect(storage.setItem).not.toHaveBeenCalled();
  expect(storage.removeItem).not.toHaveBeenCalled();
}
