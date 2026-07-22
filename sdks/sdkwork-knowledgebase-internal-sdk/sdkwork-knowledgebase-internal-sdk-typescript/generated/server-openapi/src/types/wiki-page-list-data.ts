import type { PageInfo } from './page-info';
import type { WikiPage } from './wiki-page';

export interface WikiPageListData {
  items: WikiPage[];
  pageInfo: PageInfo;
}
