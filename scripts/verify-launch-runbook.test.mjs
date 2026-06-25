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

function readJson(relativePath) {
  return JSON.parse(readRepoFile(relativePath));
}

describe('knowledgebase launch runbook alignment', () => {
  it('documents production launch sequencing and verification gates', () => {
    const runbook = readRepoFile('deployments/runbooks/production-launch.md');
    for (const section of [
      'Pre-flight',
      'Database bootstrap',
      'Deployment smoke',
      'Observability',
      'Release artifacts',
      'Rollback',
    ]) {
      assert.match(runbook, new RegExp(section, 'i'), `missing runbook section: ${section}`);
    }
    assert.match(runbook, /backup-restore\.md/);
    assert.match(runbook, /pnpm db:bootstrap/);
    assert.match(runbook, /pnpm test:smoke/);
    assert.match(runbook, /SDKWORK_KNOWLEDGEBASE_LOG_FORMAT=json/);
  });

  it('keeps backup-restore runbook verification checklist intact', () => {
    const backupRunbook = readRepoFile('deployments/runbooks/backup-restore.md');
    assert.match(backupRunbook, /pg_dump/);
    assert.match(backupRunbook, /\/readyz/);
  });

  it('defaults structured JSON logging in production topology', () => {
    const topology = readRepoFile('configs/topology/cloud.split-services.production.env');
    assert.match(topology, /^SDKWORK_KNOWLEDGEBASE_LOG_FORMAT=json/m);
  });

  it('documents OTEL enablement without exposing metrics on public ingress', () => {
    const topology = readRepoFile('configs/topology/cloud.split-services.production.env');
    const ingress = readRepoFile('deployments/kubernetes/ingress.yaml');
    assert.match(topology, /OTEL_EXPORTER_OTLP_ENDPOINT/);
    assert.doesNotMatch(ingress, /\/metrics/);
  });

  it('indexes Playwright author and search launch flows', () => {
    const playwrightConfig = readRepoFile('apps/sdkwork-knowledgebase-pc/playwright.config.ts');
    assert.match(playwrightConfig, /\.flow\\.spec\\.ts/);
    assert.match(playwrightConfig, /shell\\.smoke\\.spec\\.ts/);
  });

  it('requires supply-chain release gates in app manifest', () => {
    const manifest = readJson('sdkwork.app.config.json');
    assert.equal(manifest.security.checksumRequired, true);
    assert.equal(manifest.security.signatureRequired, true);
    assert.equal(manifest.security.sbomRequired, true);
    const webPackage = manifest.artifacts.installConfig.packages.find(
      (entry) => entry.id === 'web-production',
    );
    assert.ok(webPackage?.enabled, 'web-production package must remain enabled for launch');
  });

  it('keeps desktop bundles prelaunch-disabled until CI packaging ships', () => {
    const manifest = readJson('sdkwork.app.config.json');
    const desktopPackages = manifest.artifacts.installConfig.packages.filter((entry) =>
      String(entry.platform ?? '').startsWith('DESKTOP_'),
    );
    assert.ok(desktopPackages.length >= 3);
    for (const pkg of desktopPackages) {
      assert.equal(pkg.enabled, false, `${pkg.id} must stay disabled prelaunch`);
      assert.match(pkg.metadata?.reason ?? '', /desktop CI/i);
    }
  });

  it('indexes all three SDK families for release consumption', () => {
    const componentSpec = readJson('specs/component.spec.json');
    const families = componentSpec.contracts.sdkClients.map((entry) => entry.family).sort();
    assert.deepEqual(families, [
      'sdkwork-knowledgebase-app-sdk',
      'sdkwork-knowledgebase-backend-sdk',
      'sdkwork-knowledgebase-sdk',
    ]);
  });

  it('blocks WeChat demo fallback in production builds', () => {
    const wechatService = readRepoFile(
      'apps/sdkwork-knowledgebase-pc/packages/sdkwork-knowledgebase-pc-knowledgebase/src/services/wechat.ts',
    );
    assert.match(wechatService, /assertWechatDemoFallbackAllowed/);
    assert.match(wechatService, /shouldUseKnowledgebaseDemoFallback/);
  });

  it('does not ship debug-only Playwright flows in launch CI', () => {
    const e2eDir = path.join(repoRoot, 'apps/sdkwork-knowledgebase-pc/e2e');
    const debugFlow = path.join(e2eDir, 'debug.flow.spec.ts');
    assert.throws(() => readFileSync(debugFlow, 'utf8'), /ENOENT/);
  });
});
