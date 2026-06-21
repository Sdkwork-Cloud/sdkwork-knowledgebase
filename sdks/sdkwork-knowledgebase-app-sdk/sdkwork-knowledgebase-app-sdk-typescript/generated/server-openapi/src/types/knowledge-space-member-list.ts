import type { KnowledgeSpaceMember } from './knowledge-space-member';

export interface KnowledgeSpaceMemberList {
  members: KnowledgeSpaceMember[];
  nextCursor?: string | null;
}
