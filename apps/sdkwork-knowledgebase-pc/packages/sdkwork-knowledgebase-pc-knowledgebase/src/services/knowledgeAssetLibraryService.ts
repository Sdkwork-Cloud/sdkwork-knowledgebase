import type { KnowledgeBrowserNode } from 'sdkwork-knowledgebase-pc-core';
import { getKnowledgebaseTenantId, parseKnowledgeSpaceId, readRegisteredSpaces } from 'sdkwork-knowledgebase-pc-core';

import type { DocumentMeta } from './document';
import type { AssetType } from '../components/AssetLibraryModal';
import {
  listAllKnowledgeBrowserNodes,
  resolveBrowserDocumentId,
} from './knowledgeBrowserListService';
import { hydrateDocumentMediaUrl } from './knowledgeDriveMediaService';

export interface KnowledgeAssetLibraryItem {
  title: string;
  url: string;
  type: AssetType;
  duration?: string;
}

function spaceIdFromKbId(kbId: string): number {
  return parseKnowledgeSpaceId(kbId);
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

export async function listKnowledgeAssetLibraryItems(
  kbId: string,
  assetType: AssetType,
): Promise<KnowledgeAssetLibraryItem[]> {
  const spaceId = spaceIdFromKbId(kbId);
  const nodes = await listAllKnowledgeBrowserNodes(spaceId);
  const matches = nodes.filter((node) => mapAssetType(node) === assetType);
  const items: KnowledgeAssetLibraryItem[] = [];
  const kbIdString = String(spaceId);

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

  for (const entry of registry) {
    const kbId = String(entry.spaceId);
    const nodes = await listAllKnowledgeBrowserNodes(entry.spaceId);

    for (const node of nodes) {
      const assetType = mapAssetType(node);
      if (!assetType) {
        continue;
      }
      if (!node.name.toLowerCase().includes(trimmedQuery)) {
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
    }
  }

  return results;
}
