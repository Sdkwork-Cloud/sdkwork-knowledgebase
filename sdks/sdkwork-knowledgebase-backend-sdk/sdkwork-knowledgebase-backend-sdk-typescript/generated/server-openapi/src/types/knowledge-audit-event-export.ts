import type { KnowledgeAuditEventItem } from './knowledge-audit-event-item';

export interface KnowledgeAuditEventExport {
  /** Complete synchronous result for the subject. Requests matching more than 5,000 events fail with HTTP 413 and never return a truncated array. */
  items: KnowledgeAuditEventItem[];
}
