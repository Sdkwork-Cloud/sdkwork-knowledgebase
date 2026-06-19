const CONTENT_CACHE_KEY_PREFIX = 'sdkwork.knowledgebase.content.v1';

function contentStorageKey(tenantId: string, documentId: string): string {
  return `${CONTENT_CACHE_KEY_PREFIX}.${tenantId}.${documentId}`;
}

export function readLocalDocumentContent(
  tenantId: string,
  documentId: string,
): string | undefined {
  if (typeof window === 'undefined') {
    return undefined;
  }

  try {
    return window.localStorage.getItem(contentStorageKey(tenantId, documentId)) ?? undefined;
  } catch {
    return undefined;
  }
}

export function writeLocalDocumentContent(
  tenantId: string,
  documentId: string,
  content: string,
): void {
  if (typeof window === 'undefined') {
    return;
  }

  window.localStorage.setItem(contentStorageKey(tenantId, documentId), content);
}

export function removeLocalDocumentContent(tenantId: string, documentId: string): void {
  if (typeof window === 'undefined') {
    return;
  }

  window.localStorage.removeItem(contentStorageKey(tenantId, documentId));
}
