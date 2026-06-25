import type { KnowledgebaseErrorCode } from './knowledgebaseErrorCodes';

export class KnowledgebaseAppError extends Error {
  readonly code: KnowledgebaseErrorCode | string;
  readonly traceId?: string;
  readonly httpStatus?: number;

  constructor(
    code: KnowledgebaseErrorCode | string,
    options?: {
      cause?: unknown;
      traceId?: string;
      httpStatus?: number;
    },
  ) {
    super(code);
    this.name = 'KnowledgebaseAppError';
    this.code = code;
    this.traceId = options?.traceId;
    this.httpStatus = options?.httpStatus;
    if (options?.cause !== undefined) {
      (this as Error & { cause?: unknown }).cause = options.cause;
    }
  }
}

export function isKnowledgebaseAppError(error: unknown): error is KnowledgebaseAppError {
  return error instanceof KnowledgebaseAppError;
}

export function throwKnowledgebaseError(
  code: KnowledgebaseErrorCode | string,
  options?: {
    cause?: unknown;
    traceId?: string;
    httpStatus?: number;
  },
): never {
  throw new KnowledgebaseAppError(code, options);
}
