import type { KnowledgeBrowserView } from './knowledge-browser-view';

export interface ListKnowledgeBrowserRequest {
  spaceId: number;
  parentId?: string | null;
  view: KnowledgeBrowserView;
  cursor?: string | null;
  pageSize?: number | null;
}
