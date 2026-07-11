import type { KnowledgeBrowserNode } from './knowledge-browser-node';
import type { KnowledgeBrowserView } from './knowledge-browser-view';
import type { PageInfo } from './page-info';

/** Standard browser list response data. It follows SDKWork list semantics with items and pageInfo, and also returns the resolved Drive view context needed by clients. */
export interface KnowledgeBrowserListData {
  spaceId: string;
  /** Drive space id bound to the knowledge space. */
  driveSpaceId: string;
  /** Resolved Drive folder id for the current browser view page. When request parentId is omitted, this is the view root folder id; OKF files view resolves to sources/raw. */
  parentId?: string | null;
  view: KnowledgeBrowserView;
  pageSize: number;
  items: KnowledgeBrowserNode[];
  pageInfo: PageInfo;
}
