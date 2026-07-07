export interface KnowledgeSiteDeploymentRequest {
  spaceId: string;
  platform: string;
  siteName?: string | null;
  customDomain?: string | null;
  siteLogoDataUrl?: string | null;
}
