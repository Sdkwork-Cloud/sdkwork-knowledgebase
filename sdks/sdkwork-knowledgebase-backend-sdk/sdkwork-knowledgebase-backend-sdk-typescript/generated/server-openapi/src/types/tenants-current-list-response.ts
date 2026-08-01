import type { KnowledgeTenantStatus } from './knowledge-tenant-status';

export interface TenantsCurrentListResponse {
  code: 0;
  data: unknown & { item: KnowledgeTenantStatus; };
  /** Server-owned request correlation id. */
  traceId: string;
}
