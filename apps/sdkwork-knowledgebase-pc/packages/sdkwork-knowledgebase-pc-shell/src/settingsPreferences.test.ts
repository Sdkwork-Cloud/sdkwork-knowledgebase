import { describe, expect, it } from 'vitest';
import {
  matchesSettingsToken,
  sanitizeSettingsTabId,
  sanitizeStartupModule,
} from './settingsPreferences';

describe('settingsPreferences', () => {
  it('matchesSettingsToken matches partial query and keyword', () => {
    expect(matchesSettingsToken('theme', 'dark theme')).toBe(true);
    expect(matchesSettingsToken('xyz', 'theme')).toBe(false);
  });

  it('sanitizeSettingsTabId falls back for invalid values', () => {
    expect(sanitizeSettingsTabId('appearance')).toBe('appearance');
    expect(sanitizeSettingsTabId('invalid', 'general')).toBe('general');
  });

  it('sanitizeStartupModule accepts kb and market only', () => {
    expect(sanitizeStartupModule('market')).toBe('market');
    expect(sanitizeStartupModule('kb')).toBe('kb');
    expect(sanitizeStartupModule('other', 'kb')).toBe('kb');
  });
});
