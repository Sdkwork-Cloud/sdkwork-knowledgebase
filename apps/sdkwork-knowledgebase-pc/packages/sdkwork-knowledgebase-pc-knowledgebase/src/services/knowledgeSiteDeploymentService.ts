import {
  KnowledgebaseErrorCodes,
  parseKnowledgeSpaceId,
  requireKnowledgebaseAppSdkHttpClient,
  requireNonEmptyString,
} from 'sdkwork-knowledgebase-pc-core';

export interface SiteDeploymentOptions {
  siteName?: string;
  customDomain?: string;
  siteLogoDataUrl?: string;
}

export interface SiteDeploymentResult {
  accepted: boolean;
  status: string;
  deploymentId: string;
  url: string;
}

export async function publishKnowledgeSite(
  kbId: string,
  platform: string,
  options?: SiteDeploymentOptions,
): Promise<SiteDeploymentResult> {
  const trimmedPlatform = requireNonEmptyString(
    platform,
    KnowledgebaseErrorCodes.DEPLOYMENT_PLATFORM_REQUIRED,
  );

  const client = requireKnowledgebaseAppSdkHttpClient();
  const result = await client.knowledge.siteDeployments.create({
    spaceId: parseKnowledgeSpaceId(kbId),
    platform: trimmedPlatform,
    siteName: options?.siteName?.trim() || undefined,
    customDomain: options?.customDomain?.trim() || undefined,
    siteLogoDataUrl: options?.siteLogoDataUrl || undefined,
  });

  return {
    accepted: result.accepted === true,
    status: result.status,
    deploymentId: result.deploymentId,
    url: result.url,
  };
}
