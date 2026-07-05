import { describe, expect, it } from 'vitest';

import { KnowledgebaseErrorCodes } from './knowledgebaseErrorCodes';
import { KnowledgebaseAppError } from './knowledgebaseAppError';
import { parseSdkProblemDetails } from './sdkProblemError';
import { resolveKnowledgebaseErrorCode, resolveUserFacingErrorMessage } from './resolveUserFacingError';

describe('knowledgebase error utilities', () => {
  it('parses nested SDK problem payloads', () => {
    const problem = parseSdkProblemDetails({
      response: {
        status: 403,
        data: {
          type: 'about:blank',
          title: 'Forbidden',
          status: 403,
          code: 'tenant_id_mismatch',
          traceId: 'trace-1',
        },
      },
    });

    expect(problem?.status).toBe(403);
    expect(problem?.code).toBe('tenant_id_mismatch');
    expect(problem?.traceId).toBe('trace-1');
  });

  it('maps app errors and HTTP statuses to stable codes', () => {
    expect(
      resolveKnowledgebaseErrorCode(
        new KnowledgebaseAppError(KnowledgebaseErrorCodes.API_UNAVAILABLE_AI),
      ),
    ).toBe(KnowledgebaseErrorCodes.API_UNAVAILABLE_AI);

    expect(
      resolveKnowledgebaseErrorCode({
        status: 429,
        title: 'Too Many Requests',
      }),
    ).toBe(KnowledgebaseErrorCodes.HTTP_RATE_LIMITED);
  });

  it('parses numeric SDK problem codes', () => {
    const problem = parseSdkProblemDetails({
      status: 429,
      code: 60002,
      traceId: 'trace-quota',
    });

    expect(problem?.code).toBe('60002');
  });

  it('maps platform quota code to tenant quota error key', () => {
    expect(
      resolveKnowledgebaseErrorCode({
        status: 429,
        code: 60002,
      }),
    ).toBe(KnowledgebaseErrorCodes.TENANT_QUOTA_EXCEEDED);
  });

  it('resolves user-facing messages through translate function', () => {
    const message = resolveUserFacingErrorMessage(
      new KnowledgebaseAppError(KnowledgebaseErrorCodes.API_UNAVAILABLE),
      (key) => (key === 'api.unavailable' ? 'Backend unavailable' : 'Fallback'),
    );
    expect(message).toBe('Backend unavailable');
  });
});
