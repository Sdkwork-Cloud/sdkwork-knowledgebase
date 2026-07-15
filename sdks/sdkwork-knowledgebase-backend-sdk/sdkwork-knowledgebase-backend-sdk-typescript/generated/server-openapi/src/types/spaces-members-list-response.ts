import type { KnowledgeSpaceMemberList } from './knowledge-space-member-list';

export interface SpacesMembersListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
