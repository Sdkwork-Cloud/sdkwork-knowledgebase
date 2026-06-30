import type { KnowledgeBrowserNode } from 'sdkwork-knowledgebase-pc-core';
import { isBlank, trim } from '@sdkwork/utils';
import {
  KnowledgebaseErrorCodes,
  parseKnowledgeSpaceId,
  throwKnowledgebaseError,
} from 'sdkwork-knowledgebase-pc-core';

import {
  findKnowledgeBrowserNodeByDocumentId,
  listAllKnowledgeBrowserNodes,
} from './knowledgeBrowserListService';
import { resolveDriveNodeId } from './knowledgeDriveDocumentMetadataService';

function spaceIdFromKbId(kbId: string): number {
  return parseKnowledgeSpaceId(kbId);
}

function findBrowserNodeByParentReference(
  nodes: KnowledgeBrowserNode[],
  parentReference: string,
  kbId: string,
): KnowledgeBrowserNode | null {
  return findKnowledgeBrowserNodeByDocumentId(nodes, parentReference, kbId)
    ?? nodes.find((candidate) => candidate.id === parentReference)
    ?? null;
}

function throwParentFolderNotFound(parentReference: string): never {
  throwKnowledgebaseError(KnowledgebaseErrorCodes.PARENT_FOLDER_NOT_FOUND, {
    cause: parentReference,
  });
}

export async function resolveKnowledgeBrowserParentDriveNodeId(
  kbId: string,
  parentReference: string | null | undefined,
): Promise<string | undefined> {
  if (isBlank(parentReference)) {
    return undefined;
  }

  const trimmed = trim(parentReference)!;
  const nodes = await listAllKnowledgeBrowserNodes(spaceIdFromKbId(kbId));
  const node = findBrowserNodeByParentReference(nodes, trimmed, kbId);
  if (!node) {
    throwParentFolderNotFound(trimmed);
  }

  const driveNodeId = resolveDriveNodeId(node);
  if (!driveNodeId) {
    throwKnowledgebaseError(KnowledgebaseErrorCodes.PARENT_DRIVE_NODE_MISSING, {
      cause: trimmed,
    });
  }

  return driveNodeId;
}

export async function resolveKnowledgeBrowserParentNodeId(
  kbId: string,
  parentReference: string | null | undefined,
): Promise<string | null> {
  if (isBlank(parentReference)) {
    return null;
  }

  const trimmed = trim(parentReference)!;
  const nodes = await listAllKnowledgeBrowserNodes(spaceIdFromKbId(kbId));
  const node = findBrowserNodeByParentReference(nodes, trimmed, kbId);
  if (!node) {
    throwParentFolderNotFound(trimmed);
  }

  return node.id;
}
