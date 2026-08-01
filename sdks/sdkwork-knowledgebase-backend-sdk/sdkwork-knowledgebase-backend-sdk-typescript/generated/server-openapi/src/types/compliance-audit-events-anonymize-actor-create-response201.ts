import type { AnonymizeKnowledgeAuditSubjectResult } from './anonymize-knowledge-audit-subject-result';

export interface ComplianceAuditEventsAnonymizeActorCreateResponse201 {
  code: 0;
  data: unknown & { item: AnonymizeKnowledgeAuditSubjectResult; };
  /** Server-owned request correlation id. */
  traceId: string;
}
