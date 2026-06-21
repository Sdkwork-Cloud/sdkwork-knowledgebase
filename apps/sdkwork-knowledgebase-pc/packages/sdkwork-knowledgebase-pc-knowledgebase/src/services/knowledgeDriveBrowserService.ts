import type { KnowledgeBrowserNode } from '@sdkwork/knowledgebase-app-sdk';
import {
  getKnowledgebaseDriveAppSdkClient,
  isKnowledgebaseDriveApiAvailable,
} from 'sdkwork-knowledgebase-pc-core';

import type { DocumentMeta } from './document';

function requireDriveClient() {
  if (!isKnowledgebaseDriveApiAvailable()) {
    throw new Error('Drive SDK is required for browser node operations but is not configured.');
  }
  return getKnowledgebaseDriveAppSdkClient().client;
}

function resolveDriveNodeId(node: KnowledgeBrowserNode): string | null {
  return node.driveNodeId?.trim() || node.id?.trim() || null;
}

export async function applyDriveBrowserNodeUpdates(
  node: KnowledgeBrowserNode,
  updates: Pick<DocumentMeta, 'title' | 'parentId' | 'isPinned'>,
): Promise<void> {
  const driveNodeId = resolveDriveNodeId(node);
  if (!driveNodeId) {
    throw new Error('Drive node id is missing for this browser item.');
  }

  const drive = requireDriveClient();

  if (updates.title !== undefined && updates.title.trim() !== node.name) {
    await drive.drive.nodes.update(driveNodeId, {
      nodeName: updates.title.trim(),
    });
  }

  if (updates.parentId !== undefined) {
    const targetParent = updates.parentId?.trim() || undefined;
    const currentParent = node.parentId?.trim() || undefined;
    if (targetParent !== currentParent) {
      await drive.drive.nodes.move(driveNodeId, {
        targetParentNodeId: targetParent,
      });
    }
  }

  if (updates.isPinned !== undefined) {
    if (updates.isPinned) {
      await drive.drive.favorites.set(driveNodeId, {});
    } else {
      await drive.drive.favorites.delete(driveNodeId);
    }
  }
}

export async function deleteDriveBrowserNode(node: KnowledgeBrowserNode): Promise<void> {
  const driveNodeId = resolveDriveNodeId(node);
  if (!driveNodeId) {
    throw new Error('Drive node id is missing for this browser item.');
  }
  await requireDriveClient().drive.nodes.delete(driveNodeId);
}

export async function ensureDriveFolderPath(
  driveSpaceId: string,
  rootParentNodeId: string | null | undefined,
  relativePath: string | undefined,
  folderCache: Map<string, string>,
): Promise<string | undefined> {
  if (!relativePath || !relativePath.includes('/')) {
    return rootParentNodeId?.trim() || undefined;
  }

  const parts = relativePath.split('/');
  parts.pop();
  if (parts.length === 0) {
    return rootParentNodeId?.trim() || undefined;
  }

  let currentParent = rootParentNodeId?.trim() || undefined;
  let pathAccumulator = '';

  for (const folderName of parts) {
    pathAccumulator = pathAccumulator ? `${pathAccumulator}/${folderName}` : folderName;
    const cached = folderCache.get(pathAccumulator);
    if (cached) {
      currentParent = cached;
      continue;
    }

    const folder = await requireDriveClient().drive.nodes.folders.create({
      spaceId: driveSpaceId,
      parentNodeId: currentParent,
      nodeName: folderName,
    });
    folderCache.set(pathAccumulator, folder.id);
    currentParent = folder.id;
  }

  return currentParent;
}
