import type { KnowledgeAgentKnowledgeMode } from './knowledge-agent-knowledge-mode';
import type { KnowledgeAgentStatus } from './knowledge-agent-status';

export interface KnowledgeAgentProfileRequest {
  name: string;
  description?: string | null;
  systemInstruction: string;
  modelProviderId: string;
  modelId: string;
  modelParameters?: string | null;
  retrievalProfileId?: string | null;
  citationPolicy?: string | null;
  memoryPolicyRef?: string | null;
  toolPolicyRef?: string | null;
  answerPolicy?: string | null;
  knowledgeMode?: KnowledgeAgentKnowledgeMode;
  agentImplementationId?: string;
  status: KnowledgeAgentStatus;
}
