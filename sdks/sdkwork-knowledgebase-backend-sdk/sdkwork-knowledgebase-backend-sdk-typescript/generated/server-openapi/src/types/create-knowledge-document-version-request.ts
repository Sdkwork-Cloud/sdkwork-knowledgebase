export interface CreateKnowledgeDocumentVersionRequest {
  documentId: number;
  originalObjectRefId: number;
  checksumSha256Hex?: string | null;
  sizeBytes: number;
  mimeType?: string | null;
}
