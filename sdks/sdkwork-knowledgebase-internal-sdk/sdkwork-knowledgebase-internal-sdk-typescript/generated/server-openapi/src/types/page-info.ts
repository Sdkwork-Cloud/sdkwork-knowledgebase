export interface PageInfo {
  mode: 'cursor';
  pageSize: number;
  nextCursor?: string;
  hasMore: boolean;
}
