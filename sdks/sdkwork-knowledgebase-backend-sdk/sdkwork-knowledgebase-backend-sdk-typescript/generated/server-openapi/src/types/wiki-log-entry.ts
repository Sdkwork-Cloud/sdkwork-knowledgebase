export interface WikiLogEntry {
  occurredAt: string;
  eventType: string;
  title: string;
  actor: string;
  affectedPages: string[];
  auditEventId?: string | null;
  warnings: string[];
}
