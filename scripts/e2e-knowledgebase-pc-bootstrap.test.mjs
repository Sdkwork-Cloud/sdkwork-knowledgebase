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

describe('knowledgebase PC bootstrap e2e guards', () => {
  it('keeps production-critical feature flags enabled by default', async () => {
    const source = readRepoFile(
      'apps/sdkwork-knowledgebase-pc/packages/sdkwork-knowledgebase-pc-core/src/config/knowledgebaseFeatureFlags.ts',
    );
    assert.match(source, /documentVersionHistory:.*true/);
    assert.match(source, /documentPermissionsModal:.*true/);
    assert.match(source, /knowledgeMarketCatalog: devPreview/);
  });

  it('wires PermissionsModal to SDK-backed document access APIs', () => {
    const permissionsModal = readRepoFile(
      'apps/sdkwork-knowledgebase-pc/packages/sdkwork-knowledgebase-pc-knowledgebase/src/PermissionsModal.tsx',
    );
    assert.match(permissionsModal, /DocumentService\.getDocumentAccess/);
    assert.match(permissionsModal, /DocumentService\.updateDocumentVisibility/);
    assert.doesNotMatch(permissionsModal, /publicLink/);
  });

  it('exposes runtime bootstrap entrypoint for browser and desktop surfaces', () => {
    const bootstrap = readRepoFile(
      'apps/sdkwork-knowledgebase-pc/src/bootstrap/createKnowledgebasePcRuntime.ts',
    );
    assert.match(bootstrap, /createKnowledgebasePcRuntime/);
    assert.match(bootstrap, /createKnowledgebaseAppSdkClient/);
    assert.match(bootstrap, /configureKnowledgebaseAppSdk/);
  });

  it('declares Playwright shell, author, and search flow coverage', () => {
    const playwrightConfig = readRepoFile('apps/sdkwork-knowledgebase-pc/playwright.config.ts');
    const smokeSpec = readRepoFile('apps/sdkwork-knowledgebase-pc/e2e/shell.smoke.spec.ts');
    const authorSpec = readRepoFile('apps/sdkwork-knowledgebase-pc/e2e/author.flow.spec.ts');
    const searchSpec = readRepoFile('apps/sdkwork-knowledgebase-pc/e2e/search.flow.spec.ts');
    assert.match(playwrightConfig, /\.flow\\.spec\\.ts/);
    assert.match(playwrightConfig, /shell\\.smoke\\.spec\\.ts/);
    assert.match(smokeSpec, /knowledgebase-pc-app-shell/);
    assert.match(smokeSpec, /knowledgebase-pc-auth-shell/);
    assert.match(authorSpec, /auto-save/);
    assert.match(searchSpec, /search-source-row-doc/);
  });

  it('anchors app shell and navigation test ids for Playwright selectors', () => {
    const appShell = readRepoFile(
      'apps/sdkwork-knowledgebase-pc/packages/sdkwork-knowledgebase-pc-shell/src/AppShell.tsx',
    );
    const globalNav = readRepoFile(
      'apps/sdkwork-knowledgebase-pc/packages/sdkwork-knowledgebase-pc-shell/src/GlobalNav.tsx',
    );
    assert.match(appShell, /data-testid="knowledgebase-pc-app-shell"/);
    assert.match(globalNav, /data-testid=\{`knowledgebase-pc-nav-\$\{item\.id\}`\}/);
  });
});
