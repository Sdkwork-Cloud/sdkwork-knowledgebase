import type { PositiveInt64String } from './positive-int64-string';
import type { WikiPublicPageMetadata } from './wiki-public-page-metadata';

export interface WikiRouteResolution {
  disposition: 'PAGE' | 'REDIRECT';
  page?: WikiPublicPageMetadata;
  contentHandle?: string;
  requestedRoute?: string;
  canonicalRoute?: string;
  status?: 301 | 302 | 307 | 308;
  pagePublicVersion?: PositiveInt64String;
}
