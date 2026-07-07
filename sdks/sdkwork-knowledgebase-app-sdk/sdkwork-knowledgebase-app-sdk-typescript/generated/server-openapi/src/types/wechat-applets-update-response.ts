import type { KnowledgeWechatAppletList } from './knowledge-wechat-applet-list';

export interface WechatAppletsUpdateResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
