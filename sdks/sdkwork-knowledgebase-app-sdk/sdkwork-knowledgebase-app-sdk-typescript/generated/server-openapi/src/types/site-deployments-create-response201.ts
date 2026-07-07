import type { KnowledgeSiteDeploymentResult } from './knowledge-site-deployment-result';

export interface SiteDeploymentsCreateResponse201 {
  code: 0;
  data: unknown & KnowledgeSiteDeploymentResult;
  /** Server-owned request correlation id. */
  traceId: string;
}
