import type { KnowledgeSpaceContextBinding } from './knowledge-space-context-binding';

export interface KnowledgeSpaceContextBindingList {
  items: KnowledgeSpaceContextBinding[];
  nextCursor?: string | null;
}
