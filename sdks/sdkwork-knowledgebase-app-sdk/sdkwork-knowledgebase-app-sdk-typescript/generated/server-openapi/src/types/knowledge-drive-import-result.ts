import type { IngestionJob } from './ingestion-job';
import type { KnowledgeDocument } from './knowledge-document';
import type { KnowledgeDocumentVersion } from './knowledge-document-version';
import type { KnowledgeDriveObjectRef } from './knowledge-drive-object-ref';
import type { KnowledgeSource } from './knowledge-source';

export interface KnowledgeDriveImportResult {
  source: KnowledgeSource;
  document: KnowledgeDocument;
  version: KnowledgeDocumentVersion;
  originalObjectRef: KnowledgeDriveObjectRef;
  job: IngestionJob;
}
