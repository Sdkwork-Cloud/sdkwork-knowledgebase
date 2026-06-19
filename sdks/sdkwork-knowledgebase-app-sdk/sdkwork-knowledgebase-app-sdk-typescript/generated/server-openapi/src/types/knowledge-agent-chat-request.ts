import type { KnowledgeAgentKnowledgeMode } from './knowledge-agent-knowledge-mode';

export interface KnowledgeAgentChatRequest {
  actorId?: string | null;
  message: string;
  mode?: KnowledgeAgentKnowledgeMode;
  sessionId?: string | null;
  modelProviderId?: string | null;
  modelId?: string | null;
  agentImplementationId?: string | null;
}
