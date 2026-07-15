import { describe, expect, it } from 'vitest';

import {
  normalizeKnowledgebaseBrowserBasePath,
  toKnowledgebaseViteBasePath,
} from './browserBasePath';
import { createRuntimeConfig } from './runtimeConfig';

describe('Knowledgebase browser base path', () => {
  it('normalizes an explicit non-root path for BrowserRouter', () => {
    expect(normalizeKnowledgebaseBrowserBasePath('/apps/knowledgebase/')).toBe(
      '/apps/knowledgebase',
    );
    expect(toKnowledgebaseViteBasePath('/apps/knowledgebase/')).toBe(
      '/apps/knowledgebase/',
    );
  });

  it('falls back to root for unsafe path-like values', () => {
    for (const value of [
      'knowledgebase',
      '//knowledgebase',
      'https://knowledgebase.example.test',
      '/knowledgebase?ticket=value',
      '/knowledgebase#fragment',
      '/knowledgebase/%2e%2e',
      '/knowledgebase/../admin',
    ]) {
      expect(normalizeKnowledgebaseBrowserBasePath(value)).toBe('/');
    }
  });

  it('exposes the normalized value through typed runtime config', () => {
    const config = createRuntimeConfig({
      VITE_SDKWORK_KNOWLEDGEBASE_BROWSER_BASE_PATH: '/apps/knowledgebase/',
      VITE_SDKWORK_KNOWLEDGEBASE_ENVIRONMENT: 'production',
      VITE_SDKWORK_KNOWLEDGEBASE_RUNTIME_TARGET: 'browser',
      PROD: true,
    });

    expect(config.browserBasePath).toBe('/apps/knowledgebase');
  });
});
