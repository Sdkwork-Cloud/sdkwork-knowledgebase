import {
  KnowledgebaseErrorCodes,
  requireKnowledgebaseAppSdkHttpClient,
  requirePositiveNumber,
} from 'sdkwork-knowledgebase-pc-core';

import type { MarketKnowledgeBase } from './document';

function parseListingId(id: string): string {
  return String(requirePositiveNumber(Number(id), KnowledgebaseErrorCodes.INVALID_MARKET_LISTING));
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

import { normalizeSdkWorkListPage } from './sdkWorkListPage';

export async function listMarketKnowledgeBasesPage(
  cursor?: string | null,
  pageSize = 20,
): Promise<ReturnType<typeof normalizeSdkWorkListPage<MarketKnowledgeBase>>> {
  const client = requireKnowledgebaseAppSdkHttpClient();
  const page = normalizeSdkWorkListPage(
    await client.knowledge.market.listings.list({ cursor, pageSize }),
  );
  return {
    items: page.items.map(mapCatalogItem),
    nextCursor: page.nextCursor,
    hasMore: page.hasMore,
  };
}

export async function listMarketKnowledgeBases(): Promise<MarketKnowledgeBase[]> {
  const firstPage = await listMarketKnowledgeBasesPage();
  return firstPage.items;
}

export async function subscribeMarketListing(id: string): Promise<boolean> {
  const client = requireKnowledgebaseAppSdkHttpClient();
  const result = await client.knowledge.market.subscriptions.create({
    listingId: parseListingId(id),
  });
  return result.accepted === true;
}

export async function unsubscribeMarketListing(id: string): Promise<boolean> {
  const client = requireKnowledgebaseAppSdkHttpClient();
  await client.knowledge.market.subscriptions.delete(parseListingId(id));
  return true;
}
