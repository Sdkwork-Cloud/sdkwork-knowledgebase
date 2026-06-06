export interface KnowledgeDriveImportRequest {
  spaceId: number;
  title: string;
  driveBucket: string;
  driveObjectKey: string;
  idempotencyKey: string;
  language?: string | null;
  driveSpaceId?: string | null;
  driveNodeId?: string | null;
}
