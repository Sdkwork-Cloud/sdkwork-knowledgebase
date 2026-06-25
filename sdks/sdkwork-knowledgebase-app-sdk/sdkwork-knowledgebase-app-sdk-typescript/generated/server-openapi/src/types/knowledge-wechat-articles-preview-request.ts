import type { KnowledgeWechatArticle } from './knowledge-wechat-article';

export interface KnowledgeWechatArticlesPreviewRequest {
  accountId: string;
  wechatIds: string[];
  articles: KnowledgeWechatArticle[];
}
