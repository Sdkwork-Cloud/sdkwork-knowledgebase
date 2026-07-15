import type { GroupKnowledgebaseLaunchTarget } from './group-knowledgebase-launch-target';

export interface GroupLaunchesConsumeResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
