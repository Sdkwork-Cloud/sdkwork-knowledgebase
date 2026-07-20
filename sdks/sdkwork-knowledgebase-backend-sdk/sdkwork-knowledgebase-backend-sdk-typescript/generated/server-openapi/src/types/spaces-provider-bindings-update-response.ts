import type { KnowledgeEngineProviderBinding } from './knowledge-engine-provider-binding';

export interface SpacesProviderBindingsUpdateResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
