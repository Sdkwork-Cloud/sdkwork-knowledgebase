import type { KnowledgeUploadSession } from './knowledge-upload-session';

export interface UploadSessionsCreateResponse201 {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
