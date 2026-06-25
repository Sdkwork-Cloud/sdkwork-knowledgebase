import type { OkfRevisionReviewState } from './okf-revision-review-state';

export interface KnowledgeOkfConceptRevision {
  id: number;
  conceptRowId: number;
  revisionNo: number;
  markdownObjectRefId: number;
  contentHash: string;
  reviewState: OkfRevisionReviewState;
  createdAt: string;
}
