import { isBlank } from '@sdkwork/utils';
import type { KnowledgeBrowserNode } from 'sdkwork-knowledgebase-pc-core';
import {
  getKnowledgebaseTenantId,
  parseKnowledgeSpaceId,
  readRegisteredSpaces,
  requireKnowledgebaseAppSdkHttpClient,
} from 'sdkwork-knowledgebase-pc-core';

import type { DocumentMeta } from './document';
import type { AssetType } from '../components/AssetLibraryModal';
import {
  listKnowledgeBrowserNodesPage,
  resolveBrowserDocumentId,
} from './knowledgeBrowserListService';
import { hydrateDocumentMediaUrl } from './knowledgeDriveMediaService';

export interface KnowledgeAssetLibraryItem {
  title: string;
  url: string;
  type: AssetType;
  duration?: string;
}

export interface KnowledgeAssetLibraryPage {
  items: KnowledgeAssetLibraryItem[];
  nextCursor: string | null;
  truncated: boolean;
}

interface AssetScanState {
  folderQueue: Array<string | null>;
  visitedParents: string[];
  activeParentId: string | null;
  activeBrowserCursor: string | null;
  scannedNodes: number;
}

const MAX_ASSET_LIBRARY_ITEMS = 200;
const MAX_ASSET_SCAN_NODES = 2000;
const ASSET_LIBRARY_PAGE_SIZE = 24;

function spaceIdFromKbId(kbId: string): string {
  return parseKnowledgeSpaceId(kbId);
}

function requireSdkClient() {
  return requireKnowledgebaseAppSdkHttpClient();
}

function isFolderNode(node: KnowledgeBrowserNode): boolean {
  return node.nodeType === 'folder' || node.nodeType === 'virtual_folder';
}

function mapAssetType(node: KnowledgeBrowserNode): AssetType | null {
  const mime = node.mimeType ?? '';
  const lowerName = node.name.toLowerCase();
  if (mime.startsWith('image/') || /\.(png|jpe?g|gif|webp|svg|bmp)$/i.test(lowerName)) {
    return 'image';
  }
  if (mime.startsWith('video/') || /\.(mp4|mov|webm|mkv|avi)$/i.test(lowerName)) {
    return 'video';
  }
  if (mime.startsWith('audio/') || /\.(mp3|wav|ogg|m4a|aac|flac)$/i.test(lowerName)) {
    return 'audio';
  }
  return null;
}

function initialScanState(): AssetScanState {
  return {
    folderQueue: [null],
    visitedParents: [],
    activeParentId: null,
    activeBrowserCursor: null,
    scannedNodes: 0,
  };
}

function encodeScanCursor(state: AssetScanState): string {
  return btoa(unescape(encodeURIComponent(JSON.stringify(state))));
}

function decodeScanCursor(cursor: string | null | undefined): AssetScanState {
  if (isBlank(cursor)) {
    return initialScanState();
  }
  try {
    const parsed = JSON.parse(decodeURIComponent(escape(atob(cursor!)))) as AssetScanState;
    return {
      folderQueue: Array.isArray(parsed.folderQueue) ? parsed.folderQueue : [null],
      visitedParents: Array.isArray(parsed.visitedParents) ? parsed.visitedParents : [],
      activeParentId: parsed.activeParentId ?? null,
      activeBrowserCursor: parsed.activeBrowserCursor ?? null,
      scannedNodes: Number.isFinite(parsed.scannedNodes) ? parsed.scannedNodes : 0,
    };
  } catch {
    return initialScanState();
  }
}

function scanStateExhausted(state: AssetScanState): boolean {
  return state.folderQueue.length === 0 && state.activeParentId === null;
}

async function collectAssetNodesPage(
  spaceId: string,
  assetType: AssetType,
  state: AssetScanState,
  targetCount: number,
): Promise<{ matches: KnowledgeBrowserNode[]; state: AssetScanState; truncated: boolean }> {
  const matches: KnowledgeBrowserNode[] = [];
  let truncated = false;
  const working = { ...state, folderQueue: [...state.folderQueue] };

  while (matches.length < targetCount && !scanStateExhausted(working)) {
    if (working.activeParentId === null) {
      if (working.folderQueue.length === 0) {
        break;
      }
      working.activeParentId = working.folderQueue.shift() ?? null;
      working.activeBrowserCursor = null;
      const parentKey = working.activeParentId ?? '__root__';
      if (working.visitedParents.includes(parentKey)) {
        working.activeParentId = null;
        continue;
      }
      working.visitedParents.push(parentKey);
    }

    const page = await listKnowledgeBrowserNodesPage(spaceId, working.activeParentId, {
      cursor: working.activeBrowserCursor,
    });

    for (const node of page.items) {
      working.scannedNodes += 1;
      if (mapAssetType(node) === assetType) {
        matches.push(node);
        if (matches.length >= targetCount) {
          break;
        }
      }
      if (isFolderNode(node)) {
        working.folderQueue.push(node.id);
      }
      if (working.scannedNodes >= MAX_ASSET_SCAN_NODES) {
        truncated = true;
        working.activeParentId = null;
        working.activeBrowserCursor = null;
        working.folderQueue = [];
        return { matches, state: working, truncated: true };
      }
    }

    if (matches.length >= targetCount) {
      break;
    }

    if (page.hasMore) {
      working.activeBrowserCursor = page.nextCursor;
    } else {
      working.activeParentId = null;
      working.activeBrowserCursor = null;
    }
  }

  if (working.scannedNodes >= MAX_ASSET_SCAN_NODES) {
    truncated = true;
  }

  return { matches, state: working, truncated };
}

async function hydrateAssetItems(
  matches: KnowledgeBrowserNode[],
  assetType: AssetType,
  kbIdString: string,
): Promise<KnowledgeAssetLibraryItem[]> {
  const items: KnowledgeAssetLibraryItem[] = [];
  for (const node of matches) {
    try {
      const doc: DocumentMeta = {
        id: resolveBrowserDocumentId(node, kbIdString),
        title: node.name,
        type: assetType === 'image' ? 'image' : assetType === 'video' ? 'video' : 'audio',
        kbId: kbIdString,
        parentId: node.parentId ?? null,
        updatedAt: node.updatedAt,
        author: 'Knowledgebase',
      };
      const hydrated = await hydrateDocumentMediaUrl(doc, node);
      if (!hydrated.url) {
        continue;
      }
      items.push({
        title: node.name,
        url: hydrated.url,
        type: assetType,
      });
    } catch {
      // Skip assets that cannot be resolved to a download URL.
    }
  }
  return items;
}

export async function listKnowledgeAssetLibraryItemsPage(
  kbId: string,
  assetType: AssetType,
  cursor?: string | null,
  pageSize = ASSET_LIBRARY_PAGE_SIZE,
): Promise<KnowledgeAssetLibraryPage> {
  const spaceId = spaceIdFromKbId(kbId);
  const kbIdString = String(spaceId);
  const scanState = decodeScanCursor(cursor);
  const { matches, state, truncated } = await collectAssetNodesPage(
    spaceId,
    assetType,
    scanState,
    Math.min(pageSize, MAX_ASSET_LIBRARY_ITEMS),
  );
  const items = await hydrateAssetItems(matches, assetType, kbIdString);
  const hasMore = !scanStateExhausted(state) && items.length > 0;
  return {
    items,
    nextCursor: hasMore ? encodeScanCursor(state) : null,
    truncated,
  };
}

/** @deprecated Prefer {@link listKnowledgeAssetLibraryItemsPage} for interactive lists. */
export async function listKnowledgeAssetLibraryItems(
  kbId: string,
  assetType: AssetType,
): Promise<{ items: KnowledgeAssetLibraryItem[]; truncated: boolean }> {
  const page = await listKnowledgeAssetLibraryItemsPage(kbId, assetType, null, MAX_ASSET_LIBRARY_ITEMS);
  return { items: page.items, truncated: page.truncated };
}

function mapDocumentTypeFromAsset(node: KnowledgeBrowserNode, assetType: AssetType): DocumentMeta['type'] {
  if (assetType === 'image') {
    return 'image';
  }
  if (assetType === 'video') {
    return 'video';
  }
  if (/\.(mp3|flac|wav|m4a|aac|ogg)$/i.test(node.name) || node.mimeType?.startsWith('audio/')) {
    return node.name.toLowerCase().includes('music') ? 'music' : 'audio';
  }
  return 'audio';
}

async function searchMediaInBrowserPages(
  spaceId: string,
  kbId: string,
  trimmedQuery: string,
  limit: number,
): Promise<DocumentMeta[]> {
  const results: DocumentMeta[] = [];
  const folderQueue: Array<string | null> = [null];
  const visitedParents = new Set<string>();
  let scannedNodes = 0;

  while (folderQueue.length > 0 && results.length < limit) {
    const parentId = folderQueue.shift()!;
    const parentKey = parentId ?? '__root__';
    if (visitedParents.has(parentKey)) {
      continue;
    }
    visitedParents.add(parentKey);

    let cursor: string | null = null;
    do {
      const page = await listKnowledgeBrowserNodesPage(spaceId, parentId, { cursor });
      for (const node of page.items) {
        scannedNodes += 1;
        const assetType = mapAssetType(node);
        if (!assetType || !node.name.toLowerCase().includes(trimmedQuery)) {
          if (isFolderNode(node)) {
            folderQueue.push(node.id);
          }
          if (scannedNodes >= MAX_ASSET_SCAN_NODES) {
            return results;
          }
          continue;
        }

        const doc: DocumentMeta = {
          id: resolveBrowserDocumentId(node, kbId),
          title: node.name,
          type: mapDocumentTypeFromAsset(node, assetType),
          kbId,
          parentId: node.parentId ?? null,
          updatedAt: node.updatedAt,
          author: 'Knowledgebase',
        };

        const hydrated = await hydrateDocumentMediaUrl(doc, node);
        if (!hydrated.url) {
          continue;
        }

        results.push(hydrated);
        if (results.length >= limit) {
          return results;
        }
        if (scannedNodes >= MAX_ASSET_SCAN_NODES) {
          return results;
        }
      }
      cursor = page.hasMore ? page.nextCursor : null;
    } while (cursor && results.length < limit);
  }

  return results;
}

export async function searchKnowledgeMediaDocuments(
  query: string,
  limit = 8,
): Promise<DocumentMeta[]> {
  const tenantId = getKnowledgebaseTenantId();
  if (!tenantId) {
    return [];
  }

  const trimmedQuery = query.trim().toLowerCase();
  if (!trimmedQuery) {
    return [];
  }

  const registry = readRegisteredSpaces(tenantId);
  const results: DocumentMeta[] = [];

  try {
    const client = requireSdkClient();
    const bindings = registry.map((entry, index) => ({
      spaceId: String(entry.spaceId),
      priority: index,
      topK: limit,
    }));
    const retrieval = await client.knowledge.retrievals.create({
      query: trimmedQuery,
      bindings,
      includeCitations: false,
      includeTrace: false,
      topK: limit,
    });

    for (const hit of retrieval.hits) {
      if (!hit.title?.toLowerCase().includes(trimmedQuery)) {
        continue;
      }
      const kbId = hit.spaceId;
      const doc: DocumentMeta = {
        id: hit.documentId,
        title: hit.title?.trim() || 'Untitled',
        type: 'file',
        kbId,
        updatedAt: new Date().toISOString(),
        author: 'Knowledgebase',
        content: hit.content,
      };
      results.push(doc);
      if (results.length >= limit) {
        return results;
      }
    }
  } catch {
    // Fall back to paginated browser search.
  }

  for (const entry of registry) {
    const kbId = String(entry.spaceId);
    const pageResults = await searchMediaInBrowserPages(entry.spaceId, kbId, trimmedQuery, limit - results.length);
    results.push(...pageResults);
    if (results.length >= limit) {
      break;
    }
  }

  return results;
}
