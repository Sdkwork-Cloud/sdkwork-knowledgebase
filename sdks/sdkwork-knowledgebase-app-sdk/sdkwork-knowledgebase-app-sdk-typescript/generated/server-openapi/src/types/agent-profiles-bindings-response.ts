import type { KnowledgeAgentBinding } from './knowledge-agent-binding';

export interface AgentProfilesBindingsResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
