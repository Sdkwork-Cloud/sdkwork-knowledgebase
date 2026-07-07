import type { KnowledgeMediaTaskResult } from './knowledge-media-task-result';

export interface MediaTasksCreateResponse201 {
  code: 0;
  data: unknown & KnowledgeMediaTaskResult;
  /** Server-owned request correlation id. */
  traceId: string;
}
