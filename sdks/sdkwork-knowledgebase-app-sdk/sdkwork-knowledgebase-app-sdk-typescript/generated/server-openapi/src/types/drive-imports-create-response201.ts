import type { KnowledgeDriveImportResult } from './knowledge-drive-import-result';

export interface DriveImportsCreateResponse201 {
  code: 0;
  data: unknown & { item: KnowledgeDriveImportResult; };
  /** Server-owned request correlation id. */
  traceId: string;
}
