import type { KnowledgeWechatArticle } from './knowledge-wechat-article';

export interface KnowledgeWechatArticlesPublishRequest {
  accountIds: string[];
  articles: KnowledgeWechatArticle[];
  sendNotification?: boolean;
  groupNotification?: boolean;
  selectedGroupId?: string;
  scheduleTime?: string;
}
