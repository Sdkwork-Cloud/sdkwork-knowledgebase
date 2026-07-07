import type { KnowledgeMarketCatalogItem } from './knowledge-market-catalog-item';
import type { PageInfo } from './page-info';

export interface MarketListingsListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
