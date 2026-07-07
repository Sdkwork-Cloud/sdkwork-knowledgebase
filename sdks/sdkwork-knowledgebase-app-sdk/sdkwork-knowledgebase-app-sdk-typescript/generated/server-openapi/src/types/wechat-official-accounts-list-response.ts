import type { KnowledgeWechatOfficialAccountList } from './knowledge-wechat-official-account-list';

export interface WechatOfficialAccountsListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
