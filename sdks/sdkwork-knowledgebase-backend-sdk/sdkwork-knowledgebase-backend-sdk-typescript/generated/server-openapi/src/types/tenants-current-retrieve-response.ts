import type { KnowledgeTenantStatus } from './knowledge-tenant-status';

export interface TenantsCurrentRetrieveResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
