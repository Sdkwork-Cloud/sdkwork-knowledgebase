import type { KnowledgeBrowserNode } from '@sdkwork/knowledgebase-app-sdk';
import {
  getKnowledgebaseDriveAppSdkClient,
  isKnowledgebaseDriveApiAvailable,
} from 'sdkwork-knowledgebase-pc-core';

import type { DocumentMeta } from './document';
import { resolveDriveNodeId } from './knowledgeDriveDocumentMetadataService';

const MEDIA_TYPES = new Set<DocumentMeta['type']>([
  'image',
  'audio',
  'video',
  'pdf',
  'file',
  'music',
]);

function requireDriveClient() {
  if (!isKnowledgebaseDriveApiAvailable()) {
    return null;
  }
  return getKnowledgebaseDriveAppSdkClient().client;
}

export function isMediaDocumentType(type: DocumentMeta['type']): boolean {
  return MEDIA_TYPES.has(type);
}

export async function resolveDriveNodeDownloadUrl(driveNodeId: string): Promise<string | undefined> {
  const drive = requireDriveClient();
  if (!drive) {
    return undefined;
  }

  const response = await drive.drive.nodes.downloadUrls.create(driveNodeId, {
    requestedTtlSeconds: 3600,
  });
  return response.downloadUrl?.trim() || response.signedSourceUrl?.trim() || undefined;
}

export async function hydrateDocumentMediaUrl(
  doc: DocumentMeta,
  browserNode?: KnowledgeBrowserNode | null,
): Promise<DocumentMeta> {
  if (!isMediaDocumentType(doc.type) || doc.url) {
    return doc;
  }

  const driveNodeId = browserNode ? resolveDriveNodeId(browserNode) : null;
  if (!driveNodeId) {
    return doc;
  }

  try {
    const url = await resolveDriveNodeDownloadUrl(driveNodeId);
    if (!url) {
      return doc;
    }
    return { ...doc, url };
  } catch {
    return doc;
  }
}
