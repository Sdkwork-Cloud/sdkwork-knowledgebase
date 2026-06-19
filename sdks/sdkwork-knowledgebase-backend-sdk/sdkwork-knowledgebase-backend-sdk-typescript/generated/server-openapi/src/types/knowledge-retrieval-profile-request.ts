export interface KnowledgeRetrievalProfileRequest {
  name: string;
  strategy: string;
  topK: number;
  minScore?: number | null;
  rerankEnabled: boolean;
  contextBudgetTokens: number;
  status: string;
}
