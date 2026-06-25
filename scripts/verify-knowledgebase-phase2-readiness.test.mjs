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
  });

  it('indexes Phase 2 commercial criteria in product PRD map', () => {
    const prd = readRepoFile('docs/product/prd/PRD.md');
    const phase2 = readRepoFile('docs/product/prd/PRD-phase2-commercial-saas.md');
    assert.match(prd, /PRD-phase2-commercial-saas\.md/);
    assert.match(phase2, /Usage metering exported for billing/);
    assert.match(phase2, /ADR-2026-06-24-phase2-postgres-rls-multi-tenant/);
  });
});
