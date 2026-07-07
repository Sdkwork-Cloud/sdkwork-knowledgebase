import type { KnowledgeDriveImportResult } from './knowledge-drive-import-result';

export interface DriveImportsCreateResponse201 {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
