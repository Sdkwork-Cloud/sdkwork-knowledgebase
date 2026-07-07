import type { KnowledgeAuditEventExport } from './knowledge-audit-event-export';

export interface ComplianceAuditEventsExportCreateResponse201 {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
