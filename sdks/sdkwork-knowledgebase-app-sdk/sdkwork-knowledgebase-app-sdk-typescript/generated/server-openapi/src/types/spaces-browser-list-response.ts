import type { KnowledgeBrowserNode } from './knowledge-browser-node';

export interface SpacesBrowserListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
