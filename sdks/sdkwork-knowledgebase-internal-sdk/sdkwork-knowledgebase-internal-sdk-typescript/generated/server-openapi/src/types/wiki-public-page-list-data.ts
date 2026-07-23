import type { PageInfo } from './page-info';
import type { WikiPublicPageMetadata } from './wiki-public-page-metadata';

export interface WikiPublicPageListData {
  items: WikiPublicPageMetadata[];
  pageInfo: PageInfo;
}
