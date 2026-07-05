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

describe('knowledgebase multi-tenant isolation alignment', () => {
  it('enforces Postgres RLS and tenant session checkout', () => {
    const baseline = readRepoFile('database/ddl/baseline/postgres/0001_knowledgebase_baseline.sql');
    const tenantSession = readRepoFile(
      'crates/sdkwork-intelligence-knowledgebase-repository-sqlx/src/db/postgres_tenant_session.rs',
    );
    assert.match(baseline, /ENABLE ROW LEVEL SECURITY/);
    assert.match(tenantSession, /app\.current_tenant_id/);
    assert.match(tenantSession, /require_postgres_rls_tenant_id/);
  });

  it('covers HTTP tenant and organization guards in integration tests', () => {
    const tenantIsolation = readRepoFile(
      'crates/sdkwork-routes-knowledgebase-app-api/tests/integration_tenant_isolation.rs',
    );
    assert.match(tenantIsolation, /tenant_id_mismatch_rejects_space_retrieve/);
    assert.match(tenantIsolation, /organization_id_mismatch_rejects_when_runtime_org_configured/);
  });

  it('fail-closes space ACL when drive binding is missing', () => {
    const spaceService = readRepoFile(
      'crates/sdkwork-intelligence-knowledgebase-service/src/space.rs',
    );
    assert.match(spaceService, /not bound to a drive space for access control/);
  });

  it('enforces upload session space ACL at the app-api boundary', () => {
    const hostedUpload = readRepoFile(
      'crates/sdkwork-routes-knowledgebase-app-api/src/hosted_upload.rs',
    );
    assert.match(hostedUpload, /require_space_access[\s\S]*create_upload_session/u);
    assert.match(hostedUpload, /require_space_access[\s\S]*complete_upload_session/u);
  });

  it('documents tenant isolation operator procedures', () => {
    const spec = readRepoFile('specs/tenant-isolation.md');
    assert.match(spec, /tenant_id/);
    assert.match(spec, /RLS/);
  });
});
