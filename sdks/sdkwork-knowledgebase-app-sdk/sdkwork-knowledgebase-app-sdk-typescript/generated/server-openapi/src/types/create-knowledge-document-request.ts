import type { KnowledgeDocumentVisibility } from './knowledge-document-visibility';

export interface CreateKnowledgeDocumentRequest {
  spaceId: number;
  collectionId?: number;
  sourceId?: number | null;
  title: string;
  mimeType?: string | null;
  language?: string | null;
  visibility?: KnowledgeDocumentVisibility;
}
