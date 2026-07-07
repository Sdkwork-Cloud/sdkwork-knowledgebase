import { describe, expect, it } from 'vitest';

import { KnowledgebaseErrorCodes } from './knowledgebaseErrorCodes';
import { isKnowledgebaseAppError } from './knowledgebaseAppError';
import { parseKnowledgeSpaceId, requireHttpUrl, normalizeKnowledgeSpaceId } from './serviceGuards';

describe('serviceGuards', () => {
  it('parses valid knowledge space ids', () => {
    expect(parseKnowledgeSpaceId('42')).toBe('42');
    expect(parseKnowledgeSpaceId('332473351878475800')).toBe('332473351878475800');
  });

  it('rejects unsafe integer knowledge space ids', () => {
    try {
      normalizeKnowledgeSpaceId(332473351878475800);
      expect.fail('expected unsafe integer rejection');
    } catch (error) {
      expect(isKnowledgebaseAppError(error)).toBe(true);
      if (isKnowledgebaseAppError(error)) {
        expect(error.code).toBe(KnowledgebaseErrorCodes.INVALID_SPACE_ID);
      }
    }
  });

  it('rejects invalid knowledge space ids', () => {
    try {
      parseKnowledgeSpaceId('abc');
      expect.fail('expected invalid space id');
    } catch (error) {
      expect(isKnowledgebaseAppError(error)).toBe(true);
      if (isKnowledgebaseAppError(error)) {
        expect(error.code).toBe(KnowledgebaseErrorCodes.INVALID_SPACE_ID);
      }
    }
  });

  it('requires http or https urls', () => {
    expect(requireHttpUrl('https://example.com/path').href).toContain('example.com');
    try {
      requireHttpUrl('ftp://example.com');
      expect.fail('expected invalid scheme');
    } catch (error) {
      expect(isKnowledgebaseAppError(error)).toBe(true);
      if (isKnowledgebaseAppError(error)) {
        expect(error.code).toBe(KnowledgebaseErrorCodes.URL_INVALID_SCHEME);
      }
    }
  });
});
