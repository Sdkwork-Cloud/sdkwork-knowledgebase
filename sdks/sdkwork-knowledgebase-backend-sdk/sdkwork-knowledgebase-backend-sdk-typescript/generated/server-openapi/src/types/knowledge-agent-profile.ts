import type { KnowledgeAgentBinding } from './knowledge-agent-binding';
import type { KnowledgeAgentStatus } from './knowledge-agent-status';

export interface KnowledgeAgentProfile {
  profileId: string;
  tenantId: string;
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
  status: KnowledgeAgentStatus;
  bindings: KnowledgeAgentBinding[];
}
