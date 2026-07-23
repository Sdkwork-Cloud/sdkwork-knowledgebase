import type { WikiPublicPageListData } from './wiki-public-page-list-data';

export interface WikiPublicPageListResponse {
  code: 0;
  data: unknown & WikiPublicPageListData;
  traceId: string;
}
