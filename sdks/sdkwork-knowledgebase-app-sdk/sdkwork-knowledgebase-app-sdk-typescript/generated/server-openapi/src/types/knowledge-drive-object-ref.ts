export interface KnowledgeDriveObjectRef {
  id: string;
  spaceId: string;
  driveSpaceId: string | null;
  driveNodeId: string | null;
  logicalPath?: string | null;
  contentType?: string | null;
  sizeBytes: string;
  checksumSha256Hex?: string | null;
  objectRole: string;
  accessMode: string;
}
