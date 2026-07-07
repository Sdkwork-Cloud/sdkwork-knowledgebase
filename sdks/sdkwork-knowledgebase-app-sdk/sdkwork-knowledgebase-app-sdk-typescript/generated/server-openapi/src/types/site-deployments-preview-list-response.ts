import type { KnowledgeSiteDeploymentPreview } from './knowledge-site-deployment-preview';

export interface SiteDeploymentsPreviewListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
