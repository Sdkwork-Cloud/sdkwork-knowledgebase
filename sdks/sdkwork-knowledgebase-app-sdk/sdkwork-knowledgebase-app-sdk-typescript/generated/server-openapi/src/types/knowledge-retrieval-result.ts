import type { KnowledgeContextFragment } from './knowledge-context-fragment';
import type { KnowledgeRetrievalTrace } from './knowledge-retrieval-trace';

export interface KnowledgeRetrievalResult {
  retrievalId: string;
  trace?: KnowledgeRetrievalTrace | null;
  hits: KnowledgeContextFragment[];
}
