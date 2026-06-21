import type { KnowledgeBrowserNode } from '@sdkwork/knowledgebase-app-sdk';
import { isBlank } from '@sdkwork/sdkwork-knowledgebase-pc-commons/stringUtils';
import { getKnowledgebaseAppSdkClient } from 'sdkwork-knowledgebase-pc-core';

export interface CloudDriveBrowserItem {
  id: string;
  name: string;
  type: 'folder' | 'file';
  size?: string;
  updatedAt: string;
  mimeType?: string | null;
  driveSpaceId?: string | null;
  driveNodeId?: string | null;
  driveStorageProviderId?: string | null;
  driveBucket?: string | null;
  driveObjectKey?: string | null;
  documentId?: number | null;
}

export interface CloudDriveImportResultItem {
  title: string;
  type: string;
  documentId?: number;
  content?: string;
}

export interface CloudDriveImportFailure {
  title: string;
  message: string;
}

function requireSdkClient() {
  const sdk = getKnowledgebaseAppSdkClient();
  if (!sdk) {
    throw new Error('Knowledgebase app SDK is not configured.');
  }
  return sdk.client;
}

function spaceIdFromKbId(kbId: string): number {
  const spaceId = Number(kbId);
  if (!Number.isFinite(spaceId) || spaceId <= 0) {
    throw new Error(`Invalid knowledge space id: ${kbId}`);
  }
  return spaceId;
}

function formatBytes(bytes: number | null | undefined): string | undefined {
  if (!bytes || bytes <= 0) {
    return undefined;
  }
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  if (bytes < 1024 * 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`;
  }
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function mapBrowserNode(node: KnowledgeBrowserNode): CloudDriveBrowserItem {
  const isFolder = node.nodeType === 'folder' || node.nodeType === 'virtual_folder';
  return {
    id: node.id,
    name: node.name,
    type: isFolder ? 'folder' : 'file',
    size: formatBytes(node.sizeBytes ?? undefined),
    updatedAt: node.updatedAt,
    mimeType: node.mimeType,
    driveSpaceId: node.driveSpaceId,
    driveNodeId: node.driveNodeId,
    driveStorageProviderId: node.driveStorageProviderId,
    driveBucket: node.driveBucket,
    driveObjectKey: node.driveObjectKey,
    documentId: node.documentId,
  };
}

function mapMimeToLegacyType(name: string, mimeType?: string | null): string {
  const lowerName = name.toLowerCase();
  if (lowerName.endsWith('.md') || lowerName.endsWith('.markdown') || mimeType?.includes('markdown')) {
    return 'markdown';
  }
  if (mimeType?.includes('html') || lowerName.endsWith('.html') || lowerName.endsWith('.htm')) {
    return 'richtext';
  }
  return 'file';
}

export async function listCloudDriveBrowserItems(
  spaceId: string,
  parentId?: string | null,
): Promise<CloudDriveBrowserItem[]> {
  const client = requireSdkClient();
  const numericSpaceId = spaceIdFromKbId(spaceId);
  const items: CloudDriveBrowserItem[] = [];
  let cursor: string | null | undefined;

  do {
    const page = await client.knowledge.spaces.browser.list(numericSpaceId, {
      view: 'files',
      parentId: parentId ?? null,
      cursor,
      pageSize: 100,
    });
    for (const node of page.items) {
      items.push(mapBrowserNode(node));
    }
    cursor = page.nextCursor;
  } while (cursor);

  return items;
}

function buildIdempotencyKey(spaceId: number, item: CloudDriveBrowserItem): string {
  const nodeId = item.driveNodeId ?? item.id;
  return `pc-drive-import-${spaceId}-${nodeId}`.slice(0, 128);
}

async function importDriveFile(
  numericSpaceId: number,
  item: CloudDriveBrowserItem,
): Promise<CloudDriveImportResultItem> {
  const client = requireSdkClient();
  const driveNodeId = item.driveNodeId ?? item.id;
  if (isBlank(driveNodeId)) {
    throw new Error(`Drive node id is missing for "${item.name}".`);
  }

  const result = await client.knowledge.driveImports.create({
    spaceId: numericSpaceId,
    title: item.name,
    idempotencyKey: buildIdempotencyKey(numericSpaceId, item),
    driveSpaceId: item.driveSpaceId ?? null,
    driveNodeId,
    driveStorageProviderId: '',
    driveBucket: '',
    driveObjectKey: '',
    language: null,
  });

  return {
    title: result.document.title,
    type: mapMimeToLegacyType(item.name, item.mimeType),
    documentId: result.document.id,
    content: `# ${result.document.title}\n\nImported from enterprise drive.`,
  };
}

export async function importCloudDriveItems(
  spaceId: string,
  items: CloudDriveBrowserItem[],
): Promise<CloudDriveImportResultItem[]> {
  const numericSpaceId = spaceIdFromKbId(spaceId);
  const imported: CloudDriveImportResultItem[] = [];
  const failures: CloudDriveImportFailure[] = [];

  for (const item of items) {
    if (item.type === 'folder') {
      continue;
    }

    try {
      imported.push(await importDriveFile(numericSpaceId, item));
    } catch (error) {
      failures.push({
        title: item.name,
        message: error instanceof Error ? error.message : String(error),
      });
    }
  }

  if (imported.length === 0 && failures.length > 0) {
    throw new Error(
      `Drive import failed: ${failures.map((entry) => `${entry.title} (${entry.message})`).join('; ')}`,
    );
  }

  if (failures.length > 0) {
    console.warn('[KnowledgeDriveImportService] partial import failures', failures);
  }

  return imported;
}
