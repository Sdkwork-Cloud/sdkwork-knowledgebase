import type { KnowledgeBrowserNode } from './knowledge-browser-node';
import type { KnowledgeBrowserView } from './knowledge-browser-view';

export interface KnowledgeBrowserPage {
  spaceId: string;
  driveSpaceId: string;
  parentId?: string | null;
  view: KnowledgeBrowserView;
  pageSize: number;
  items: KnowledgeBrowserNode[];
  nextCursor?: string | null;
}
