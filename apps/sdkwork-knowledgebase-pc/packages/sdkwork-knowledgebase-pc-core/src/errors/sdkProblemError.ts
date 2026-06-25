export interface SdkProblemDetails {
  type?: string;
  title?: string;
  status?: number;
  detail?: string | null;
  instance?: string | null;
  code?: string | null;
  traceId?: string | null;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}

function normalizeProblemDetails(value: unknown): SdkProblemDetails | null {
  if (!isRecord(value)) {
    return null;
  }

  const status = typeof value.status === 'number' ? value.status : undefined;
  const title = typeof value.title === 'string' ? value.title : undefined;
  const detail = typeof value.detail === 'string' ? value.detail : value.detail === null ? null : undefined;
  const type = typeof value.type === 'string' ? value.type : undefined;
  const code = typeof value.code === 'string' ? value.code : value.code === null ? null : undefined;
  const traceId =
    typeof value.traceId === 'string'
      ? value.traceId
      : typeof value.trace_id === 'string'
        ? value.trace_id
        : value.traceId === null
          ? null
          : undefined;

  if (status === undefined && title === undefined && detail === undefined && code === undefined) {
    return null;
  }

  return { type, title, status, detail, instance: null, code, traceId };
}

function extractProblemFromRecord(record: Record<string, unknown>): SdkProblemDetails | null {
  const direct = normalizeProblemDetails(record);
  if (direct) {
    return direct;
  }

  for (const key of ['problem', 'body', 'data', 'error']) {
    const nested = normalizeProblemDetails(record[key]);
    if (nested) {
      return nested;
    }
  }

  const response = record.response;
  if (isRecord(response)) {
    const fromResponse = normalizeProblemDetails(response.data ?? response.body);
    if (fromResponse) {
      return {
        ...fromResponse,
        status: fromResponse.status ?? (typeof response.status === 'number' ? response.status : undefined),
      };
    }
    if (typeof response.status === 'number') {
      return { status: response.status };
    }
  }

  return null;
}

export function parseSdkProblemDetails(error: unknown): SdkProblemDetails | null {
  if (!error) {
    return null;
  }

  if (isRecord(error)) {
    const extracted = extractProblemFromRecord(error);
    if (extracted) {
      return extracted;
    }
  }

  if (error instanceof Error) {
    const withCause = (error as Error & { cause?: unknown }).cause;
    if (withCause) {
      const fromCause = parseSdkProblemDetails(withCause);
      if (fromCause) {
        return fromCause;
      }
    }
  }

  return null;
}
