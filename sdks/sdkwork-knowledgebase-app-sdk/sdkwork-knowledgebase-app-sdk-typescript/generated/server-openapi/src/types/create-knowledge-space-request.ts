import type { KnowledgeAgentKnowledgeMode } from './knowledge-agent-knowledge-mode';

export interface CreateKnowledgeSpaceRequest {
  name: string;
  description?: string | null;
  knowledgeMode?: KnowledgeAgentKnowledgeMode;
}
