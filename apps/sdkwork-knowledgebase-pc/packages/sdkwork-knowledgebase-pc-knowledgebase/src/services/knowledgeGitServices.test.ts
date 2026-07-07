import { afterEach, describe, expect, it } from 'vitest';
import {
  bindKnowledgebaseSessionStore,
  configureKnowledgebaseAppSdk,
  setKnowledgebaseApiEnabled,
  type SessionSnapshot,
  type SessionStore,
} from 'sdkwork-knowledgebase-pc-core';

import {
  importGitRepository,
  type GitImportProgress,
} from './knowledgeGitImportService';
import {
  syncGitRepository,
  type GitSyncProgress,
} from './knowledgeGitSyncService';

const EMPTY_SESSION_STORE: SessionStore = createStaticSessionStore({});
const ELLIPSIS = '\u2026';
const MOJIBAKE_PATTERN = /[\uFFFD\u951F]/;

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
        // Test fake; listener lifecycle is not relevant for Git service orchestration.
      };
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

function installRegisteredSpace(tenantId: string, spaceId: string): void {
  const storage = new Map<string, string>();
  storage.set(
    `sdkwork.knowledgebase.spaces.v1.${tenantId}`,
    JSON.stringify([
      {
        spaceId,
        kbType: 'team',
        createdAt: '2026-07-08T00:00:00.000Z',
      },
    ]),
  );

  Object.defineProperty(globalThis, 'window', {
    configurable: true,
    value: {
      localStorage: {
        getItem: (key: string) => storage.get(key) ?? null,
        setItem: (key: string, value: string) => storage.set(key, value),
        removeItem: (key: string) => storage.delete(key),
      },
    },
  });
}

function configureFakeKnowledgeGitClient(): void {
  configureKnowledgebaseAppSdk({
    client: {
      knowledge: {
        gitImports: {
          create: async () => ({
            importedCount: 2,
            skippedCount: 1,
          }),
        },
        gitSyncs: {
          create: async () => ({
            accepted: true,
            status: 'completed',
            hash: 'abc123',
            syncedCount: 3,
          }),
        },
      },
    } as never,
    setTokenManager() {
      // Test fake; token propagation is covered by pc-core SDK bootstrap tests.
    },
  });
  setKnowledgebaseApiEnabled(true);
}

function prepareRegisteredGitService(): void {
  bindTenantContext('tenant-1');
  installRegisteredSpace('tenant-1', '42');
  configureFakeKnowledgeGitClient();
}

afterEach(() => {
  bindKnowledgebaseSessionStore(EMPTY_SESSION_STORE);
  setKnowledgebaseApiEnabled(false);
  Reflect.deleteProperty(globalThis, 'window');
});

describe('knowledge Git services', () => {
  it('reports readable import progress messages without mojibake', async () => {
    const progress: GitImportProgress[] = [];
    prepareRegisteredGitService();

    await expect(
      importGitRepository(
        '42',
        'https://git.example.test/repo.git',
        'main',
        undefined,
        (entry) => progress.push(entry),
      ),
    ).resolves.toEqual({
      importedCount: 2,
      skippedCount: 1,
    });

    expect(progress.map((entry) => entry.message)).toEqual([
      `Resolving repository on branch "main"${ELLIPSIS}`,
      `Importing repository files on the server${ELLIPSIS}`,
      'Imported 2 file(s).',
    ]);
    expect(progress.map((entry) => entry.message).join('\n')).not.toMatch(MOJIBAKE_PATTERN);
  });

  it('reports readable sync progress messages without mojibake', async () => {
    const progress: GitSyncProgress[] = [];
    prepareRegisteredGitService();

    await expect(
      syncGitRepository(
        '42',
        'https://git.example.test/repo.git',
        'release',
        'Publish knowledgebase changes',
        undefined,
        (entry) => progress.push(entry),
      ),
    ).resolves.toEqual({
      accepted: true,
      status: 'completed',
      hash: 'abc123',
      syncedCount: 3,
    });

    expect(progress.map((entry) => entry.message)).toEqual([
      `Resolving repository on branch "release"${ELLIPSIS}`,
      `Pushing knowledge base documents to the remote repository${ELLIPSIS}`,
      'Synced 3 file(s) to Git.',
    ]);
    expect(progress.map((entry) => entry.message).join('\n')).not.toMatch(MOJIBAKE_PATTERN);
  });
});
