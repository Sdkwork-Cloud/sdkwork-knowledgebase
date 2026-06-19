import type { KnowledgeUploadSessionStatus } from './knowledge-upload-session-status';

export interface KnowledgeUploadSession {
  id: string;
  spaceId: string;
  title: string;
  uploadLogicalPath: string;
  status: KnowledgeUploadSessionStatus;
  expiresAt: string;
}
