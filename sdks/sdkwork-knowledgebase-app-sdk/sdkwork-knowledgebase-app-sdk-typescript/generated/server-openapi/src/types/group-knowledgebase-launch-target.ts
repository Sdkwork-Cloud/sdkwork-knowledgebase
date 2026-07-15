import type { GroupKnowledgeSpaceLifecycleState } from './group-knowledge-space-lifecycle-state';

export interface GroupKnowledgebaseLaunchTarget {
  conversationId: string;
  spaceId: string;
  spaceUuid: string;
  groupName: string;
  lifecycleState: GroupKnowledgeSpaceLifecycleState;
}
