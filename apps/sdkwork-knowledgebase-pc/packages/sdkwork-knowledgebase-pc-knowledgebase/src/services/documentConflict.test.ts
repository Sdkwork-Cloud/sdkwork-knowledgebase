import { describe, expect, it } from 'vitest';
import {
  isDocumentConflictError,
  throwDocumentConflict,
} from './documentConflict';

describe('document conflict guard', () => {
  it('recognizes the canonical conflict error', () => {
    let caught: unknown;
    try {
      throwDocumentConflict();
    } catch (error) {
      caught = error;
    }
    expect(caught).toBeDefined();
    expect(isDocumentConflictError(caught)).toBe(true);
  });

  it('recognizes conflicts wrapped inside a cause chain', () => {
    const wrapped = new Error('outer', {
      cause: { code: 'operation.documentConflict' },
    });
    expect(isDocumentConflictError(wrapped)).toBe(true);
  });

  it('rejects unrelated failures', () => {
    expect(isDocumentConflictError(new Error('network down'))).toBe(false);
    expect(isDocumentConflictError({ code: 'operation.ingestFailed' })).toBe(false);
    expect(isDocumentConflictError(null)).toBe(false);
  });
});
