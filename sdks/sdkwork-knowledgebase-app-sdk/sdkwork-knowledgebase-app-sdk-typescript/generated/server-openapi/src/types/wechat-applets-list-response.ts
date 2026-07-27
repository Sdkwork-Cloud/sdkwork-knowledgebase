import type { KnowledgeWechatAppletList } from './knowledge-wechat-applet-list';

export interface WechatAppletsListResponse {
  code: 0;
  data: unknown & { item: KnowledgeWechatAppletList; };
  /** Server-owned request correlation id. */
  traceId: string;
}
