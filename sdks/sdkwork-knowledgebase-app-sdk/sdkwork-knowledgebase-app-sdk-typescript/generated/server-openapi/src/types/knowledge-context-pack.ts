import type { KnowledgeCitation } from './knowledge-citation';
import type { KnowledgeContextFragment } from './knowledge-context-fragment';
import type { KnowledgeMemoryContextFragment } from './knowledge-memory-context-fragment';

export interface KnowledgeContextPack {
  contextPackId: string;
  retrievalId?: string | null;
  query: string;
  fragments: KnowledgeContextFragment[];
  estimatedTokens: number;
  citations: KnowledgeCitation[];
  truncated: boolean;
  memoryFragments: KnowledgeMemoryContextFragment[];
}
