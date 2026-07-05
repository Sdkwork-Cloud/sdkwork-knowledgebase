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

/** In-memory demo store when API is unavailable and preview fallback is enabled. */
let demoOfficialAccounts: OfficialAccount[] = [];

/** In-memory demo store when API is unavailable and preview fallback is enabled. */
let demoApplets: WechatAppletConfig[] = [];

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
    return JSON.parse(JSON.stringify(demoOfficialAccounts));
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
    demoOfficialAccounts = JSON.parse(JSON.stringify(accounts));
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
    return JSON.parse(JSON.stringify(demoApplets));
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
    demoApplets = JSON.parse(JSON.stringify(applets));
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
