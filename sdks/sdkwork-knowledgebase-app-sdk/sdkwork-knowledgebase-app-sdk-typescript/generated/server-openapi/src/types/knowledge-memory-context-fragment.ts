export interface KnowledgeMemoryContextFragment {
  memoryId: string;
  title?: string | null;
  content: string;
  score?: number | null;
  rank: number;
  tokenCount?: number | null;
  sourceUri?: string | null;
  policyRef?: string | null;
}
