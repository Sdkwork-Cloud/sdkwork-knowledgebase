import type { KnowledgeWechatFanTagList } from './knowledge-wechat-fan-tag-list';

export interface WechatOfficialAccountsFanTagsListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
