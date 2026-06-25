const CONTENT_CACHE_KEY_PREFIX = 'sdkwork.knowledgebase.content.v2';
const MAX_CACHED_CONTENT_BYTES = 512 * 1024;

interface CachedDocumentContent {
  content: string;
  contentVersion: string;
  cachedAt: string;
}

function contentStorageKey(tenantId: string, documentId: string): string {
  return `${CONTENT_CACHE_KEY_PREFIX}.${tenantId}.${documentId}`;
}

function getSessionStorage(): Storage | null {
  if (typeof window === 'undefined') {
    return null;
  }
  try {
    return window.sessionStorage;
  } catch {
    return null;
  }
}

function parseCachedDocumentContent(raw: string): CachedDocumentContent | undefined {
  try {
    const parsed = JSON.parse(raw) as Partial<CachedDocumentContent>;
    if (
      typeof parsed.content === 'string'
      && typeof parsed.contentVersion === 'string'
      && parsed.contentVersion.length > 0
    ) {
      return {
        content: parsed.content,
        contentVersion: parsed.contentVersion,
        cachedAt: typeof parsed.cachedAt === 'string' ? parsed.cachedAt : new Date().toISOString(),
      };
    }
  } catch {
    // Fall through to legacy plain-text cache entries.
  }

  if (raw.length > 0) {
    return {
      content: raw,
      contentVersion: 'legacy',
      cachedAt: new Date().toISOString(),
    };
  }

  return undefined;
}

export function readLocalDocumentContent(
  tenantId: string,
  documentId: string,
  expectedVersion?: string,
): string | undefined {
  const storage = getSessionStorage();
  if (!storage) {
    return undefined;
  }

  try {
    const raw = storage.getItem(contentStorageKey(tenantId, documentId));
    if (!raw) {
      return undefined;
    }
    const cached = parseCachedDocumentContent(raw);
    if (!cached) {
      return undefined;
    }
    if (expectedVersion && cached.contentVersion !== expectedVersion) {
      return undefined;
    }
    return cached.content;
  } catch {
    return undefined;
  }
}

export function writeLocalDocumentContent(
  tenantId: string,
  documentId: string,
  content: string,
  contentVersion?: string,
): void {
  const storage = getSessionStorage();
  if (!storage) {
    return;
  }

  if (content.length > MAX_CACHED_CONTENT_BYTES) {
    return;
  }

  const payload: CachedDocumentContent = {
    content,
    contentVersion: contentVersion ?? new Date().toISOString(),
    cachedAt: new Date().toISOString(),
  };
  storage.setItem(contentStorageKey(tenantId, documentId), JSON.stringify(payload));
}

export function removeLocalDocumentContent(tenantId: string, documentId: string): void {
  const storage = getSessionStorage();
  if (!storage) {
    return;
  }

  storage.removeItem(contentStorageKey(tenantId, documentId));
}

export function clearLocalDocumentContentForTenant(tenantId: string): void {
  const storage = getSessionStorage();
  if (!storage) {
    return;
  }

  const prefix = `${CONTENT_CACHE_KEY_PREFIX}.${tenantId}.`;
  const keysToRemove: string[] = [];
  for (let index = 0; index < storage.length; index += 1) {
    const key = storage.key(index);
    if (key?.startsWith(prefix)) {
      keysToRemove.push(key);
    }
  }
  keysToRemove.forEach((key) => storage.removeItem(key));
}
