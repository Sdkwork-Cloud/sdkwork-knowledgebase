import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  bindKnowledgebaseSessionStore,
  configureKnowledgebaseAppSdk,
  readLocalDocumentContent,
  setKnowledgebaseApiEnabled,
  type SessionSnapshot,
  type SessionStore,
} from 'sdkwork-knowledgebase-pc-core';

import { createDocument } from './knowledgebaseDocumentApiBridge';

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
});
