import { isBlank, trim } from '@sdkwork/utils';
import {
  getKnowledgebaseAppSdkClient,
  isKnowledgebaseApiAvailable,
  shouldUseKnowledgebaseDemoFallback,
} from '../api/knowledgebaseApiRegistry';
import {
  getKnowledgebaseDriveAppSdkClient,
  isKnowledgebaseDriveApiAvailable,
} from '../api/knowledgebaseDriveApiRegistry';
import { getKnowledgebaseTenantId, readRegisteredSpaces } from '../api/knowledgebaseSpaceRegistry';
import type { KnowledgebaseAppSdkClient } from '../sdk/knowledgebaseAppSdkClient';
import { KnowledgebaseErrorCodes } from './knowledgebaseErrorCodes';
import { throwKnowledgebaseError } from './knowledgebaseAppError';

export function requireKnowledgebaseApiAvailable(
  code: string = KnowledgebaseErrorCodes.API_UNAVAILABLE,
): void {
  if (!isKnowledgebaseApiAvailable()) {
    throwKnowledgebaseError(code);
  }
}

/** Offline import / preview-only UI must not write synthetic data when API is live. */
export function assertKnowledgebasePreviewFeature(featureLabel: string): void {
  if (!shouldUseKnowledgebaseDemoFallback()) {
    throwKnowledgebaseError(KnowledgebaseErrorCodes.FEATURE_PREVIEW_ONLY, {
      cause: featureLabel,
    });
  }
}

export function requireKnowledgebaseAppSdkClient(): KnowledgebaseAppSdkClient {
  return getKnowledgebaseAppSdkClient();
}

export function requireKnowledgebaseAppSdkHttpClient() {
  return requireKnowledgebaseAppSdkClient().client;
}

export function parseKnowledgeSpaceId(kbId: string): number {
  const spaceId = Number(kbId);
  if (!Number.isFinite(spaceId) || spaceId <= 0) {
    throwKnowledgebaseError(KnowledgebaseErrorCodes.INVALID_SPACE_ID);
  }
  return spaceId;
}

export function requireDriveApiClient() {
  if (!isKnowledgebaseDriveApiAvailable()) {
    throwKnowledgebaseError(KnowledgebaseErrorCodes.API_UNAVAILABLE_DRIVE);
  }
  return getKnowledgebaseDriveAppSdkClient().client;
}

export function requireHttpUrl(value: string): URL {
  const trimmed = trim(value);
  if (isBlank(trimmed)) {
    throwKnowledgebaseError(KnowledgebaseErrorCodes.URL_REQUIRED);
  }

  let parsed: URL;
  try {
    parsed = new URL(trimmed);
  } catch {
    throwKnowledgebaseError(KnowledgebaseErrorCodes.INVALID_URL);
  }

  if (parsed.protocol !== 'http:' && parsed.protocol !== 'https:') {
    throwKnowledgebaseError(KnowledgebaseErrorCodes.URL_INVALID_SCHEME);
  }

  return parsed;
}

export function requireNonEmptyString(
  value: string,
  code: string,
): string {
  const normalized = trim(value);
  if (isBlank(normalized)) {
    throwKnowledgebaseError(code);
  }
  return normalized;
}

export function requirePositiveNumber(value: number, code: string): number {
  if (!Number.isFinite(value) || value <= 0) {
    throwKnowledgebaseError(code);
  }
  return value;
}

export function requireKnowledgebaseTenantId(): string {
  const tenantId = getKnowledgebaseTenantId();
  if (tenantId === undefined || isBlank(tenantId)) {
    throwKnowledgebaseError(KnowledgebaseErrorCodes.TENANT_CONTEXT_REQUIRED);
  }
  return tenantId.trim();
}

export function requirePrimaryRegisteredSpaceId(): number {
  const tenantId = requireKnowledgebaseTenantId();
  const registry = readRegisteredSpaces(tenantId);
  const firstSpace = registry[0];
  if (!firstSpace) {
    throwKnowledgebaseError(KnowledgebaseErrorCodes.SPACE_REQUIRED);
  }
  return firstSpace.spaceId;
}

export function requireDriveSpaceIdFromKbSpace(
  driveSpaceId: string | null | undefined,
): string {
  const normalized = trim(driveSpaceId ?? '');
  if (isBlank(normalized)) {
    throwKnowledgebaseError(KnowledgebaseErrorCodes.DRIVE_SPACE_MISSING);
  }
  return normalized;
}

export function requireDriveNodeId(
  driveNodeId: string | null | undefined,
): string {
  const normalized = trim(driveNodeId ?? '');
  if (isBlank(normalized)) {
    throwKnowledgebaseError(KnowledgebaseErrorCodes.DRIVE_NODE_ID_MISSING);
  }
  return normalized;
}
