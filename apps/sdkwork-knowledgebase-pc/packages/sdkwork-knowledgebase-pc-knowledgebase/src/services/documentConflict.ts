import {
  KnowledgebaseErrorCodes,
  throwKnowledgebaseError,
} from 'sdkwork-knowledgebase-pc-core';

/**
 * Conflict guard for document saves: the save chain throws
 * `DOCUMENT_CONFLICT` when the server `updatedAt` no longer matches the base
 * version this client opened/saved. Components can use this predicate to show
 * a merge prompt instead of a generic failure toast.
 */
export function isDocumentConflictError(error: unknown): boolean {
  if (error && typeof error === 'object') {
    const code = (error as { code?: unknown }).code;
    if (typeof code === 'string') {
      return code === KnowledgebaseErrorCodes.DOCUMENT_CONFLICT;
    }
    const cause = (error as { cause?: unknown }).cause;
    if (cause && typeof cause === 'object') {
      const causeCode = (cause as { code?: unknown }).code;
      if (typeof causeCode === 'string') {
        return causeCode === KnowledgebaseErrorCodes.DOCUMENT_CONFLICT;
      }
    }
  }
  return false;
}

/**
 * Throws the canonical conflict error used by the save chain.
 */
export function throwDocumentConflict(): never {
  throwKnowledgebaseError(KnowledgebaseErrorCodes.DOCUMENT_CONFLICT, {
    cause: 'document was updated elsewhere since it was opened',
  });
}
