import type { KnowledgeTenantStatusEnum } from './knowledge-tenant-status-enum';
import type { KnowledgeTenantQuotaStatus } from './knowledge-tenant-quota-status';

export interface KnowledgeTenantStatus {
  tenantName?: string | null;
  status: KnowledgeTenantStatusEnum;
  spaceCount: string;
  documentCount: string;
  createdAt?: string | null;
  quota?: KnowledgeTenantQuotaStatus | null;
}
