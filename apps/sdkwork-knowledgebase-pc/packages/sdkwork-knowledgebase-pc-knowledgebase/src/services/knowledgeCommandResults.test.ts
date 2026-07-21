import { afterEach, describe, expect, it } from 'vitest';
import {
  configureKnowledgebaseAppSdk,
  setKnowledgebaseApiEnabled,
  setKnowledgebaseNetworkOnline,
} from 'sdkwork-knowledgebase-pc-core';

import { subscribeMarketListing } from './knowledgeMarketService';
import { runImageGenerationTask, runSpeechToTextTask } from './knowledgeMediaTaskService';
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
