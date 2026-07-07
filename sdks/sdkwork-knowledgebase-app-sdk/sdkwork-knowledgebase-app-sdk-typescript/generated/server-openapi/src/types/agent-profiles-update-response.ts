import type { KnowledgeAgentProfile } from './knowledge-agent-profile';

export interface AgentProfilesUpdateResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
