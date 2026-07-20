import type { SdkWorkCommandData } from './sdk-work-command-data';

export interface SpacesProviderBindingsTestResponse {
  code: 0;
  data: unknown & SdkWorkCommandData;
  /** Server-owned request correlation id. */
  traceId: string;
}
