import { describe, expect, it } from 'vitest';
import { resolveKnowledgebaseFeatureFlags } from './knowledgebaseFeatureFlags';

describe('resolveKnowledgebaseFeatureFlags', () => {
  it('enables document version history by default in production', () => {
    const flags = resolveKnowledgebaseFeatureFlags('production', {
      VITE_SDKWORK_KNOWLEDGEBASE_ENABLE_PREVIEW_FEATURES: 'false',
    });
    expect(flags.documentVersionHistory).toBe(true);
  });

  it('enables document permissions by default in production', () => {
    const flags = resolveKnowledgebaseFeatureFlags('production', {
      VITE_SDKWORK_KNOWLEDGEBASE_ENABLE_PREVIEW_FEATURES: 'false',
    });
    expect(flags.documentPermissionsModal).toBe(true);
  });

  it('keeps preview-only features disabled in production by default', () => {
    const flags = resolveKnowledgebaseFeatureFlags('production', {
      VITE_SDKWORK_KNOWLEDGEBASE_ENABLE_PREVIEW_FEATURES: 'false',
    });
    expect(flags.notesImport).toBe(false);
    expect(flags.knowledgeMarketCatalog).toBe(false);
  });

  it('allows explicit opt-out for document version history', () => {
    const flags = resolveKnowledgebaseFeatureFlags('production', {
      VITE_SDKWORK_KNOWLEDGEBASE_FEATURE_VERSION_HISTORY: 'false',
    });
    expect(flags.documentVersionHistory).toBe(false);
  });
});
