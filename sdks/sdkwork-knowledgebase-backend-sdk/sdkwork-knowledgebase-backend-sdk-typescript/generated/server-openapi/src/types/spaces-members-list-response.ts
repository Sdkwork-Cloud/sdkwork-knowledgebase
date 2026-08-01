import type { KnowledgeSpaceMemberList } from './knowledge-space-member-list';

export interface SpacesMembersListResponse {
  code: 0;
  data: unknown & { item: KnowledgeSpaceMemberList; };
  /** Server-owned request correlation id. */
  traceId: string;
}
