import type { WikiPublicationResourceData } from './wiki-publication-resource-data';

export interface WikiPublicationResponse {
  code: 0;
  data: unknown & WikiPublicationResourceData;
  traceId: string;
}
