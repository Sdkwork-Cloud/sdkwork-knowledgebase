import type { KnowledgeWechatOfficialAccountList } from './knowledge-wechat-official-account-list';

export interface WechatOfficialAccountsUpdateResponse {
  code: 0;
  data: unknown & { item: KnowledgeWechatOfficialAccountList; };
  /** Server-owned request correlation id. */
  traceId: string;
}
