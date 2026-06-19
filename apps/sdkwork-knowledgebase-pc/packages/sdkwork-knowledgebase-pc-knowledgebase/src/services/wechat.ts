import { DocumentMeta } from './document';

export interface OfficialAccount {
  id: string;
  name: string;
  type: 'subscription' | 'service';
  avatar: string;
  description?: string;
  appId: string;
  appSecret: string;
  serverUrl?: string;
  token?: string;
  encodingAesKey?: string;
  encryptMode?: 'plain' | 'compatible' | 'safe';
  domainVerifyFileName?: string;
  domainVerifyFileContent?: string;
  jsSecureDomains?: string[]; // JS接口安全域名 (最多5个)
  webAuthDomains?: string[]; // 网页授权域名 (最多2个)
  businessDomains?: string[]; // 业务域名 (最多3个)
  group?: string;
}

export interface WechatAppletConfig {
  id: string;
  name: string;
  appId: string;
  originalId?: string;
  appSecret?: string;
  path: string;
  avatar: string;
  group?: string;
  description?: string;
  requestDomain?: string[]; // request合法域名
  socketDomain?: string[]; // socket合法域名
  uploadDomain?: string[]; // uploadFile合法域名
  downloadDomain?: string[]; // downloadFile合法域名
  udpDomain?: string[]; // udp合法域名
  tcpDomain?: string[]; // tcp合法域名
  businessDomain?: string[]; // 业务域名 (WebView)
  domainVerifyFileName?: string; // 业务域名校验文件
  domainVerifyFileContent?: string; // 业务域名校验文件内容
  msgToken?: string;
  msgEncodingAESKey?: string;
  msgDataFormat?: 'json' | 'xml';
  msgEncryptMode?: 'plain' | 'compatible' | 'safe';
}

export interface WechatArticle {
  id: string;
  title: string;
  author: string;
  content?: string;
  cover?: string;
  abstract?: string;
  isOriginal?: boolean;
  commentType?: 'everyone' | 'follower' | 'none';
  coverZoom?: number;
  coverOffsetX?: number;
  coverOffsetY?: number;
  coverAspect?: '2.35' | '1:1';
}

const delay = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

// ============ MOCK DATA ============
let mockOfficialAccounts: OfficialAccount[] = [
  {
    id: '1',
    name: 'AI 人工智能基地',
    type: 'subscription',
    avatar: '🤖',
    appId: 'wx57d8123abc456ef0',
    appSecret: '3a8f9d0c2e4b6d8f0a2c4e6b8d0a2c4e',
    serverUrl: 'https://api.ai-base.com/wechat/callback',
    token: 'aibasetoken123456',
    encodingAesKey: '9JbM4zRzKpY8mFqXwT4n6pLmD2s8eVjY1hC3vB5nKxW',
    encryptMode: 'safe',
    domainVerifyFileName: 'MP_verify_ai_base.txt',
    domainVerifyFileContent: 'ai-base-verification-code-12345',
    group: '科技数码'
  },
  {
    id: '2',
    name: '独立开发者周刊',
    type: 'subscription',
    avatar: '💡',
    appId: 'wx92f8713def456ba1',
    appSecret: 'e8f0a2c4e6b8d0a2c4e6b8d0a3a8f9d0',
    serverUrl: 'https://api.indiedev.org/wechat/events',
    token: 'indiedevtoken7890',
    encodingAesKey: '6HdB5zRzKpY8mFqXwT4n6pLmD2s8eVjY1hC3vB5nKxW',
    encryptMode: 'safe',
    domainVerifyFileName: 'MP_verify_indiedev.txt',
    domainVerifyFileContent: 'indie-developer-weekly-code-98765',
    group: '科技数码'
  },
  {
    id: '3',
    name: '极客前线',
    type: 'service',
    avatar: '⚡',
    appId: 'wx31d6248fed192ca5',
    appSecret: 'c2e4b6d8f0a2c4e6b8d0a2c4e6b8d0a2',
    serverUrl: 'https://services.geekfront.cn/wx/msg',
    token: 'geekfronttokensecret99',
    encodingAesKey: '2FfZ8zRzKpY8mFqXwT4n6pLmD2s8eVjY1hC3vB5nKxW',
    encryptMode: 'safe',
    domainVerifyFileName: 'MP_verify_geekfront.txt',
    domainVerifyFileContent: 'geek-frontline-verification-code-5555',
    group: '企业矩阵'
  }
];

let mockApplets: WechatAppletConfig[] = [
  {
    id: 'a1',
    name: '文章助手',
    appId: 'wx1234567890abcdef',
    path: 'pages/index/index',
    avatar: 'AS',
    group: '工具',
    description: '办公辅助'
  },
  {
    id: 'a2',
    name: 'AI绘图',
    appId: 'wxabcdef1234567890',
    path: 'pages/home/main',
    avatar: '🎨',
    group: 'AI工具',
    description: '智能生成配图'
  }
];

export class WechatService {
  /**
   * 取回所有公众号账号信息
   */
  static async getOfficialAccounts(): Promise<OfficialAccount[]> {
    await delay(300);
    return JSON.parse(JSON.stringify(mockOfficialAccounts));
  }

  /**
   * 批量保存/更新公众号配置信息
   */
  static async saveOfficialAccounts(accounts: OfficialAccount[]): Promise<boolean> {
    await delay(500);
    mockOfficialAccounts = JSON.parse(JSON.stringify(accounts));
    return true;
  }

  /**
   * 取回所有小程序配置信息
   */
  static async getApplets(): Promise<WechatAppletConfig[]> {
    await delay(300);
    return JSON.parse(JSON.stringify(mockApplets));
  }

  /**
   * 批量保存/更新小程序配置信息
   */
  static async saveApplets(applets: WechatAppletConfig[]): Promise<boolean> {
    await delay(500);
    mockApplets = JSON.parse(JSON.stringify(applets));
    return true;
  }


  /**
   * 发布/推送微信图文消息
   */
  static async publishArticles(
      selectedAccountIds: string[], 
      articles: WechatArticle[], 
      options?: { 
          sendNotification?: boolean; 
          groupNotification?: boolean; 
          selectedGroupId?: string; 
          scheduleTime?: string | null; 
      }
  ): Promise<{ success: boolean; message: string }> {
    await delay(1500); // Simulate network publish action
    if (!selectedAccountIds.length || !articles.length) {
      throw new Error('No accounts or articles provided.');
    }
    console.log('Publishing to accounts:', selectedAccountIds, 'with options:', options);
    return { success: true, message: 'Articles successfully pushed to WeChat.' };
  }

  /**
   * 发送预览到指定微信
   */
  static async sendPreview(accountId: string, wechatIds: string[], articles: WechatArticle[]): Promise<{ success: boolean; message: string }> {
    await delay(1200); // Simulate network publish action
    if (!accountId || !wechatIds.length || !articles.length) {
      throw new Error('Invalid arguments for sendPreview.');
    }
    return { success: true, message: 'Preview successfully sent to WeChat.' };
  }

  /**
   * 智能排版功能
   */
  static async autoFormatContent(content: string, type: string): Promise<string> {
    await delay(1200); // Simulate processing
    // Simple mock formatting: just wrapping with a stylish div
    return `<div style="font-family: inherit; color: #222; text-align: justify; line-height: 1.8; padding: 20px 10px;">${content}</div>`;
  }
}
