import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import {
  KnowledgebaseErrorCodes,
  setKnowledgebaseApiEnabled,
} from 'sdkwork-knowledgebase-pc-core';

import { WechatService, type OfficialAccount, type WechatAppletConfig } from './wechat';

const officialAccount: OfficialAccount = {
  id: 'official-1',
  name: 'Official account',
  type: 'service',
  avatar: 'https://media.example.test/official.png',
  appId: 'wx-official-1',
  appSecret: 'secret',
};

const applet: WechatAppletConfig = {
  id: 'applet-1',
  name: 'Applet',
  appId: 'wx-applet-1',
  path: 'pages/index/index',
  avatar: 'https://media.example.test/applet.png',
};

const article = {
  id: 'article-1',
  title: 'Article',
  author: 'Author',
  content: 'Body',
};

const failClosedOperations: Array<[string, () => Promise<unknown>]> = [
  ['list official accounts', () => WechatService.getOfficialAccounts()],
  ['save official accounts', () => WechatService.saveOfficialAccounts([officialAccount])],
  ['list applets', () => WechatService.getApplets()],
  ['save applets', () => WechatService.saveApplets([applet])],
  ['list fan tags', () => WechatService.listFanTags('official-1')],
  ['publish articles', () => WechatService.publishArticles(['official-1'], [article])],
  ['send previews', () => WechatService.sendPreview('official-1', ['wechat-user'], [article])],
  ['auto-format content', () => WechatService.autoFormatContent('Body', 'clean')],
];

beforeEach(() => {
  setKnowledgebaseApiEnabled(false);
  vi.stubEnv('VITE_SDKWORK_KNOWLEDGEBASE_ENABLE_DEMO_MODE', 'true');
  vi.useFakeTimers();
});

afterEach(() => {
  setKnowledgebaseApiEnabled(false);
  vi.unstubAllEnvs();
  vi.useRealTimers();
});

describe('WechatService fail-closed behavior', () => {
  it.each(failClosedOperations)(
    'rejects %s when the composed app SDK is unavailable even in demo mode',
    async (_name, operation) => {
      const assertion = expect(operation()).rejects.toMatchObject({
        code: KnowledgebaseErrorCodes.API_UNAVAILABLE_WECHAT,
      });

      await vi.runAllTimersAsync();
      await assertion;
    },
  );
});
