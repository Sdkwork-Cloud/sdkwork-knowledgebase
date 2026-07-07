export interface KnowledgeAuditEventItem {
  id: string;
  eventType: string;
  actorType: string;
  actorId: string;
  resourceType: string;
  resourceId?: string | null;
  result: string;
  traceId?: string | null;
  createdAt: string;
}
