export interface SdkWorkListPageInfo {
  nextCursor?: string | null;
  hasMore?: boolean;
}

export interface SdkWorkListPage<T> {
  items: T[];
  pageInfo: SdkWorkListPageInfo;
}

export interface NormalizedSdkWorkListPage<T> {
  items: T[];
  nextCursor: string | null;
  hasMore: boolean;
}

export function normalizeSdkWorkListPage<T>(
  page: unknown,
): NormalizedSdkWorkListPage<T> {
  if (!isRecord(page) || !Array.isArray(page.items)) {
    throw new Error('SDK list response must include an items array.');
  }
  const pageInfo = isRecord(page.pageInfo) ? page.pageInfo : {};
  const nextCursor = typeof pageInfo.nextCursor === 'string' ? pageInfo.nextCursor : null;
  const hasMore = typeof pageInfo.hasMore === 'boolean' ? pageInfo.hasMore : Boolean(nextCursor);
  return {
    items: page.items as T[],
    nextCursor,
    hasMore,
  };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}
