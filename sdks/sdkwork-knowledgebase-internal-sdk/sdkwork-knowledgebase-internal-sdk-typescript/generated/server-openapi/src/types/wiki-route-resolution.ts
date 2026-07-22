import type { PositiveInt64String } from './positive-int64-string';
import type { WikiPage } from './wiki-page';

export interface WikiRouteResolution {
  disposition: 'PAGE' | 'REDIRECT';
  page?: WikiPage;
  contentHandle?: string;
  requestedRoute?: string;
  canonicalRoute?: string;
  status?: 301 | 302 | 307 | 308;
  pagePublicVersion?: PositiveInt64String;
}
