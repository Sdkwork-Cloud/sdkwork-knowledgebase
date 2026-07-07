import type { KnowledgeSpaceContextBinding } from './knowledge-space-context-binding';
import type { PageInfo } from './page-info';

export interface SpacesContextBindingsListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
