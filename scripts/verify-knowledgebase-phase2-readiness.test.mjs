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

describe('knowledgebase Phase 2 commercial readiness alignment', () => {
  it('documents Postgres RLS multi-tenant isolation decision', () => {
    const adr = readRepoFile('docs/adr/ADR-2026-06-24-phase2-postgres-rls-multi-tenant.md');
    assert.match(adr, /Row Level Security \(RLS\)/);
    assert.match(adr, /app\.current_tenant_id/);
  });

  it('ships Postgres RLS migration for tenant-scoped kb_* tables', () => {
    const baseline = readRepoFile('database/ddl/baseline/postgres/0001_knowledgebase_baseline.sql');
    const crateMigration = readRepoFile(
      'crates/sdkwork-intelligence-knowledgebase-repository-sqlx/migrations/postgres/V202606260001__knowledgebase_postgres_rls.sql',
    );
    assert.match(baseline, /ENABLE ROW LEVEL SECURITY/);
    assert.match(baseline, /tenant_isolation/);
    assert.match(baseline, /kb_audit_event/);
    assert.match(crateMigration, /app\.current_tenant_id/);
  });

  it('wires Postgres tenant session on pool checkout (Phase 2.2)', () => {
    const bootstrap = readRepoFile(
      'crates/sdkwork-intelligence-knowledgebase-repository-sqlx/src/db/bootstrap.rs',
    );
    const tenantSession = readRepoFile(
      'crates/sdkwork-intelligence-knowledgebase-repository-sqlx/src/db/postgres_tenant_session.rs',
    );
    assert.match(bootstrap, /after_connect/);
    assert.match(bootstrap, /set_postgres_session_tenant_id/);
    assert.match(tenantSession, /require_postgres_rls_tenant_id/);
    const webAudit = readRepoFile('crates/sdkwork-routes-knowledgebase-backend-api/src/web_audit_store.rs');
    assert.match(webAudit, /connect_and_bootstrap_webstore_database_from_env/);
    assert.match(webAudit, /shared_audit_emitter_pg/);
  });

  it('exports billable usage counters from observability crate', () => {
    const billingModule = readRepoFile(
      'crates/sdkwork-knowledgebase-observability/src/billing_metrics.rs',
    );
    const observabilityLib = readRepoFile('crates/sdkwork-knowledgebase-observability/src/lib.rs');
    assert.match(billingModule, /knowledge_retrievals_total/);
    assert.match(billingModule, /knowledge_context_packs_total/);
    assert.match(billingModule, /billing_event = "knowledge\.retrieval\.completed"/);
    assert.match(observabilityLib, /billing_metrics::render_billing_prometheus_metrics/);
  });

  it('records retrieval and context pack billing events in service layer', () => {
    const retrievalService = readRepoFile(
      'crates/sdkwork-intelligence-knowledgebase-service/src/retrieval.rs',
    );
    const ingestService = readRepoFile(
      'crates/sdkwork-intelligence-knowledgebase-service/src/ingest/service.rs',
    );
    assert.match(retrievalService, /record_retrieval_completed/);
    assert.match(retrievalService, /record_context_pack_completed/);
    assert.match(ingestService, /record_ingest_job_succeeded/);
    assert.match(ingestService, /record_ingest_job_failed/);
  });

  it('documents audit retention and GDPR operator procedures', () => {
    const runbook = readRepoFile('docs/runbooks/audit-retention.md');
    assert.match(runbook, /kb_audit_event/);
    assert.match(runbook, /GDPR/);
    assert.match(runbook, /365 days/);
    assert.match(runbook, /compliance\.auditEvents\.export/);
    assert.match(runbook, /compliance\.auditEvents\.anonymizeActor/);
  });

  it('indexes Phase 2 commercial criteria in product PRD map', () => {
    const prd = readRepoFile('docs/product/prd/PRD.md');
    const phase2 = readRepoFile('docs/product/prd/PRD-phase2-commercial-saas.md');
    assert.match(prd, /PRD-phase2-commercial-saas\.md/);
    assert.match(phase2, /Usage metering exported for billing/);
    assert.match(phase2, /ADR-2026-06-24-phase2-postgres-rls-multi-tenant/);
  });
});
