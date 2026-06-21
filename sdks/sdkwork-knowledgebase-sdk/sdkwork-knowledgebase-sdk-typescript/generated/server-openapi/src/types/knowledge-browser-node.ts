import type { KnowledgeBrowserNodePermissions } from './knowledge-browser-node-permissions';
import type { KnowledgeBrowserNodeType } from './knowledge-browser-node-type';

export interface KnowledgeBrowserNode {
  id: string;
  nodeType: KnowledgeBrowserNodeType;
  name: string;
  parentId?: string | null;
  path: string;
  driveSpaceId?: string | null;
  driveNodeId?: string | null;
  documentId?: number | null;
  documentVersionId?: number | null;
  conceptId?: number | null;
  conceptRevisionId?: number | null;
  mimeType?: string | null;
  sizeBytes?: number | null;
  ingestState?: string | null;
  parseState?: string | null;
  indexState?: string | null;
  okfState?: string | null;
  childrenCount?: number | null;
  updatedAt: string;
  driveStorageProviderId?: string | null;
  driveBucket?: string | null;
  driveObjectKey?: string | null;
  permissions: KnowledgeBrowserNodePermissions;
}
