import type { KnowledgeAuditEventExport } from './knowledge-audit-event-export';

export interface ComplianceAuditEventsExportCreateResponse201 {
  code: 0;
  data: unknown & { item: KnowledgeAuditEventExport; };
  /** Server-owned request correlation id. */
  traceId: string;
}
