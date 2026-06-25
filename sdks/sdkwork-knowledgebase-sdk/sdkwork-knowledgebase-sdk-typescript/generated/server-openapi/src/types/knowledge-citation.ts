export interface KnowledgeCitation {
  documentId: string;
  documentVersionId?: string | null;
  chunkId?: string | null;
  title: string;
  sourceUri?: string | null;
  locator?: string | null;
  score?: number | null;
}
