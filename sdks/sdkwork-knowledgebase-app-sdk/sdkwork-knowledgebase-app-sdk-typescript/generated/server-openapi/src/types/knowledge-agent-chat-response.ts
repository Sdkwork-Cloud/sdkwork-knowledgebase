import type { KnowledgeAgentChatCitation } from './knowledge-agent-chat-citation';
import type { KnowledgeAgentKnowledgeMode } from './knowledge-agent-knowledge-mode';

export interface KnowledgeAgentChatResponse {
  chatId: string;
  answer: string;
  mode: KnowledgeAgentKnowledgeMode;
  agentImplementationId: string;
  modelProviderId: string;
  modelId: string;
  citations: KnowledgeAgentChatCitation[];
  retrievalId?: string | null;
  sessionId?: string | null;
}
