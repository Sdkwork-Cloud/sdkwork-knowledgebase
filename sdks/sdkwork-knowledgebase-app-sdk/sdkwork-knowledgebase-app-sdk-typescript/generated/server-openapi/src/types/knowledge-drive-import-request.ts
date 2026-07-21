export interface KnowledgeDriveImportRequest {
  spaceId: string;
  title: string;
  driveSpaceId: string;
  driveNodeId: string;
  idempotencyKey: string;
  language?: string | null;
}
