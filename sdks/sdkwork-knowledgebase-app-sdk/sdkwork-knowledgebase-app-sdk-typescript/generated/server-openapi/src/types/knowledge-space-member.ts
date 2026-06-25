import type { KnowledgeSpaceMemberRole } from './knowledge-space-member-role';
import type { KnowledgeSpaceMemberSubjectType } from './knowledge-space-member-subject-type';

export interface KnowledgeSpaceMember {
  subjectType: KnowledgeSpaceMemberSubjectType;
  subjectId: string;
  role: KnowledgeSpaceMemberRole;
  inherited: boolean;
}
