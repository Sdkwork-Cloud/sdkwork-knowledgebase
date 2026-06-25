import type { KnowledgeBrowserNode } from '@sdkwork/knowledgebase-app-sdk';
import { requireKnowledgebaseAppSdkHttpClient } from 'sdkwork-knowledgebase-pc-core';

const BROWSER_NODE_CACHE_TTL_MS = 30_000;
const BROWSER_CACHE_MAX_ENTRIES = 256;
const browserParentCache = new Map<string, BrowserNodeCacheEntry>();

interface BrowserNodeCacheEntry {
  fetchedAt: number;
  nodes: KnowledgeBrowserNode[];
}

const browserNodeCache = new Map<number, BrowserNodeCacheEntry>();

function parentCacheKey(spaceId: number, parentId: string | null): string {
  return `${spaceId}:${parentId ?? '__root__'}`;
}

function collectLoadedBrowserNodes(spaceId: number): KnowledgeBrowserNode[] {
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
  for (const [spaceId, entry] of browserNodeCache.entries()) {
    if (now - entry.fetchedAt >= BROWSER_NODE_CACHE_TTL_MS) {
      browserNodeCache.delete(spaceId);
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
  while (browserNodeCache.size > BROWSER_CACHE_MAX_ENTRIES) {
    const oldestKey = browserNodeCache.keys().next().value;
    if (oldestKey === undefined) {
      break;
    }
    browserNodeCache.delete(oldestKey);
  }
}

function rememberBrowserParentCacheEntry(cacheKey: string, nodes: KnowledgeBrowserNode[]): void {
  purgeExpiredBrowserCacheEntries();
  browserParentCache.set(cacheKey, {
    fetchedAt: Date.now(),
    nodes,
  });
  trimBrowserCacheMaps();
}

function rememberBrowserNodeCacheEntry(spaceId: number, nodes: KnowledgeBrowserNode[]): void {
  purgeExpiredBrowserCacheEntries();
  browserNodeCache.set(spaceId, {
    fetchedAt: Date.now(),
    nodes,
  });
  trimBrowserCacheMaps();
}

function invalidateBrowserCachesForSpaceId(spaceId: number): void {
  browserNodeCache.delete(spaceId);
  const prefix = `${spaceId}:`;
  for (const key of browserParentCache.keys()) {
    if (key.startsWith(prefix)) {
      browserParentCache.delete(key);
    }
  }
}

export function invalidateKnowledgeBrowserNodeCache(spaceId?: number): void {
  if (spaceId === undefined) {
    browserNodeCache.clear();
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

export async function listKnowledgeBrowserNodesForParent(
  spaceId: number,
  parentId: string | null,
  options?: { fresh?: boolean },
): Promise<KnowledgeBrowserNode[]> {
  const cacheKey = parentCacheKey(spaceId, parentId);
  if (!options?.fresh) {
    const cached = browserParentCache.get(cacheKey);
    if (cached && Date.now() - cached.fetchedAt < BROWSER_NODE_CACHE_TTL_MS) {
      return cached.nodes;
    }
    if (cached) {
      browserParentCache.delete(cacheKey);
    }
  }

  const client = requireSdkClient();
  const items: KnowledgeBrowserNode[] = [];
  let cursor: string | null | undefined;

  do {
    const page = await client.knowledge.spaces.browser.list(spaceId, {
      view: 'files',
      parentId,
      cursor,
      pageSize: 100,
    });
    items.push(...page.items);
    cursor = page.nextCursor;
  } while (cursor);

  rememberBrowserParentCacheEntry(cacheKey, items);
  return items;
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
  return collectLoadedBrowserNodes(spaceId);
}

async function fetchAllKnowledgeBrowserNodes(spaceId: number): Promise<KnowledgeBrowserNode[]> {
  const all: KnowledgeBrowserNode[] = [];
  const queue: Array<string | null> = [null];
  const visitedParents = new Set<string>();

  while (queue.length > 0) {
    const parentId = queue.shift();
    const parentKey = parentId ?? '__root__';
    if (visitedParents.has(parentKey)) {
      continue;
    }
    visitedParents.add(parentKey);

    const children = await listKnowledgeBrowserNodesForParent(spaceId, parentId);
    for (const node of children) {
      all.push(node);
      if (node.nodeType === 'folder' || node.nodeType === 'virtual_folder') {
        queue.push(node.id);
      }
    }
  }

  return all;
}

export async function listAllKnowledgeBrowserNodes(
  spaceId: number,
  options?: { fresh?: boolean },
): Promise<KnowledgeBrowserNode[]> {
  if (!options?.fresh) {
    const cached = browserNodeCache.get(spaceId);
    if (cached && Date.now() - cached.fetchedAt < BROWSER_NODE_CACHE_TTL_MS) {
      return cached.nodes;
    }
    if (cached) {
      browserNodeCache.delete(spaceId);
    }
  }

  const nodes = await fetchAllKnowledgeBrowserNodes(spaceId);
  rememberBrowserNodeCacheEntry(spaceId, nodes);
  return nodes;
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
