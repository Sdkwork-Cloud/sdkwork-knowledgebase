import type { OkfQueryResult } from './okf-query-result';

export interface OkfQueriesFileAnswerResponse {
  code: 0;
  data: unknown & { item: OkfQueryResult; };
  /** Server-owned request correlation id. */
  traceId: string;
}
