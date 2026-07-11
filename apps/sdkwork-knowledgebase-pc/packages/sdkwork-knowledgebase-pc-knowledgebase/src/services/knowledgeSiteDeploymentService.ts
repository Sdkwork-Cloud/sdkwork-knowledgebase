import {
  KnowledgebaseErrorCodes,
  parseKnowledgeSpaceId,
  requireKnowledgebaseAppSdkHttpClient,
  requireNonEmptyString,
  throwKnowledgebaseError,
} from 'sdkwork-knowledgebase-pc-core';

export interface SiteDeploymentOptions {
  siteName?: string;
  customDomain?: string;
  siteLogoDataUrl?: string;
}

export interface SiteDeploymentResult {
  accepted: true;
  status: 'completed';
  deploymentId: string;
  url: string;
}

export function isVerifiedSiteDeploymentUrl(value: unknown): value is string {
  if (typeof value !== 'string' || !value.trim()) {
    return false;
  }
  try {
    const url = new URL(value);
    return url.protocol === 'https:'
      && url.hostname.length > 0
      && !url.username
      && !url.password;
  } catch {
    return false;
  }
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

  const deploymentId = String(result.deploymentId ?? '').trim();
  if (
    result.accepted !== true
    || result.status !== 'completed'
    || !deploymentId
    || !isVerifiedSiteDeploymentUrl(result.url)
  ) {
    throwKnowledgebaseError(KnowledgebaseErrorCodes.OPERATION_FAILED, {
      cause: new Error('site deployment response did not contain verified publisher evidence'),
    });
  }

  return {
    accepted: true,
    status: 'completed',
    deploymentId,
    url: result.url.trim(),
  };
}
