import type { KnowledgeOkfConceptRevision } from './knowledge-okf-concept-revision';
import type { PageInfo } from './page-info';

/** One bounded cursor page of OKF concept revisions. */
export interface KnowledgeOkfConceptRevisionList {
  items: KnowledgeOkfConceptRevision[];
  pageInfo: PageInfo;
}
