import type { KnowledgeAgentProfile } from './knowledge-agent-profile';

export interface AgentProfilesCreateResponse201 {
  code: 0;
  data: unknown & { item: KnowledgeAgentProfile; };
  /** Server-owned request correlation id. */
  traceId: string;
}
