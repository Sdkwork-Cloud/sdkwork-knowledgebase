import { afterEach, describe, expect, it } from 'vitest';
import {
  configureKnowledgebaseAppSdk,
  KnowledgebaseErrorCodes,
  setKnowledgebaseApiEnabled,
  setKnowledgebaseNetworkOnline,
} from 'sdkwork-knowledgebase-pc-core';

import { subscribeMarketListing } from './knowledgeMarketService';
import { runImageGenerationTask, runSpeechToTextTask } from './knowledgeMediaTaskService';
import { publishKnowledgeSite } from './knowledgeSitePublicationService';
import { WechatService } from './wechat';

function configureFakeKnowledgeClient(knowledge: unknown): void {
  configureKnowledgebaseAppSdk({
    client: { knowledge } as never,
    setTokenManager() {
      // Test fake; token propagation is covered by pc-core SDK bootstrap tests.
    },
  });
  setKnowledgebaseApiEnabled(true);
  setKnowledgebaseNetworkOnline(true);
}

afterEach(() => {
  setKnowledgebaseApiEnabled(false);
});

describe('knowledge command result services', () => {
  it('treats accepted market subscription results as successful commands', async () => {
    configureFakeKnowledgeClient({
      market: {
        subscriptions: {
          create: async () => ({ accepted: true, status: 'completed' }),
        },
      },
    });

    await expect(subscribeMarketListing('42')).resolves.toBe(true);
  });

  it('publishes a version-fenced immutable site release', async () => {
    configureFakeKnowledgeClient({
      sites: {
        retrieve: async () => ({
          id: '7001', spaceId: '123', title: 'Docs', visibility: 'public',
          themeId: 'default', publishMode: 'manual', version: '3',
        }),
        update: async () => ({
          id: '7001', spaceId: '123', title: 'Docs', visibility: 'public',
          themeId: 'default', publishMode: 'manual', version: '4',
        }),
      },
      siteReleases: {
        create: async () => ({
          site: { id: '7001' },
          release: { id: '9001' },
          publicUrl: 'https://123.kb.sdkwork.com/',
        }),
      },
    });

    await expect(publishKnowledgeSite('123', { siteName: 'Docs' })).resolves.toEqual({
      siteId: '7001',
      releaseId: '9001',
      url: 'https://123.kb.sdkwork.com/',
    });
  });

  it('rejects site publication results without a safe public URL', async () => {
    configureFakeKnowledgeClient({
      sites: {
        retrieve: async () => ({
          id: '7001', spaceId: '123', title: 'Docs', visibility: 'public',
          themeId: 'default', publishMode: 'manual', version: '3',
        }),
        update: async () => ({
          id: '7001', spaceId: '123', title: 'Docs', visibility: 'public',
          themeId: 'default', publishMode: 'manual', version: '4',
        }),
      },
      siteReleases: {
        create: async () => ({
          site: { id: '7001' }, release: { id: '9001' }, publicUrl: '',
        }),
      },
    });

    await expect(publishKnowledgeSite('123', { siteName: 'Docs' })).rejects.toMatchObject({
      code: KnowledgebaseErrorCodes.OPERATION_FAILED,
    });
  });

  it('uses accepted media task results and validates task-specific payload fields', async () => {
    configureFakeKnowledgeClient({
      mediaTasks: {
        create: async (body: { taskType: string }) =>
          body.taskType === 'speech_to_text'
            ? {
                accepted: true,
                status: 'completed',
                text: 'transcript',
                suggestions: [],
                similars: [],
              }
            : {
                accepted: true,
                status: 'completed',
                url: 'https://kb.example.test/image.png',
                resolution: '1024x1024',
                suggestions: ['try brighter lighting'],
                similars: [],
              },
      },
    });

    await expect(runSpeechToTextTask('https://kb.example.test/audio.mp3', { spaceId: '123' }))
      .resolves.toBe('transcript');
    await expect(runImageGenerationTask('cover image', '1:1', 'default', { spaceId: '123' }))
      .resolves.toEqual({
        url: 'https://kb.example.test/image.png',
        resolution: '1024x1024',
        suggestions: ['try brighter lighting'],
        similars: [],
      });
  });

  it('returns accepted WeChat command results without legacy message fields', async () => {
    configureFakeKnowledgeClient({
      wechat: {
        articles: {
          publish: async () => ({ accepted: true, status: 'completed' }),
        },
      },
    });

    await expect(
      WechatService.publishArticles(
        ['official-1'],
        [{ id: 'article-1', title: 'Title', author: 'Author', content: 'Body' }],
      ),
    ).resolves.toEqual({ accepted: true, status: 'completed' });
  });
});
