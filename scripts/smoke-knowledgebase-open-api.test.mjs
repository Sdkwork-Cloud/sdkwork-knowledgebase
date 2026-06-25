import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, it } from 'node:test';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, '..');

function readRepoFile(relativePath) {
  return readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

export function resolveSmokeOpenApiBaseUrl() {
  return (
    process.env.SDKWORK_KNOWLEDGEBASE_SMOKE_OPEN_URL?.trim()
    || process.env.SDKWORK_KNOWLEDGEBASE_SMOKE_OPEN_API_BASE_URL?.trim()
    || ''
  );
}

export function resolveSmokeOpenApiKey() {
  return process.env.SDKWORK_KNOWLEDGEBASE_SMOKE_API_KEY?.trim() || '';
}

describe('knowledgebase open API smoke helpers', () => {
  it('probes context pack and retrieval when open API credentials are configured', async (t) => {
    const baseUrl = resolveSmokeOpenApiBaseUrl();
    const apiKey = resolveSmokeOpenApiKey();
    if (!baseUrl || !apiKey) {
      t.skip(
        'Set SDKWORK_KNOWLEDGEBASE_SMOKE_OPEN_URL and SDKWORK_KNOWLEDGEBASE_SMOKE_API_KEY to run live open API smoke checks.',
      );
      return;
    }

    const normalizedBase = baseUrl.replace(/\/+$/, '');
    const headers = {
      accept: 'application/json',
      'content-type': 'application/json',
      'X-API-Key': apiKey,
    };

    const contextPackResponse = await fetch(`${normalizedBase}/knowledge/v3/api/context_packs`, {
      method: 'POST',
      headers,
      body: JSON.stringify({
        query: 'launch readiness smoke probe',
        bindings: [{ spaceId: '1', priority: 0, topK: 3 }],
        includeCitations: true,
        topK: 3,
      }),
    });
    assert.ok(
      contextPackResponse.status >= 200 && contextPackResponse.status < 500,
      `context_packs returned ${contextPackResponse.status}`,
    );

    const retrievalResponse = await fetch(`${normalizedBase}/knowledge/v3/api/retrievals`, {
      method: 'POST',
      headers,
      body: JSON.stringify({
        query: 'launch readiness smoke probe',
        bindings: [{ spaceId: '1', priority: 0, topK: 3 }],
        includeCitations: true,
        topK: 3,
      }),
    });
    assert.ok(
      retrievalResponse.status >= 200 && retrievalResponse.status < 500,
      `retrievals returned ${retrievalResponse.status}`,
    );
  });
});

describe('knowledgebase open API contract guards', () => {
  it('declares api-key auth on context pack and retrieval operations', () => {
    const openapi = JSON.parse(
      readRepoFile('apis/open-api/knowledgebase-open-api.openapi.json'),
    );
    for (const routePath of ['/knowledge/v3/api/context_packs', '/knowledge/v3/api/retrievals']) {
      const operation = openapi.paths?.[routePath]?.post;
      assert.equal(operation?.['x-sdkwork-auth-mode'], 'api-key', routePath);
    }
  });
});
