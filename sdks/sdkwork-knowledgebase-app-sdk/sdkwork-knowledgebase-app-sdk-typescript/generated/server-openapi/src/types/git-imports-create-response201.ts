import type { KnowledgeGitImportResult } from './knowledge-git-import-result';

export interface GitImportsCreateResponse201 {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
