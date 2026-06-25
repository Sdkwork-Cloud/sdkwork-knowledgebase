import type { KnowledgeAccessLevel } from './knowledge-access-level';
import type { KnowledgeContextType } from './knowledge-context-type';

export interface CreateKnowledgeSpaceContextBindingRequest {
  spaceId: string;
  contextType: KnowledgeContextType;
  contextId: string;
  contextName?: string | null;
  accessLevel?: KnowledgeAccessLevel;
}
