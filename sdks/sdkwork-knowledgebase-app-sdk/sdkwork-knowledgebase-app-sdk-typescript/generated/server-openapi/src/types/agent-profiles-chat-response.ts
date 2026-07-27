import type { KnowledgeAgentChatResponse } from './knowledge-agent-chat-response';

export interface AgentProfilesChatResponse {
  code: 0;
  data: unknown & { item: KnowledgeAgentChatResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
