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
  page: SdkWorkListPage<T>,
): NormalizedSdkWorkListPage<T> {
  const nextCursor = page.pageInfo.nextCursor ?? null;
  const hasMore = page.pageInfo.hasMore ?? Boolean(nextCursor);
  return {
    items: page.items,
    nextCursor,
    hasMore,
  };
}
