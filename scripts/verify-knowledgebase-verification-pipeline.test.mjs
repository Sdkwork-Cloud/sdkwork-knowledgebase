import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, it } from 'node:test';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, '..');
const verificationPipelineTestCommand =
  'node --test --experimental-test-isolation=none scripts/verify-knowledgebase-verification-pipeline.test.mjs';

function readRepoFile(relativePath) {
  return readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

function readJson(relativePath) {
  return JSON.parse(readRepoFile(relativePath));
}

describe('knowledgebase verification pipeline output hygiene', () => {
  it('runs Rust workspace tests with quiet test harness output', () => {
    const packageJson = readJson('package.json');
    assert.equal(packageJson.scripts['test:rust'], 'cargo test --workspace -- --quiet');

    const verifyPhase1 = readRepoFile('tools/verify_phase1.ps1');
    assert.match(verifyPhase1, /Invoke-Checked cargo test --workspace -- --quiet/);
  });

  it('includes the verification pipeline guard in root check', () => {
    const packageJson = readJson('package.json');
    assert.equal(packageJson.scripts['check:verification-pipeline'], verificationPipelineTestCommand);
    assert.equal(packageJson.scripts.check, 'pnpm exec sdkwork-app check');
    assert.match(packageJson.scripts['_sdkwork:check'], /pnpm check:verification-pipeline/);
  });
});
