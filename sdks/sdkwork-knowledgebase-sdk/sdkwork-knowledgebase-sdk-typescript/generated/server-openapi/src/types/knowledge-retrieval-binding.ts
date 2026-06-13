import type { KnowledgeFilter } from './knowledge-filter';

export interface KnowledgeRetrievalBinding {
  spaceId: string;
  collectionId?: string | null;
  sourceFilter?: KnowledgeFilter[] | null;
  documentFilter?: KnowledgeFilter[] | null;
  priority: number;
  topK?: number | null;
  minScore?: number | null;
}
