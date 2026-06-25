import type { KnowledgeDocumentVersionState } from './knowledge-document-version-state';

export interface KnowledgeDocumentVersion {
  id: number;
  documentId: number;
  versionNo: number;
  originalObjectRefId: number;
  checksumSha256Hex?: string | null;
  sizeBytes: number;
  mimeType?: string | null;
  parseState: KnowledgeDocumentVersionState;
  indexState: KnowledgeDocumentVersionState;
}
