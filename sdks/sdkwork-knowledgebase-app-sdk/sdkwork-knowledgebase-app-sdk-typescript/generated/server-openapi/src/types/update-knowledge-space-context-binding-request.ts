import type { KnowledgeAccessLevel } from './knowledge-access-level';

export interface UpdateKnowledgeSpaceContextBindingRequest {
  contextName?: string | null;
  accessLevel?: KnowledgeAccessLevel;
}
