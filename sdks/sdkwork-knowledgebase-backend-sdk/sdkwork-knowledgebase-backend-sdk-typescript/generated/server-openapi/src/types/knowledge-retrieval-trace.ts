export interface KnowledgeRetrievalTrace {
  retrievalTraceId: string;
  status: string;
  latencyMs?: number | null;
  resultCount: number;
}
