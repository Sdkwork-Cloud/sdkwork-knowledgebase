import type { KnowledgeSpaceMemberRole } from './knowledge-space-member-role';
import type { KnowledgeSpaceMemberSubjectType } from './knowledge-space-member-subject-type';

export interface GrantKnowledgeSpaceMemberRequest {
  subjectType: KnowledgeSpaceMemberSubjectType;
  subjectId: string;
  role: KnowledgeSpaceMemberRole;
}
