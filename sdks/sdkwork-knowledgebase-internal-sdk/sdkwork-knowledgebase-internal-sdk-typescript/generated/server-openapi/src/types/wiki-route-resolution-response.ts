import type { WikiRouteResolutionResourceData } from './wiki-route-resolution-resource-data';

export interface WikiRouteResolutionResponse {
  code: 0;
  data: unknown & WikiRouteResolutionResourceData;
  traceId: string;
}
