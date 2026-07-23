import type { NonNegativeInt64String } from './non-negative-int64-string';
import type { PositiveInt64String } from './positive-int64-string';
import type { WikiFileKind } from './wiki-file-kind';

export interface WikiPublicPageMetadata {
  projectionUuid: string;
  canonicalRoute: string;
  fileKind: WikiFileKind;
  mediaType: string;
  sizeBytes: NonNegativeInt64String;
  contentSha256: string;
  title?: string;
  description?: string;
  locale?: string;
  navOrder?: number;
  pagePublicVersion: PositiveInt64String;
  publicUpdatedAt: string;
}
