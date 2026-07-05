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

describe('knowledgebase billing metering alignment', () => {
  it('exports Prometheus billing counters and structured billing_event logs', () => {
    const billingModule = readRepoFile(
      'crates/sdkwork-knowledgebase-observability/src/billing_metrics.rs',
    );
    assert.match(billingModule, /knowledge_retrievals_total/);
    assert.match(billingModule, /knowledge_ingest_jobs_succeeded_total/);
    assert.match(billingModule, /knowledge_ingest_jobs_failed_total/);
    assert.match(billingModule, /billing_event/);
  });

  it('records billable events from retrieval and ingest services', () => {
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

  it('mounts billing metrics on observability HTTP surface', () => {
    const observabilityLib = readRepoFile('crates/sdkwork-knowledgebase-observability/src/lib.rs');
    assert.match(observabilityLib, /render_billing_prometheus_metrics/);
  });

  it('documents tenant status API for commercial operators', () => {
    const backendPaths = readRepoFile(
      'crates/sdkwork-routes-knowledgebase-backend-api/src/paths.rs',
    );
    assert.match(backendPaths, /tenants\/current/);
    const hostedBackend = readRepoFile(
      'crates/sdkwork-routes-knowledgebase-app-api/src/hosted_backend.rs',
    );
    assert.match(hostedBackend, /retrieve_current_tenant[\s\S]*summarize_tenant_knowledgebase/u);
  });
});
