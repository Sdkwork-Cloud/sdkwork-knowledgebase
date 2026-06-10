import type { KnowledgeFilter } from './knowledge-filter';

export interface KnowledgeAgentBindingRequest {
  tenantId: string;
  profileId: string;
  spaceId: string;
  collectionId?: string | null;
  sourceFilter?: KnowledgeFilter[] | null;
  documentFilter?: KnowledgeFilter[] | null;
  priority: number;
  topK?: number | null;
  minScore?: number | null;
  enabled: boolean;
}
