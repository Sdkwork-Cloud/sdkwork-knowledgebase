import type { KnowledgeRetrievalBinding } from './knowledge-retrieval-binding';

export interface KnowledgeContextPackRequest {
  tenantId: string;
  actorId?: string | null;
  query: string;
  retrievalProfileId?: string | null;
  bindings: KnowledgeRetrievalBinding[];
  contextBudgetTokens: number;
  includeCitations: boolean;
  memoryPolicyRef?: string | null;
}
