export interface KnowledgeRetrievalProfile {
  retrievalProfileId: string;
  tenantId: string;
  name: string;
  strategy: string;
  topK: number;
  minScore?: number | null;
  rerankEnabled: boolean;
  contextBudgetTokens: number;
  status: string;
}
