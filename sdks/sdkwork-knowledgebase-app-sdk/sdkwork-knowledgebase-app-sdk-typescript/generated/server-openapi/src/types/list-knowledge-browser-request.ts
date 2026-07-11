import type { KnowledgeBrowserView } from './knowledge-browser-view';

/** Browser list request. parentId is a Drive folder id within the selected view root; omit parentId to resolve the view root. */
export interface ListKnowledgeBrowserRequest {
  spaceId: string;
  /** Drive folder id within the selected browser view root. For OKF files view, it must be under sources/raw; for okf_bundle, under okf; for outputs, under output. */
  parentId?: string | null;
  /** files shows original files, okf_bundle shows generated OKF bundle content, outputs shows generated outputs. */
  view: KnowledgeBrowserView;
  cursor?: string | null;
  /** JSON request body page size. HTTP GET uses page_size on the wire. */
  pageSize?: number | null;
}
