import type { KnowledgeWikiSourceFileCommandResult } from './knowledge-wiki-source-file-command-result';

export interface WikiSourceFilesPublishResponse {
  code: 0;
  data: unknown & { item: KnowledgeWikiSourceFileCommandResult; };
  /** Server-owned request correlation id. */
  traceId: string;
}
