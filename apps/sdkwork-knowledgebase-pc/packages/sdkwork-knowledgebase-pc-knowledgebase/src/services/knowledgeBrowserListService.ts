import type { KnowledgeBrowserNode } from 'sdkwork-knowledgebase-pc-core';
import { requireKnowledgebaseAppSdkHttpClient } from 'sdkwork-knowledgebase-pc-core';
import { normalizeSdkWorkListPage } from './sdkWorkListPage';

const BROWSER_NODE_CACHE_TTL_MS = 30_000;
const BROWSER_CACHE_MAX_ENTRIES = 256;
const DEFAULT_BROWSER_PAGE_SIZE = 100;
const browserParentCache = new Map<string, BrowserNodeCacheEntry>();

interface BrowserNodeCacheEntry {
  fetchedAt: number;
  nodes: KnowledgeBrowserNode[];
  nextCursor: string | null;
  hasMore: boolean;
}

function parentCacheKey(spaceId: number, parentId: string | null): string {
  return `${spaceId}:${parentId ?? '__root__'}`;
}

export function getLoadedKnowledgeBrowserNodes(spaceId: number): KnowledgeBrowserNode[] {
  const prefix = `${spaceId}:`;
  const merged = new Map<string, KnowledgeBrowserNode>();
  for (const [key, entry] of browserParentCache.entries()) {
    if (!key.startsWith(prefix)) {
      continue;
    }
    for (const node of entry.nodes) {
      merged.set(node.id, node);
    }
  }
  return Array.from(merged.values());
}

function requireSdkClient() {
  return requireKnowledgebaseAppSdkHttpClient();
}

function purgeExpiredBrowserCacheEntries(now = Date.now()): void {
  for (const [key, entry] of browserParentCache.entries()) {
    if (now - entry.fetchedAt >= BROWSER_NODE_CACHE_TTL_MS) {
      browserParentCache.delete(key);
    }
  }
}

function trimBrowserCacheMaps(): void {
  while (browserParentCache.size > BROWSER_CACHE_MAX_ENTRIES) {
    const oldestKey = browserParentCache.keys().next().value;
    if (oldestKey === undefined) {
      break;
    }
    browserParentCache.delete(oldestKey);
  }
}

function rememberBrowserParentCacheEntry(
  cacheKey: string,
  nodes: KnowledgeBrowserNode[],
  nextCursor: string | null,
  hasMore: boolean,
): void {
  purgeExpiredBrowserCacheEntries();
  browserParentCache.set(cacheKey, {
    fetchedAt: Date.now(),
    nodes,
    nextCursor,
    hasMore,
  });
  trimBrowserCacheMaps();
}

function invalidateBrowserCachesForSpaceId(spaceId: number): void {
  const prefix = `${spaceId}:`;
  for (const key of browserParentCache.keys()) {
    if (key.startsWith(prefix)) {
      browserParentCache.delete(key);
    }
  }
}

export function invalidateKnowledgeBrowserNodeCache(spaceId?: number): void {
  if (spaceId === undefined) {
    browserParentCache.clear();
    return;
  }
  invalidateBrowserCachesForSpaceId(spaceId);
}

export function invalidateKnowledgeBrowserNodeCacheForSpaceIds(
  ...spaceIds: Array<number | null | undefined>
): void {
  for (const spaceId of spaceIds) {
    if (spaceId !== null && spaceId !== undefined && Number.isFinite(spaceId) && spaceId > 0) {
      invalidateBrowserCachesForSpaceId(spaceId);
    }
  }
}

export function invalidateKnowledgeBrowserNodeCacheForKbIds(
  ...kbIds: Array<string | null | undefined>
): void {
  for (const kbId of kbIds) {
    if (kbId === null || kbId === undefined) {
      continue;
    }
    const spaceId = Number(kbId);
    if (Number.isFinite(spaceId) && spaceId > 0) {
      invalidateBrowserCachesForSpaceId(spaceId);
    }
  }
}

export interface KnowledgeBrowserNodesPageResult {
  items: KnowledgeBrowserNode[];
  nextCursor: string | null;
  hasMore: boolean;
}

export async function listKnowledgeBrowserNodesPage(
  spaceId: number,
  parentId: string | null,
  options?: { cursor?: string | null; pageSize?: number; fresh?: boolean },
): Promise<KnowledgeBrowserNodesPageResult> {
  const pageSize = options?.pageSize ?? DEFAULT_BROWSER_PAGE_SIZE;
  const cursor = options?.cursor ?? null;
  const cacheKey = parentCacheKey(spaceId, parentId);

  if (!cursor && !options?.fresh) {
    const cached = browserParentCache.get(cacheKey);
    if (cached && Date.now() - cached.fetchedAt < BROWSER_NODE_CACHE_TTL_MS) {
      return {
        items: cached.nodes,
        nextCursor: cached.nextCursor,
        hasMore: cached.hasMore,
      };
    }
    if (cached) {
      browserParentCache.delete(cacheKey);
    }
  }

  const client = requireSdkClient();
  const page = await client.knowledge.spaces.browser.list(spaceId, {
    view: 'files',
    parentId,
    cursor,
    pageSize,
  });
  const normalized = normalizeSdkWorkListPage(page);
  const nextCursor = normalized.nextCursor;
  const hasMore = normalized.hasMore;

  if (!cursor) {
    rememberBrowserParentCacheEntry(cacheKey, normalized.items, nextCursor, hasMore);
  }

  return {
    items: normalized.items,
    nextCursor,
    hasMore,
  };
}

export async function listKnowledgeBrowserNodesForParent(
  spaceId: number,
  parentId: string | null,
  options?: { fresh?: boolean; pageSize?: number },
): Promise<KnowledgeBrowserNode[]> {
  const page = await listKnowledgeBrowserNodesPage(spaceId, parentId, {
    fresh: options?.fresh,
    pageSize: options?.pageSize,
  });
  return page.items;
}

export async function ensureKnowledgeBrowserFolderLoaded(
  spaceId: number,
  folderId: string | null,
): Promise<KnowledgeBrowserNode[]> {
  return listKnowledgeBrowserNodesForParent(spaceId, folderId);
}

export async function listLoadedKnowledgeBrowserNodes(
  spaceId: number,
  options?: { includeRoot?: boolean },
): Promise<KnowledgeBrowserNode[]> {
  if (options?.includeRoot !== false) {
    await listKnowledgeBrowserNodesForParent(spaceId, null);
  }
  return getLoadedKnowledgeBrowserNodes(spaceId);
}

export function findKnowledgeBrowserNodeByDocumentId(
  nodes: KnowledgeBrowserNode[],
  documentId: string,
  kbId: string,
): KnowledgeBrowserNode | null {
  return nodes.find(
    (candidate) => resolveBrowserDocumentId(candidate, kbId) === documentId,
  ) ?? nodes.find((candidate) => candidate.id === documentId) ?? null;
}

export function resolveBrowserDocumentId(node: KnowledgeBrowserNode, kbId: string): string {
  if (node.conceptId) {
    return `okf:${kbId}:${node.conceptId}`;
  }
  if (node.documentId) {
    return String(node.documentId);
  }
  return node.id;
}
