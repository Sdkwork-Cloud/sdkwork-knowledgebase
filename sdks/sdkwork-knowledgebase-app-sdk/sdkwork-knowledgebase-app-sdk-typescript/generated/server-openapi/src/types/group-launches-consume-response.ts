import type { GroupKnowledgebaseLaunchTarget } from './group-knowledgebase-launch-target';

export interface GroupLaunchesConsumeResponse {
  code: 0;
  data: unknown & { item: GroupKnowledgebaseLaunchTarget; };
  /** Server-owned request correlation id. */
  traceId: string;
}
