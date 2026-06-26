import type { KnowledgeBrowserNode } from '@sdkwork/knowledgebase-app-sdk';
import { isBlank, trim } from '@sdkwork/sdkwork-knowledgebase-pc-commons/stringUtils';
import { isKnowledgebaseDriveApiAvailable, requireDriveApiClient } from 'sdkwork-knowledgebase-pc-core';

import type { DocumentMeta, FolderNode } from './document';

export const DOCUMENT_TAGS_PROPERTY_KEY = 'sdkwork.knowledgebase.document.tags.v1';
export const DOCUMENT_ORDER_PROPERTY_KEY = 'sdkwork.knowledgebase.document.order.v1';

export interface DriveDocumentMetadata {
  tags: string[];
  order?: number;
}

function requireDriveClient() {
  return requireDriveApiClient();
}

export function resolveDriveNodeId(node: KnowledgeBrowserNode): string | null {
  return trim(node.driveNodeId) || trim(node.id) || null;
}

function parseTagsProperty(raw?: string | null): string[] {
  if (!raw) {
    return [];
  }
  try {
    const parsed = JSON.parse(raw) as unknown;
    if (!Array.isArray(parsed)) {
      return [];
    }
    return parsed.filter((entry): entry is string => typeof entry === 'string' && !isBlank(entry));
  } catch {
    return raw
      .split(',')
      .map((entry) => entry.trim())
      .filter(Boolean);
  }
}

function parseOrderProperty(raw?: string | null): number | undefined {
  if (!raw) {
    return undefined;
  }
  const value = Number(raw);
  return Number.isFinite(value) ? value : undefined;
}

export async function readDriveDocumentMetadata(
  driveNodeId: string,
): Promise<DriveDocumentMetadata> {
  const response = await requireDriveClient().nodeProperties.list(driveNodeId, {
    visibility: 'app_public',
    pageSize: 20,
  });
  const tagsEntry = response.items.find((item) => item.propertyKey === DOCUMENT_TAGS_PROPERTY_KEY);
  const orderEntry = response.items.find((item) => item.propertyKey === DOCUMENT_ORDER_PROPERTY_KEY);
  return {
    tags: parseTagsProperty(tagsEntry?.propertyValue),
    order: parseOrderProperty(orderEntry?.propertyValue),
  };
}

export async function writeDriveDocumentTags(
  driveNodeId: string,
  tags: string[],
): Promise<void> {
  const client = requireDriveClient();
  if (tags.length === 0) {
    try {
      await client.nodeProperties.delete(driveNodeId, DOCUMENT_TAGS_PROPERTY_KEY, {
        visibility: 'app_public',
      });
    } catch {
      // Property may not exist yet.
    }
    return;
  }

  await client.nodeProperties.set(driveNodeId, DOCUMENT_TAGS_PROPERTY_KEY, {
    value: JSON.stringify(tags),
    visibility: 'app_public',
  });
}

export async function writeDriveDocumentOrder(
  driveNodeId: string,
  order: number,
): Promise<void> {
  await requireDriveClient().nodeProperties.set(driveNodeId, DOCUMENT_ORDER_PROPERTY_KEY, {
    value: String(order),
    visibility: 'app_public',
  });
}

export async function listDriveFavoriteNodeIds(driveSpaceId: string): Promise<Set<string>> {
  const favorites = new Set<string>();
  const drive = requireDriveClient();
  let pageToken: string | undefined;

  do {
    const page = await drive.drive.favorites.list({
      spaceId: driveSpaceId,
      pageSize: '100',
      pageToken,
    });
    for (const node of page.items ?? []) {
      if (node.id) {
        favorites.add(node.id);
      }
    }
    pageToken = page.nextPageToken;
  } while (pageToken);

  return favorites;
}

async function enrichDocumentTreeItem(
  item: FolderNode | DocumentMeta,
  nodeByDocId: Map<string, KnowledgeBrowserNode>,
  loadOkfTags: (conceptRowId: number) => Promise<string[] | undefined>,
  favoriteNodeIds?: Set<string>,
): Promise<void> {
  if (item.type === 'folder') {
    const folderNode = nodeByDocId.get(item.id);
    const folderDriveNodeId = folderNode ? resolveDriveNodeId(folderNode) : null;
    if (folderDriveNodeId && favoriteNodeIds?.has(folderDriveNodeId)) {
      item.isPinned = true;
    }

    const children = (item as FolderNode).children ?? [];
    await Promise.all(
      children.map((child) => enrichDocumentTreeItem(child, nodeByDocId, loadOkfTags, favoriteNodeIds)),
    );
    return;
  }

  const okfMatch = /^okf:\d+:(\d+)$/.exec(item.id);
  if (okfMatch) {
    const tags = await loadOkfTags(Number(okfMatch[1]));
    if (tags) {
      item.tags = tags;
    }
    return;
  }

  const node = nodeByDocId.get(item.id);
  const driveNodeId = node ? resolveDriveNodeId(node) : null;
  if (!driveNodeId || !isKnowledgebaseDriveApiAvailable()) {
    return;
  }

  if (favoriteNodeIds?.has(driveNodeId)) {
    item.isPinned = true;
  }

  try {
    const metadata = await readDriveDocumentMetadata(driveNodeId);
    item.tags = metadata.tags;
    item.order = metadata.order;
  } catch {
    // Leave metadata unset when drive properties are unavailable.
  }
}

export async function enrichDocumentTreeMetadata(
  items: (FolderNode | DocumentMeta)[],
  nodes: KnowledgeBrowserNode[],
  kbId: string,
  loadOkfTags: (conceptRowId: number) => Promise<string[] | undefined>,
  favoriteNodeIds?: Set<string>,
): Promise<void> {
  const nodeByDocId = new Map<string, KnowledgeBrowserNode>();
  for (const node of nodes) {
    const docId = node.conceptId
      ? `okf:${kbId}:${node.conceptId}`
      : node.documentId
        ? String(node.documentId)
        : node.id;
    nodeByDocId.set(docId, node);
  }

  await Promise.all(
    items.map((item) => enrichDocumentTreeItem(item, nodeByDocId, loadOkfTags, favoriteNodeIds)),
  );
}
