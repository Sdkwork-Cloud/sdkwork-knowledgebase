import {
  KnowledgebaseErrorCodes,
  parseKnowledgeSpaceId,
  requireKnowledgebaseAppSdkHttpClient,
  requireNonEmptyString,
  throwKnowledgebaseError,
} from 'sdkwork-knowledgebase-pc-core';
import { isBlank, trim } from '@sdkwork/utils';

export interface SitePublicationOptions {
  siteName?: string;
  customHost?: string;
}

export interface SitePublicationResult {
  siteId: string;
  releaseId: string;
  url: string;
}

export function isVerifiedKnowledgeSiteUrl(value: unknown): value is string {
  if (typeof value !== 'string' || isBlank(value)) {
    return false;
  }
  try {
    const url = new URL(value);
    if (url.username || url.password || !url.hostname) {
      return false;
    }
    if (url.protocol === 'https:') {
      return true;
    }
    return url.protocol === 'http:' && isLocalNetworkHostname(url.hostname);
  } catch {
    return false;
  }
}

export async function publishKnowledgeSite(
  kbId: string,
  options?: SitePublicationOptions,
): Promise<SitePublicationResult> {
  const spaceId = parseKnowledgeSpaceId(kbId);
  const title = requireNonEmptyString(
    options?.siteName ?? `Knowledgebase ${spaceId}`,
    KnowledgebaseErrorCodes.DEPLOYMENT_PLATFORM_REQUIRED,
  );
  const client = requireKnowledgebaseAppSdkHttpClient();
  const existing = await retrieveSiteOrNull(client, spaceId);
  let site = await client.knowledge.sites.update(spaceId, {
    spaceId,
    title,
    visibility: 'public',
    homepageConceptId: existing?.homepageConceptId ?? null,
    themeId: existing?.themeId ?? 'default',
    publishMode: existing?.publishMode ?? 'manual',
    expectedVersion: existing?.version ?? null,
  });

  const customHost = trim(options?.customHost ?? '');
  if (customHost) {
    const bindingType = customHost.includes('.') ? 'external_domain' : 'custom_prefix';
    try {
      await client.knowledge.siteHostBindings.create(site.id, {
        bindingType,
        host: customHost,
        canonical: bindingType === 'custom_prefix',
        expectedSiteVersion: site.version,
      });
      site = await client.knowledge.sites.retrieve(spaceId);
    } catch (error) {
      if (httpStatusOf(error) !== 409) {
        throw error;
      }
      site = await client.knowledge.sites.retrieve(spaceId);
    }
  }

  const publication = await client.knowledge.siteReleases.create(site.id, {
    expectedSiteVersion: site.version,
  });
  const siteId = trim(String(publication.site?.id ?? ''));
  const releaseId = trim(String(publication.release?.id ?? ''));
  if (!siteId || !releaseId || !isVerifiedKnowledgeSiteUrl(publication.publicUrl)) {
    throwKnowledgebaseError(KnowledgebaseErrorCodes.OPERATION_FAILED, {
      cause: new Error('site publication response is missing release evidence or a safe URL'),
    });
  }
  return {
    siteId,
    releaseId,
    url: trim(publication.publicUrl),
  };
}

async function retrieveSiteOrNull(
  client: ReturnType<typeof requireKnowledgebaseAppSdkHttpClient>,
  spaceId: string,
) {
  try {
    return await client.knowledge.sites.retrieve(spaceId);
  } catch (error) {
    if (httpStatusOf(error) === 404) {
      return null;
    }
    throw error;
  }
}

function httpStatusOf(error: unknown): number | undefined {
  if (!error || typeof error !== 'object') {
    return undefined;
  }
  const candidate = error as {
    httpStatus?: unknown;
    status?: unknown;
    response?: { status?: unknown };
    cause?: { status?: unknown; response?: { status?: unknown } };
  };
  for (const value of [
    candidate.httpStatus,
    candidate.status,
    candidate.response?.status,
    candidate.cause?.status,
    candidate.cause?.response?.status,
  ]) {
    if (typeof value === 'number') {
      return value;
    }
  }
  return undefined;
}

function isLocalNetworkHostname(hostname: string): boolean {
  if (hostname === 'localhost' || hostname === '127.0.0.1' || hostname === '::1') {
    return true;
  }
  const octets = hostname.split('.').map(Number);
  if (octets.length !== 4 || octets.some((octet) => !Number.isInteger(octet) || octet < 0 || octet > 255)) {
    return false;
  }
  return octets[0] === 10
    || octets[0] === 127
    || (octets[0] === 172 && octets[1] >= 16 && octets[1] <= 31)
    || (octets[0] === 192 && octets[1] === 168)
    || (octets[0] === 169 && octets[1] === 254);
}

