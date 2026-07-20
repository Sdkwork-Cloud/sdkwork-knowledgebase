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

export function resolveSmokeBackendBaseUrl() {
  return (
    process.env.SDKWORK_KNOWLEDGEBASE_SMOKE_BACKEND_URL?.trim()
    || process.env.SDKWORK_KNOWLEDGEBASE_SMOKE_BACKEND_API_BASE_URL?.trim()
    || ''
  );
}

export function resolveSmokeAccessToken() {
  return process.env.SDKWORK_KNOWLEDGEBASE_SMOKE_ACCESS_TOKEN?.trim() || '';
}

describe('knowledgebase backend admin smoke helpers', () => {
  it('lists ingest sources when backend credentials are configured', async (t) => {
    const baseUrl = resolveSmokeBackendBaseUrl();
    const accessToken = resolveSmokeAccessToken();
    if (!baseUrl || !accessToken) {
      t.skip(
        'Set SDKWORK_KNOWLEDGEBASE_SMOKE_BACKEND_URL and SDKWORK_KNOWLEDGEBASE_SMOKE_ACCESS_TOKEN to run live admin smoke checks.',
      );
      return;
    }

    const normalizedBase = baseUrl.replace(/\/+$/, '');
    const response = await fetch(`${normalizedBase}/backend/v3/api/knowledge/sources`, {
      method: 'GET',
      headers: {
        accept: 'application/json',
        'Access-Token': accessToken,
      },
    });
    assert.equal(response.status, 200, `sources returned ${response.status}`);
    const payload = await response.json();
    assert.ok(Array.isArray(payload.items) || Array.isArray(payload.sources) || payload.items === undefined);
  });

  it('rejects unauthenticated backend source listing', async (t) => {
    const baseUrl = resolveSmokeBackendBaseUrl();
    if (!baseUrl) {
      t.skip('Set SDKWORK_KNOWLEDGEBASE_SMOKE_BACKEND_URL to run live admin smoke checks.');
      return;
    }

    const normalizedBase = baseUrl.replace(/\/+$/, '');
    const response = await fetch(`${normalizedBase}/backend/v3/api/knowledge/sources`, {
      method: 'GET',
      headers: { accept: 'application/json' },
    });
    assert.equal(response.status, 401);
  });
});

describe('knowledgebase backend admin contract guards', () => {
  it('declares knowledge.platform.manage permission on source operations', () => {
    const openapi = JSON.parse(
      readRepoFile('apis/backend-api/knowledgebase-backend-api.openapi.json'),
    );
    const listOperation = openapi.paths?.['/backend/v3/api/knowledge/sources']?.get;
    assert.equal(listOperation?.['x-sdkwork-permission'], 'knowledge.platform.manage');
  });
});
