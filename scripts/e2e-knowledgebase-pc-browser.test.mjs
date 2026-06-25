import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

const baseUrl = process.env.SDKWORK_KNOWLEDGEBASE_E2E_BASE_URL?.replace(/\/$/, '');

describe('knowledgebase PC browser shell e2e (optional live probe)', () => {
  it('loads renderer shell when SDKWORK_KNOWLEDGEBASE_E2E_BASE_URL is configured', async (t) => {
    if (!baseUrl) {
      t.skip('set SDKWORK_KNOWLEDGEBASE_E2E_BASE_URL to run live browser shell probe');
      return;
    }

    const response = await fetch(baseUrl, {
      headers: { accept: 'text/html' },
      redirect: 'follow',
    });
    assert.equal(response.ok, true, `expected HTML 200 from ${baseUrl}, got ${response.status}`);

    const html = await response.text();
    assert.match(html, /id=["']root["']/);
    assert.match(html, /sdkwork-knowledgebase-pc/i);
  });

  it('serves a JavaScript module entry for the renderer bootstrap', async (t) => {
    if (!baseUrl) {
      t.skip('set SDKWORK_KNOWLEDGEBASE_E2E_BASE_URL to run live browser shell probe');
      return;
    }

    const indexResponse = await fetch(baseUrl);
    const indexHtml = await indexResponse.text();
    const moduleMatch = indexHtml.match(/src=["']([^"']+\.tsx?)["']/);
    assert.ok(moduleMatch, 'index.html must reference a module script entry');

    const moduleUrl = new URL(moduleMatch[1], baseUrl);
    const moduleResponse = await fetch(moduleUrl);
    assert.equal(
      moduleResponse.ok,
      true,
      `expected module 200 from ${moduleUrl}, got ${moduleResponse.status}`,
    );
    const moduleSource = await moduleResponse.text();
    assert.match(moduleSource, /createRoot|ReactDOM|main/i);
  });
});
