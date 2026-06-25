import { resolveUserFacingErrorMessage, type ErrorTranslateFn } from 'sdkwork-knowledgebase-pc-core';

import { toast } from './toast-manager';

export function toastKnowledgebaseError(
  error: unknown,
  translate: ErrorTranslateFn,
): void {
  toast.error(resolveUserFacingErrorMessage(error, translate));
}
