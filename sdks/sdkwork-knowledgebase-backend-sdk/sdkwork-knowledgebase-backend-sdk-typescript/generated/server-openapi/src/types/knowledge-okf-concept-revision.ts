import type { OkfRevisionReviewState } from './okf-revision-review-state';

export interface KnowledgeOkfConceptRevision {
  id: string;
  conceptRowId: string;
  revisionNo: string;
  markdownObjectRefId: string;
  contentHash: string;
  reviewState: OkfRevisionReviewState;
  createdAt: string;
}
