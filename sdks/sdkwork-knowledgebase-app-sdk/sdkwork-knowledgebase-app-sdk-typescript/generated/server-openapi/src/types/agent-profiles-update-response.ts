import type { KnowledgeAgentProfile } from './knowledge-agent-profile';

export interface AgentProfilesUpdateResponse {
  code: 0;
  data: unknown & { item: KnowledgeAgentProfile; };
  /** Server-owned request correlation id. */
  traceId: string;
}
