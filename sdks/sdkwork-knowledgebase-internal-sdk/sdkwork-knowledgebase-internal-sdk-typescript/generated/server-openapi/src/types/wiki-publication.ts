import type { PositiveInt64String } from './positive-int64-string';

export interface WikiPublication {
  publicationUuid: string;
  title: string;
  description?: string;
  homepageSourcePath: string;
  defaultLocale: string;
  supportedLocales: string[];
  navigationMode: 'DIRECTORY' | 'FRONT_MATTER' | 'CURATED';
  themeKey: string;
  themeVersion: string;
  rendererPolicyVersion: string;
  searchEnabled: boolean;
  robotsPolicy: 'INDEX_FOLLOW' | 'NOINDEX_NOFOLLOW';
  sitemapEnabled: boolean;
  providerGeneration: PositiveInt64String;
  navigationGeneration: PositiveInt64String;
  searchGeneration: PositiveInt64String;
}
