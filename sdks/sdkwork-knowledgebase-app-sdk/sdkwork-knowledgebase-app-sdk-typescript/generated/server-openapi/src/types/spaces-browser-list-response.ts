import type { KnowledgeBrowserListData } from './knowledge-browser-list-data';

export interface SpacesBrowserListResponse {
  code: 0;
  data: unknown & KnowledgeBrowserListData;
  /** Server-owned request correlation id. */
  traceId: string;
}
