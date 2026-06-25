import { KnowledgebaseErrorCodes } from './knowledgebaseErrorCodes';
import { isKnowledgebaseAppError } from './knowledgebaseAppError';
import { parseSdkProblemDetails } from './sdkProblemError';

export type ErrorTranslateFn = (
  key: string,
  options?: { defaultValue?: string; [key: string]: unknown },
) => string;

function mapHttpStatusToCode(status?: number): string {
  switch (status) {
    case 401:
      return KnowledgebaseErrorCodes.HTTP_UNAUTHORIZED;
    case 403:
      return KnowledgebaseErrorCodes.HTTP_FORBIDDEN;
    case 404:
      return KnowledgebaseErrorCodes.HTTP_NOT_FOUND;
    case 429:
      return KnowledgebaseErrorCodes.HTTP_RATE_LIMITED;
    default:
      if (status !== undefined && status >= 500) {
        return KnowledgebaseErrorCodes.HTTP_SERVER;
      }
      return KnowledgebaseErrorCodes.GENERIC;
  }
}

export function resolveKnowledgebaseErrorCode(error: unknown): string {
  if (isKnowledgebaseAppError(error)) {
    return error.code;
  }

  const problem = parseSdkProblemDetails(error);
  if (problem?.code) {
    return problem.code;
  }
  if (problem?.status !== undefined) {
    return mapHttpStatusToCode(problem.status);
  }

  if (error instanceof TypeError && /fetch|network/i.test(error.message)) {
    return KnowledgebaseErrorCodes.HTTP_NETWORK;
  }

  return KnowledgebaseErrorCodes.GENERIC;
}

export function resolveUserFacingErrorMessage(
  error: unknown,
  translate: ErrorTranslateFn,
): string {
  const code = resolveKnowledgebaseErrorCode(error);
  const problem = parseSdkProblemDetails(error);
  const traceSuffix =
    import.meta.env?.DEV && problem?.traceId ? ` (${problem.traceId})` : '';

  const message = translate(code, {
    defaultValue: translate('generic', { defaultValue: 'Something went wrong.' }),
  });

  return `${message}${traceSuffix}`;
}
