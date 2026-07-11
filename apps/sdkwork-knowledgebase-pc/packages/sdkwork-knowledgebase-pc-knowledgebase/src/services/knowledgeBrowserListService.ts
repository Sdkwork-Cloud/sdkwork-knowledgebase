import type { KnowledgeBrowserNode, KnowledgeBrowserView } from 'sdkwork-knowledgebase-pc-core';
import {
  isKnowledgebaseAppError,
  parseKnowledgeSpaceId,
  requireKnowledgebaseAppSdkHttpClient,
} from 'sdkwork-knowledgebase-pc-core';
import { normalizeSdkWorkListPage } from './sdkWorkListPage';

const BROWSER_NODE_CACHE_TTL_MS = 30_000;
const BROWSER_CACHE_MAX_ENTRIES = 256;
const DEFAULT_BROWSER_PAGE_SIZE = 100;
const browserParentCache = new Map<string, BrowserNodeCacheEntry>();

interface BrowserNodeCacheEntry {
  fetchedAt: number;
  view: KnowledgeBrowserView;
  parentId: string | null;
  nodes: KnowledgeBrowserNode[];
  nextCursor: string | null;
  hasMore: boolean;
}

const DEFAULT_BROWSER_VIEW: KnowledgeBrowserView = 'files';

function parentCacheKey(
  spaceId: string,
  view: KnowledgeBrowserView,
  parentId: string | null,
): string {
  return `${spaceId}:${view}:${parentId ?? '__root__'}`;
}

export function getLoadedKnowledgeBrowserNodes(
  spaceId: string,
  options?: { view?: KnowledgeBrowserView },
): KnowledgeBrowserNode[] {
  const prefix = `${spaceId}:`;
  const view = options?.view ?? DEFAULT_BROWSER_VIEW;
  const merged = new Map<string, KnowledgeBrowserNode>();
  for (const [key, entry] of browserParentCache.entries()) {
    if (!key.startsWith(prefix)) {
      continue;
    }
    if (entry.view !== view) {
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
  view: KnowledgeBrowserView,
  parentId: string | null,
  nodes: KnowledgeBrowserNode[],
  nextCursor: string | null,
  hasMore: boolean,
): void {
  purgeExpiredBrowserCacheEntries();
  browserParentCache.set(cacheKey, {
    fetchedAt: Date.now(),
    view,
    parentId,
    nodes,
    nextCursor,
    hasMore,
  });
  trimBrowserCacheMaps();
}

function invalidateBrowserCachesForSpaceId(spaceId: string): void {
  const prefix = `${spaceId}:`;
  for (const key of browserParentCache.keys()) {
    if (key.startsWith(prefix)) {
      browserParentCache.delete(key);
    }
  }
}

export function invalidateKnowledgeBrowserNodeCache(spaceId?: string): void {
  if (spaceId === undefined) {
    browserParentCache.clear();
    return;
  }
  invalidateBrowserCachesForSpaceId(spaceId);
}

export function invalidateKnowledgeBrowserNodeCacheForSpaceIds(
  ...spaceIds: Array<string | null | undefined>
): void {
  for (const spaceId of spaceIds) {
    if (spaceId !== null && spaceId !== undefined && spaceId !== '' && spaceId !== '0') {
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
    try {
      invalidateBrowserCachesForSpaceId(parseKnowledgeSpaceId(kbId));
    } catch (error) {
      if (!isKnowledgebaseAppError(error)) {
        throw error;
      }
    }
  }
}

export interface KnowledgeBrowserNodesPageResult {
  parentId: string | null;
  items: KnowledgeBrowserNode[];
  nextCursor: string | null;
  hasMore: boolean;
}

function resolveBrowserPageParentId(page: unknown): string | null {
  if (typeof page !== 'object' || page === null || Array.isArray(page)) {
    return null;
  }
  const parentId = (page as { parentId?: unknown }).parentId;
  return typeof parentId === 'string' && parentId.trim() ? parentId : null;
}

export async function listKnowledgeBrowserNodesPage(
  spaceId: string,
  parentId: string | null,
  options?: {
    cursor?: string | null;
    pageSize?: number;
    fresh?: boolean;
    view?: KnowledgeBrowserView;
  },
): Promise<KnowledgeBrowserNodesPageResult> {
  const view = options?.view ?? DEFAULT_BROWSER_VIEW;
  const pageSize = options?.pageSize ?? DEFAULT_BROWSER_PAGE_SIZE;
  const cursor = options?.cursor ?? null;
  const cacheKey = parentCacheKey(spaceId, view, parentId);

  if (!cursor && !options?.fresh) {
    const cached = browserParentCache.get(cacheKey);
    if (cached && Date.now() - cached.fetchedAt < BROWSER_NODE_CACHE_TTL_MS) {
      return {
        parentId: cached.parentId,
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
    view,
    parentId,
    cursor,
    pageSize,
  });
  const normalized = normalizeSdkWorkListPage<KnowledgeBrowserNode>(page);
  const pageParentId = resolveBrowserPageParentId(page);
  const nextCursor = normalized.nextCursor;
  const hasMore = normalized.hasMore;

  if (!cursor) {
    rememberBrowserParentCacheEntry(
      cacheKey,
      view,
      pageParentId,
      normalized.items,
      nextCursor,
      hasMore,
    );
  }

  return {
    parentId: pageParentId,
    items: normalized.items,
    nextCursor,
    hasMore,
  };
}

export async function listKnowledgeBrowserNodesForParent(
  spaceId: string,
  parentId: string | null,
  options?: { fresh?: boolean; pageSize?: number; view?: KnowledgeBrowserView },
): Promise<KnowledgeBrowserNode[]> {
  const page = await listKnowledgeBrowserNodesPage(spaceId, parentId, {
    fresh: options?.fresh,
    pageSize: options?.pageSize,
    view: options?.view,
  });
  return page.items;
}

export async function ensureKnowledgeBrowserFolderLoaded(
  spaceId: string,
  folderId: string | null,
  options?: { view?: KnowledgeBrowserView },
): Promise<KnowledgeBrowserNode[]> {
  return listKnowledgeBrowserNodesForParent(spaceId, folderId, {
    view: options?.view,
  });
}

export async function listLoadedKnowledgeBrowserNodes(
  spaceId: string,
  options?: { includeRoot?: boolean; view?: KnowledgeBrowserView },
): Promise<KnowledgeBrowserNode[]> {
  const view = options?.view ?? DEFAULT_BROWSER_VIEW;
  if (options?.includeRoot !== false) {
    await listKnowledgeBrowserNodesForParent(spaceId, null, { view });
  }
  return getLoadedKnowledgeBrowserNodes(spaceId, { view });
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
