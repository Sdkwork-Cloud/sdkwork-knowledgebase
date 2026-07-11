import type { OkfConceptSummary } from './okf-concept-summary';
import type { PageInfo } from './page-info';

/** One bounded cursor page of published OKF concept summaries. */
export interface OkfConceptSummaryList {
  items: OkfConceptSummary[];
  pageInfo: PageInfo;
}
