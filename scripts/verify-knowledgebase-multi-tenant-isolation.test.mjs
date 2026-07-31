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
  it('enforces Postgres tenant and organization RLS session scope', () => {
    const organizationMigration = readRepoFile(
      'database/migrations/postgres/202607310001_core_organization_isolation.up.sql',
    );
    const tenantSession = readRepoFile(
      'crates/sdkwork-intelligence-knowledgebase-repository-sqlx/src/db/postgres_tenant_session.rs',
    );
    assert.match(organizationMigration, /ENABLE ROW LEVEL SECURITY/);
    assert.match(organizationMigration, /organization_isolation/);
    assert.match(organizationMigration, /app\.current_organization_id/);
    assert.match(tenantSession, /app\.current_tenant_id/);
    assert.match(tenantSession, /app\.current_organization_id/);
    assert.match(tenantSession, /require_postgres_rls_tenant_id/);
    assert.match(tenantSession, /require_postgres_rls_organization_id/);
  });

  it('binds source reads and writes to tenant and organization', () => {
    const sourceStore = readRepoFile(
      'crates/sdkwork-intelligence-knowledgebase-repository-sqlx/src/sqlite_import_stores.rs',
    );
    const isolationTest = readRepoFile(
      'crates/sdkwork-intelligence-knowledgebase-repository-sqlx/tests/sqlite_source_connector_metadata.rs',
    );
    assert.match(sourceStore, /WHERE tenant_id = \$1 AND organization_id = \$2/);
    assert.match(isolationTest, /source_store_isolates_organizations_within_one_tenant/);
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

  it('documents tenant isolation operator procedures', () => {
    const spec = readRepoFile('specs/tenant-isolation.md');
    assert.match(spec, /tenant_id/);
    assert.match(spec, /RLS/);
  });
});
