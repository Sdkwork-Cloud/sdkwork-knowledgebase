import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, it } from 'node:test';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, '..');

const OPENAPI_PATHS = [
  'sdks/sdkwork-knowledgebase-app-sdk/openapi/knowledgebase-app-api.openapi.json',
  'sdks/sdkwork-knowledgebase-backend-sdk/openapi/knowledgebase-backend-api.openapi.json',
  'sdks/sdkwork-knowledgebase-sdk/openapi/knowledgebase-open-api.openapi.json',
];

function readRepoFile(relativePath) {
  return readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

describe('knowledgebase observability standard alignment', () => {
  it('declares traceId on ProblemDetails across all API authorities', () => {
    for (const relativePath of OPENAPI_PATHS) {
      const openapi = JSON.parse(readRepoFile(relativePath));
      const problem = openapi.components?.schemas?.ProblemDetails;
      assert.ok(problem, `${relativePath} must declare ProblemDetails`);
      assert.ok(
        problem.properties?.traceId,
        `${relativePath} ProblemDetails must include traceId`,
      );
    }
  });

  it('exports auth failure, audit, and billing counters from observability crate', () => {
    const observabilityLib = readRepoFile('crates/sdkwork-knowledgebase-observability/src/lib.rs');
    const auditModule = readRepoFile('crates/sdkwork-knowledgebase-observability/src/audit.rs');
    const billingModule = readRepoFile(
      'crates/sdkwork-knowledgebase-observability/src/billing_metrics.rs',
    );
    assert.match(observabilityLib, /knowledge_api_auth_failures_total/);
    assert.match(observabilityLib, /knowledgebase_health_status/);
    assert.match(auditModule, /knowledge_audit_space_member_granted_total/);
    assert.match(billingModule, /knowledge_retrievals_total/);
    assert.match(billingModule, /knowledge_ingest_jobs_succeeded_total/);
    assert.match(auditModule, /knowledge\.space\.member_granted/);
    assert.match(auditModule, /knowledge\.space\.member_revoked/);
  });

  it('documents audit and OTLP env keys for production operations', () => {
    const deploymentsReadme = readRepoFile('deployments/README.md');
    const productionEnv = readRepoFile('configs/topology/cloud.split-services.production.env');
    assert.match(deploymentsReadme, /OTEL_EXPORTER_OTLP_ENDPOINT/);
    assert.match(deploymentsReadme, /knowledgebase_health_status/);
    assert.match(deploymentsReadme, /knowledge\.document\.visibility_changed/);
    assert.match(productionEnv, /OTEL_EXPORTER_OTLP_ENDPOINT/);
    assert.match(productionEnv, /SDKWORK_KNOWLEDGEBASE_LOG_FORMAT/);
  });
});
