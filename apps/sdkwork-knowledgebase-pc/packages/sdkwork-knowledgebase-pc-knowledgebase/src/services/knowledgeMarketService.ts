import {
  KnowledgebaseErrorCodes,
  requireKnowledgebaseAppSdkHttpClient,
  requirePositiveNumber,
} from 'sdkwork-knowledgebase-pc-core';

import type { MarketKnowledgeBase } from './document';

function parseListingId(id: string): number {
  return requirePositiveNumber(Number(id), KnowledgebaseErrorCodes.INVALID_MARKET_LISTING);
}

function mapCatalogItem(item: {
  id: string;
  title: string;
  icon: string;
  description: string;
  author: string;
  tags: string[];
  subscribersCount: number;
  documentsCount: number;
  provider: string;
  modelName: string;
  isSubscribed: boolean;
}): MarketKnowledgeBase {
  return {
    id: item.id,
    title: item.title,
    icon: item.icon,
    description: item.description,
    author: item.author,
    tags: item.tags,
    subscribersCount: item.subscribersCount,
    documentsCount: item.documentsCount,
    provider: item.provider,
    modelName: item.modelName,
    isSubscribed: item.isSubscribed,
  };
}

export async function listMarketKnowledgeBases(): Promise<MarketKnowledgeBase[]> {
  const client = requireKnowledgebaseAppSdkHttpClient();
  const catalog = await client.knowledge.market.listings.list();
  return catalog.items.map(mapCatalogItem);
}

export async function subscribeMarketListing(id: string): Promise<boolean> {
  const client = requireKnowledgebaseAppSdkHttpClient();
  const result = await client.knowledge.market.subscriptions.create({
    listingId: parseListingId(id),
  });
  return result.success;
}

export async function unsubscribeMarketListing(id: string): Promise<boolean> {
  const client = requireKnowledgebaseAppSdkHttpClient();
  const result = await client.knowledge.market.subscriptions.delete(parseListingId(id));
  return result.success;
}
