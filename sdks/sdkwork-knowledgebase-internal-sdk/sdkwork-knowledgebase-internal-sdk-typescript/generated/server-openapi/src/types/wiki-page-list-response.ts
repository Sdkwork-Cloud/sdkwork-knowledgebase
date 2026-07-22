import type { WikiPageListData } from './wiki-page-list-data';

export interface WikiPageListResponse {
  code: 0;
  data: unknown & WikiPageListData;
  traceId: string;
}
