import type { KnowledgeMarketSubscriptionResult } from './knowledge-market-subscription-result';

export interface MarketSubscriptionsCreateResponse201 {
  code: 0;
  data: unknown & KnowledgeMarketSubscriptionResult;
  /** Server-owned request correlation id. */
  traceId: string;
}
