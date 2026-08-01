import { describe, expect, it } from 'vitest';
import {
  formatFileSizeLimit,
  isFileWithinSizeLimit,
  MAX_DOMAIN_VERIFICATION_TEXT_BYTES,
} from './fileInputLimits';

describe('file input limits', () => {
  it('accepts a file exactly at the configured limit', () => {
    expect(isFileWithinSizeLimit(
      { size: MAX_DOMAIN_VERIFICATION_TEXT_BYTES },
      MAX_DOMAIN_VERIFICATION_TEXT_BYTES,
    )).toBe(true);
  });

  it('rejects a file before FileReader can load content over the limit', () => {
    expect(isFileWithinSizeLimit(
      { size: MAX_DOMAIN_VERIFICATION_TEXT_BYTES + 1 },
      MAX_DOMAIN_VERIFICATION_TEXT_BYTES,
    )).toBe(false);
  });

  it('rejects invalid size metadata and invalid limits', () => {
    expect(isFileWithinSizeLimit({ size: Number.NaN }, 1)).toBe(false);
    expect(isFileWithinSizeLimit({ size: -1 }, 1)).toBe(false);
    expect(isFileWithinSizeLimit({ size: 0 }, 0)).toBe(false);
  });

  it('formats limits through sdkwork-utils', () => {
    expect(formatFileSizeLimit(MAX_DOMAIN_VERIFICATION_TEXT_BYTES)).toBe('64 KB');
  });
});
