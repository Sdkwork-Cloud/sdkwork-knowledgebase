const RECENT_DOCS_KEY_PREFIX = 'sdkwork.knowledgebase.recent.v1';
const MAX_RECENT_DOCUMENTS = 32;

export interface RecentDocumentEntry {
  id: string;
  title: string;
  type: 'richtext' | 'code' | 'markdown' | 'file' | 'image' | 'audio' | 'video' | 'folder' | 'pdf' | 'music';
  kbId?: string;
  updatedAt: string;
  author?: string;
}

function recentStorageKey(tenantId: string): string {
  return `${RECENT_DOCS_KEY_PREFIX}.${tenantId}`;
}

export function readRecentDocuments(tenantId: string): RecentDocumentEntry[] {
  if (typeof window === 'undefined') {
    return [];
  }

  try {
    const raw = window.localStorage.getItem(recentStorageKey(tenantId));
    if (!raw) {
      return [];
    }
    const parsed = JSON.parse(raw) as RecentDocumentEntry[];
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

export function touchRecentDocument(
  tenantId: string,
  entry: RecentDocumentEntry,
): RecentDocumentEntry[] {
  if (typeof window === 'undefined') {
    return [];
  }

  const next = readRecentDocuments(tenantId).filter((item) => item.id !== entry.id);
  next.unshift({
    ...entry,
    updatedAt: entry.updatedAt || new Date().toISOString(),
  });

  const trimmed = next.slice(0, MAX_RECENT_DOCUMENTS);
  window.localStorage.setItem(recentStorageKey(tenantId), JSON.stringify(trimmed));
  return trimmed;
}

export function removeRecentDocument(tenantId: string, documentId: string): void {
  if (typeof window === 'undefined') {
    return;
  }

  const next = readRecentDocuments(tenantId).filter((item) => item.id !== documentId);
  window.localStorage.setItem(recentStorageKey(tenantId), JSON.stringify(next));
}
