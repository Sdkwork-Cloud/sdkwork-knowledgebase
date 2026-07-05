export interface ExportKnowledgeAuditEventsRequest {
  actorId: string;
}

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

export interface KnowledgeAuditEventExport {
  items: KnowledgeAuditEventItem[];
}

export interface AnonymizeKnowledgeAuditSubjectRequest {
  actorId: string;
}

export interface AnonymizeKnowledgeAuditSubjectResult {
  anonymizedCount: string;
}
