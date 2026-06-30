import type {
  KnowledgeWechatApplet,
  KnowledgeWechatArticle,
  KnowledgeWechatOfficialAccount,
} from 'sdkwork-knowledgebase-pc-core';
import {
  getKnowledgebaseAppSdkClient,
  isKnowledgebaseApiAvailable,
  KnowledgebaseErrorCodes,
  shouldUseKnowledgebaseDemoFallback,
  throwKnowledgebaseError,
} from 'sdkwork-knowledgebase-pc-core';

import { AIService } from './ai';
import {
  hydrateAppletSecrets,
  hydrateOfficialAccountSecrets,
  isDesktopSecureStorageAvailable,
  persistAppletSecrets,
  persistOfficialAccountSecrets,
} from './wechatCredentialStore';

function assertWechatDemoFallbackAllowed(): void {
  if (!shouldUseKnowledgebaseDemoFallback()) {
    throwKnowledgebaseError(KnowledgebaseErrorCodes.API_UNAVAILABLE_WECHAT);
  }
}

function toOfficialAccount(account: KnowledgeWechatOfficialAccount): OfficialAccount {
  return {
    id: account.id,
    name: account.name,
    type: account.type === 'service' ? 'service' : 'subscription',
    avatar: account.avatar,
    description: account.description,
    appId: account.appId,
    appSecret: account.appSecret ?? '',
    serverUrl: account.serverUrl,
    token: account.token,
    encodingAesKey: account.encodingAesKey,
    encryptMode: account.encryptMode as OfficialAccount['encryptMode'],
    domainVerifyFileName: account.domainVerifyFileName,
    domainVerifyFileContent: account.domainVerifyFileContent,
    jsSecureDomains: account.jsSecureDomains,
    webAuthDomains: account.webAuthDomains,
    businessDomains: account.businessDomains,
    group: account.group,
  };
}

function fromOfficialAccount(account: OfficialAccount): KnowledgeWechatOfficialAccount {
  return {
    id: account.id,
    name: account.name,
    type: account.type,
    avatar: account.avatar,
    description: account.description,
    appId: account.appId,
    appSecret: account.appSecret || undefined,
    serverUrl: account.serverUrl,
    token: account.token,
    encodingAesKey: account.encodingAesKey,
    encryptMode: account.encryptMode,
    domainVerifyFileName: account.domainVerifyFileName,
    domainVerifyFileContent: account.domainVerifyFileContent,
    jsSecureDomains: account.jsSecureDomains,
    webAuthDomains: account.webAuthDomains,
    businessDomains: account.businessDomains,
    group: account.group,
  };
}

function toApplet(applet: KnowledgeWechatApplet): WechatAppletConfig {
  return {
    id: applet.id,
    name: applet.name,
    appId: applet.appId,
    originalId: applet.originalId,
    appSecret: applet.appSecret,
    path: applet.path,
    avatar: applet.avatar,
    group: applet.group,
    description: applet.description,
    requestDomain: applet.requestDomain,
    socketDomain: applet.socketDomain,
    uploadDomain: applet.uploadDomain,
    downloadDomain: applet.downloadDomain,
    udpDomain: applet.udpDomain,
    tcpDomain: applet.tcpDomain,
    businessDomain: applet.businessDomain,
    domainVerifyFileName: applet.domainVerifyFileName,
    domainVerifyFileContent: applet.domainVerifyFileContent,
    msgToken: applet.msgToken,
    msgEncodingAESKey: applet.msgEncodingAESKey,
    msgDataFormat: applet.msgDataFormat as WechatAppletConfig['msgDataFormat'],
    msgEncryptMode: applet.msgEncryptMode as WechatAppletConfig['msgEncryptMode'],
  };
}

function fromApplet(applet: WechatAppletConfig): KnowledgeWechatApplet {
  return {
    id: applet.id,
    name: applet.name,
    appId: applet.appId,
    originalId: applet.originalId,
    appSecret: applet.appSecret,
    path: applet.path,
    avatar: applet.avatar,
    group: applet.group,
    description: applet.description,
    requestDomain: applet.requestDomain,
    socketDomain: applet.socketDomain,
    uploadDomain: applet.uploadDomain,
    downloadDomain: applet.downloadDomain,
    udpDomain: applet.udpDomain,
    tcpDomain: applet.tcpDomain,
    businessDomain: applet.businessDomain,
    domainVerifyFileName: applet.domainVerifyFileName,
    domainVerifyFileContent: applet.domainVerifyFileContent,
    msgToken: applet.msgToken,
    msgEncodingAESKey: applet.msgEncodingAESKey,
    msgDataFormat: applet.msgDataFormat,
    msgEncryptMode: applet.msgEncryptMode,
  };
}

function toArticle(article: WechatArticle): KnowledgeWechatArticle {
  return {
    id: article.id,
    title: article.title,
    author: article.author,
    content: article.content,
    cover: article.cover,
    abstract: article.abstract,
  };
}

const MOCK_SECRET_PLACEHOLDER = 'mock-only-not-a-real-secret';

function wechatSdk() {
  return getKnowledgebaseAppSdkClient().client.knowledge.wechat;
}

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
  jsSecureDomains?: string[];
  webAuthDomains?: string[];
  businessDomains?: string[];
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
  requestDomain?: string[];
  socketDomain?: string[];
  uploadDomain?: string[];
  downloadDomain?: string[];
  udpDomain?: string[];
  tcpDomain?: string[];
  businessDomain?: string[];
  domainVerifyFileName?: string;
  domainVerifyFileContent?: string;
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

let mockOfficialAccounts: OfficialAccount[] = [
  {
    id: '1',
    name: 'AI 人工智能基地',
    type: 'subscription',
    avatar: '🤖',
    appId: 'wx57d8123abc456ef0',
    appSecret: MOCK_SECRET_PLACEHOLDER,
    serverUrl: 'https://api.ai-base.com/wechat/callback',
    token: 'mock-token',
    encodingAesKey: 'mock-encoding-aes-key',
    encryptMode: 'safe',
    domainVerifyFileName: 'MP_verify_ai_base.txt',
    domainVerifyFileContent: 'ai-base-verification-code-12345',
    group: '科技数码',
  },
  {
    id: '2',
    name: '独立开发者周刊',
    type: 'subscription',
    avatar: '💡',
    appId: 'wx92f8713def456ba1',
    appSecret: MOCK_SECRET_PLACEHOLDER,
    serverUrl: 'https://api.indiedev.org/wechat/events',
    token: 'mock-token',
    encodingAesKey: 'mock-encoding-aes-key',
    encryptMode: 'safe',
    domainVerifyFileName: 'MP_verify_indiedev.txt',
    domainVerifyFileContent: 'indie-developer-weekly-code-98765',
    group: '科技数码',
  },
  {
    id: '3',
    name: '极客前线',
    type: 'service',
    avatar: '⚡',
    appId: 'wx31d6248fed192ca5',
    appSecret: MOCK_SECRET_PLACEHOLDER,
    serverUrl: 'https://services.geekfront.cn/wx/msg',
    token: 'mock-token',
    encodingAesKey: 'mock-encoding-aes-key',
    encryptMode: 'safe',
    domainVerifyFileName: 'MP_verify_geekfront.txt',
    domainVerifyFileContent: 'geek-frontline-verification-code-5555',
    group: '企业矩阵',
  },
];

let mockApplets: WechatAppletConfig[] = [
  {
    id: 'a1',
    name: '文章助手',
    appId: 'wx1234567890abcdef',
    path: 'pages/index/index',
    avatar: 'AS',
    group: '工具',
    description: '办公辅助',
  },
  {
    id: 'a2',
    name: 'AI绘图',
    appId: 'wxabcdef1234567890',
    path: 'pages/home/main',
    avatar: '🎨',
    group: 'AI工具',
    description: '智能生成配图',
  },
];

export class WechatService {
  static async getOfficialAccounts(): Promise<OfficialAccount[]> {
    if (isKnowledgebaseApiAvailable()) {
      const list = await wechatSdk().officialAccounts.list();
      const accounts = list.accounts.map(toOfficialAccount);
      if (isDesktopSecureStorageAvailable()) {
        return Promise.all(accounts.map((account) => hydrateOfficialAccountSecrets(account)));
      }
      return accounts;
    }
    assertWechatDemoFallbackAllowed();
    await delay(300);
    return JSON.parse(JSON.stringify(mockOfficialAccounts));
  }

  static async saveOfficialAccounts(accounts: OfficialAccount[]): Promise<boolean> {
    if (isKnowledgebaseApiAvailable()) {
      const prepared = isDesktopSecureStorageAvailable()
        ? await Promise.all(
            accounts.map(async (account) => {
              await persistOfficialAccountSecrets(account);
              return hydrateOfficialAccountSecrets(account);
            }),
          )
        : accounts;
      await wechatSdk().officialAccounts.replace({
        accounts: prepared.map(fromOfficialAccount),
      });
      return true;
    }
    assertWechatDemoFallbackAllowed();
    await delay(500);
    mockOfficialAccounts = JSON.parse(JSON.stringify(accounts));
    return true;
  }

  static async getApplets(): Promise<WechatAppletConfig[]> {
    if (isKnowledgebaseApiAvailable()) {
      const list = await wechatSdk().applets.list();
      const applets = list.applets.map(toApplet);
      if (isDesktopSecureStorageAvailable()) {
        return Promise.all(applets.map((applet) => hydrateAppletSecrets(applet)));
      }
      return applets;
    }
    assertWechatDemoFallbackAllowed();
    await delay(300);
    return JSON.parse(JSON.stringify(mockApplets));
  }

  static async saveApplets(applets: WechatAppletConfig[]): Promise<boolean> {
    if (isKnowledgebaseApiAvailable()) {
      const prepared = isDesktopSecureStorageAvailable()
        ? await Promise.all(
            applets.map(async (applet) => {
              await persistAppletSecrets(applet);
              return hydrateAppletSecrets(applet);
            }),
          )
        : applets;
      await wechatSdk().applets.replace({
        applets: prepared.map(fromApplet),
      });
      return true;
    }
    assertWechatDemoFallbackAllowed();
    await delay(500);
    mockApplets = JSON.parse(JSON.stringify(applets));
    return true;
  }

  static async publishArticles(
    selectedAccountIds: string[],
    articles: WechatArticle[],
    options?: {
      sendNotification?: boolean;
      groupNotification?: boolean;
      selectedGroupId?: string;
      scheduleTime?: string | null;
    },
  ): Promise<{ success: boolean; message: string }> {
    if (!selectedAccountIds.length || !articles.length) {
      throwKnowledgebaseError(KnowledgebaseErrorCodes.WECHAT_INVALID_ARGS);
    }
    if (isKnowledgebaseApiAvailable()) {
      return wechatSdk().articles.publish({
        accountIds: selectedAccountIds,
        articles: articles.map(toArticle),
        sendNotification: options?.sendNotification,
        groupNotification: options?.groupNotification,
        selectedGroupId: options?.selectedGroupId,
        scheduleTime: options?.scheduleTime ?? undefined,
      });
    }
    assertWechatDemoFallbackAllowed();
    await delay(1500);
    return { success: true, message: '' };
  }

  static async sendPreview(
    accountId: string,
    wechatIds: string[],
    articles: WechatArticle[],
  ): Promise<{ success: boolean; message: string }> {
    if (!accountId || !wechatIds.length || !articles.length) {
      throwKnowledgebaseError(KnowledgebaseErrorCodes.WECHAT_INVALID_ARGS);
    }
    if (isKnowledgebaseApiAvailable()) {
      return wechatSdk().articles.preview({
        accountId,
        wechatIds,
        articles: articles.map(toArticle),
      });
    }
    assertWechatDemoFallbackAllowed();
    await delay(1200);
    return { success: true, message: '' };
  }

  static async autoFormatContent(content: string, type: string): Promise<string> {
    if (isKnowledgebaseApiAvailable()) {
      const wrapped = `<article data-wechat-format="${type}">${content}</article>`;
      return AIService.streamRewrite(wrapped, () => undefined);
    }

    assertWechatDemoFallbackAllowed();
    await delay(1200);
    return `<div style="font-family: inherit; color: #222; text-align: justify; line-height: 1.8; padding: 20px 10px;">${content}</div>`;
  }
}
