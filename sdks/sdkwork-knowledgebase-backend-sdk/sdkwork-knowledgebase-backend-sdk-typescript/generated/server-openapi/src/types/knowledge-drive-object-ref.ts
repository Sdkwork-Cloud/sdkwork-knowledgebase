export interface KnowledgeDriveObjectRef {
  id: number;
  spaceId: number;
  driveSpaceId?: string | null;
  driveNodeId?: string | null;
  logicalPath?: string | null;
  driveProviderKind: string;
  driveBucket: string;
  driveObjectKey: string;
  driveObjectVersion?: string | null;
  driveEtag?: string | null;
  contentType?: string | null;
  sizeBytes: number;
  checksumSha256Hex?: string | null;
  objectRole: string;
  accessMode: string;
  driveStorageProviderId: string;
}
