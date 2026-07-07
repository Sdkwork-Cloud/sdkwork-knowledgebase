import type { KnowledgeGitSyncResult } from './knowledge-git-sync-result';

export interface GitSyncsCreateResponse201 {
  code: 0;
  data: unknown & KnowledgeGitSyncResult;
  /** Server-owned request correlation id. */
  traceId: string;
}
