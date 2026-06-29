import type { OkfLogEntry } from './okf-log-entry';

export interface OkfLogEntriesCreateResponse201 {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
