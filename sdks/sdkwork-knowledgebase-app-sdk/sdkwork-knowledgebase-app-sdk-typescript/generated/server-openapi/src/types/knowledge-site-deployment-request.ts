export interface KnowledgeSiteDeploymentRequest {
  spaceId: number;
  platform: string;
  siteName?: string | null;
  customDomain?: string | null;
  siteLogoDataUrl?: string | null;
}
