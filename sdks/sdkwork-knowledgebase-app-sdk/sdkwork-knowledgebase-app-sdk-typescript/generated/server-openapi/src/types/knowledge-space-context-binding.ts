import type { KnowledgeAccessLevel } from './knowledge-access-level';
import type { KnowledgeContextBindingStatus } from './knowledge-context-binding-status';
import type { KnowledgeContextType } from './knowledge-context-type';

export interface KnowledgeSpaceContextBinding {
  id: string;
  tenantId: string;
  spaceId: string;
  contextType: KnowledgeContextType;
  contextId: string;
  contextName?: string | null;
  accessLevel: KnowledgeAccessLevel;
  status: KnowledgeContextBindingStatus;
  createdBy: string;
  createdAt: string;
  updatedAt: string;
}
