import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

const DEFAULT_PROBE_PATHS = ['/livez', '/readyz'];
const METRICS_PROBE_PATHS = ['/metrics'];

export function resolveSmokeBaseUrl() {
  return (
    process.env.SDKWORK_KNOWLEDGEBASE_SMOKE_BASE_URL?.trim()
    || process.env.SMOKE_BASE_URL?.trim()
    || ''
  );
}

export function resolveSplitServiceSmokeUrls() {
  return {
    app: process.env.SDKWORK_KNOWLEDGEBASE_SMOKE_APP_URL?.trim() || '',
    backend: process.env.SDKWORK_KNOWLEDGEBASE_SMOKE_BACKEND_URL?.trim() || '',
    open: process.env.SDKWORK_KNOWLEDGEBASE_SMOKE_OPEN_URL?.trim() || '',
    worker: process.env.SDKWORK_KNOWLEDGEBASE_SMOKE_WORKER_URL?.trim() || '',
  };
}

export function resolveSmokeMetricsUrls() {
  const rawValue = (
    process.env.SDKWORK_KNOWLEDGEBASE_SMOKE_METRICS_URLS?.trim()
    || process.env.SDKWORK_KNOWLEDGEBASE_SMOKE_METRICS_URL?.trim()
    || ''
  );
  if (!rawValue) {
    return [];
  }
  return rawValue.split(',').map((value) => value.trim()).filter(Boolean);
}

export async function probeKnowledgebaseHttpSurface(baseUrl, paths = DEFAULT_PROBE_PATHS) {
  const normalizedBase = baseUrl.replace(/\/+$/, '');
  const results = [];

  for (const path of paths) {
    const response = await fetch(`${normalizedBase}${path}`, {
      method: 'GET',
      headers: { accept: '*/*' },
    });
    results.push({ path, status: response.status });
    if (path === '/metrics') {
      const body = await response.text();
      assert.match(body, /knowledge_api_requests_total/);
      assert.match(body, /knowledgebase_health_status/);
      assert.match(body, /knowledge_api_auth_failures_total/);
    }
  }

  return results;
}

describe('knowledgebase API smoke helpers', () => {
  it('defines default probe paths for public production health surfaces', () => {
    assert.deepEqual(DEFAULT_PROBE_PATHS, ['/livez', '/readyz']);
    assert.deepEqual(METRICS_PROBE_PATHS, ['/metrics']);
  });

  it('keeps default public health smoke off the metrics endpoint', async () => {
    const requestedPaths = [];
    const originalFetch = globalThis.fetch;
    globalThis.fetch = async (url) => {
      requestedPaths.push(new URL(url).pathname);
      return new Response(
        'knowledge_api_requests_total 1\nknowledgebase_health_status 1\nknowledge_api_auth_failures_total 0\n',
        { status: 200 },
      );
    };

    try {
      await probeKnowledgebaseHttpSurface('https://knowledgebase.example.com');
    } finally {
      globalThis.fetch = originalFetch;
    }

    assert.deepEqual(requestedPaths, ['/livez', '/readyz']);
  });

  it('resolves explicit internal metrics smoke URLs without enabling public metrics', () => {
    const originalMetricsUrl = process.env.SDKWORK_KNOWLEDGEBASE_SMOKE_METRICS_URL;
    const originalMetricsUrls = process.env.SDKWORK_KNOWLEDGEBASE_SMOKE_METRICS_URLS;
    process.env.SDKWORK_KNOWLEDGEBASE_SMOKE_METRICS_URL = 'http://sdkwork-knowledgebase-app-api';
    process.env.SDKWORK_KNOWLEDGEBASE_SMOKE_METRICS_URLS = ' http://sdkwork-knowledgebase-backend-api , http://sdkwork-knowledgebase-worker:18085 ';

    try {
      assert.deepEqual(resolveSmokeMetricsUrls(), [
        'http://sdkwork-knowledgebase-backend-api',
        'http://sdkwork-knowledgebase-worker:18085',
      ]);
      assert.deepEqual(DEFAULT_PROBE_PATHS, ['/livez', '/readyz']);
    } finally {
      if (originalMetricsUrl === undefined) {
        delete process.env.SDKWORK_KNOWLEDGEBASE_SMOKE_METRICS_URL;
      } else {
        process.env.SDKWORK_KNOWLEDGEBASE_SMOKE_METRICS_URL = originalMetricsUrl;
      }
      if (originalMetricsUrls === undefined) {
        delete process.env.SDKWORK_KNOWLEDGEBASE_SMOKE_METRICS_URLS;
      } else {
        process.env.SDKWORK_KNOWLEDGEBASE_SMOKE_METRICS_URLS = originalMetricsUrls;
      }
    }
  });

  it('probes livez and readyz when a smoke base URL is configured', async (t) => {
    const baseUrl = resolveSmokeBaseUrl();
    if (!baseUrl) {
      t.skip('Set SDKWORK_KNOWLEDGEBASE_SMOKE_BASE_URL to run live API smoke checks.');
      return;
    }

    const results = await probeKnowledgebaseHttpSurface(baseUrl);
    for (const result of results) {
      assert.ok(
        result.status >= 200 && result.status < 400,
        `${result.path} returned ${result.status}`,
      );
    }
  });

  it('rejects unauthenticated backend-api access when smoke base URL is configured', async (t) => {
    const baseUrl = resolveSmokeBaseUrl();
    if (!baseUrl) {
      t.skip('Set SDKWORK_KNOWLEDGEBASE_SMOKE_BASE_URL to run live API smoke checks.');
      return;
    }

    const normalizedBase = baseUrl.replace(/\/+$/, '');
    const response = await fetch(`${normalizedBase}/backend/v3/api/knowledge/sources`, {
      method: 'GET',
      headers: { accept: 'application/json' },
    });
    assert.equal(response.status, 401);
  });

  it('probes split-service health surfaces when per-service URLs are configured', async (t) => {
    const urls = resolveSplitServiceSmokeUrls();
    const configured = Object.entries(urls).filter(([, value]) => Boolean(value));
    if (configured.length === 0) {
      t.skip(
        'Set SDKWORK_KNOWLEDGEBASE_SMOKE_APP_URL, _BACKEND_URL, _OPEN_URL, and/or _WORKER_URL for split-service smoke.',
      );
      return;
    }

    for (const [service, baseUrl] of configured) {
      const results = await probeKnowledgebaseHttpSurface(baseUrl);
      for (const result of results) {
        assert.ok(
          result.status >= 200 && result.status < 400,
          `${service}${result.path} returned ${result.status}`,
        );
      }
    }
  });

  it('probes metrics only when an internal metrics smoke URL is explicitly configured', async (t) => {
    const metricsUrls = resolveSmokeMetricsUrls();
    if (metricsUrls.length === 0) {
      t.skip('Set SDKWORK_KNOWLEDGEBASE_SMOKE_METRICS_URL or _METRICS_URLS to run internal metrics smoke checks.');
      return;
    }

    for (const metricsUrl of metricsUrls) {
      const results = await probeKnowledgebaseHttpSurface(metricsUrl, METRICS_PROBE_PATHS);
      assert.deepEqual(results.map((result) => result.path), METRICS_PROBE_PATHS);
      for (const result of results) {
        assert.ok(
          result.status >= 200 && result.status < 400,
          `${metricsUrl}${result.path} returned ${result.status}`,
        );
      }
    }
  });
});
