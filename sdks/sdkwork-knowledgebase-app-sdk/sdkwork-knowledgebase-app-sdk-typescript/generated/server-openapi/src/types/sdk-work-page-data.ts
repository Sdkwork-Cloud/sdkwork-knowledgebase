import type { PageInfo } from './page-info';

export interface SdkWorkPageData<T> {
  items: T[];
  pageInfo: PageInfo;
}
