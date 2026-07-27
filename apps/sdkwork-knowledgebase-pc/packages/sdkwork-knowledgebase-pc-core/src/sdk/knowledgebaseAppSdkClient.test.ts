import { describe, expect, it } from 'vitest';

import { createKnowledgebaseAppClient } from '@sdkwork/knowledgebase-app-sdk';

describe('createKnowledgebaseAppClient', () => {
  it('exposes the nested knowledge market resource surface', () => {
    const client = createKnowledgebaseAppClient({
      authMode: 'dual-token',
      baseUrl: 'https://knowledgebase.example.test',
    });

    expect(client.knowledge.market.listings.list).toBeTypeOf('function');
    expect(client.knowledge.market.subscriptions.create).toBeTypeOf('function');
    expect(client.knowledge.market.subscriptions.delete).toBeTypeOf('function');
  });
});
