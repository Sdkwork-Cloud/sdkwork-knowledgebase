import type { GroupKnowledgebaseLaunchCapability } from './group-knowledgebase-launch-capability';

export interface GroupLaunchCapabilityResponse {
  code: 0;
  data: unknown & { item: GroupKnowledgebaseLaunchCapability; };
  /** Server-owned request correlation id. */
  traceId: string;
}
