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

describe('knowledgebase security standard alignment', () => {
  it('enforces fail-closed tenant and organization guards in hosted access', () => {
    const hostedAccess = readRepoFile(
      'crates/sdkwork-routes-knowledgebase-app-api/src/hosted_access.rs',
    );
    assert.match(hostedAccess, /tenant_id_mismatch/);
    assert.match(hostedAccess, /organization_id_mismatch/);
    assert.match(hostedAccess, /StatusCode::FORBIDDEN/);
    assert.match(hostedAccess, /ensure_runtime_tenant/);
  });

  it('routes runtime authentication through sdkwork-iam without product bypass flags', () => {
    const bootstrap = readRepoFile(
      'crates/sdkwork-routes-knowledgebase-app-api/src/bootstrap.rs',
    );
    assert.doesNotMatch(bootstrap, /SDKWORK_KNOWLEDGEBASE_DEV_AUTH_BYPASS/);
    assert.match(bootstrap, /validate_snowflake_node_id_for_production/);
    const appBootstrap = readRepoFile(
      'crates/sdkwork-routes-knowledgebase-app-api/src/web_bootstrap.rs',
    );
    assert.match(appBootstrap, /iam_web_request_context_resolver_from_env/);
  });

  it('documents tenant isolation integration coverage', () => {
    const tenantIsolationTest = readRepoFile(
      'crates/sdkwork-routes-knowledgebase-app-api/tests/integration_tenant_isolation.rs',
    );
    assert.match(tenantIsolationTest, /tenant_id_mismatch_rejects_space_retrieve/);
    assert.match(tenantIsolationTest, /organization_id_mismatch_rejects_when_runtime_org_configured/);
  });

  it('enforces backend knowledge.admin authorization policy', () => {
    const webBootstrap = readRepoFile(
      'crates/sdkwork-routes-knowledgebase-backend-api/src/web_bootstrap.rs',
    );
    assert.match(webBootstrap, /KnowledgeBackendAuthorizationPolicy/);
    assert.match(webBootstrap, /knowledge\.platform\.manage permission is required/);
    assert.match(webBootstrap, /with_authorization_policy/);
    assert.match(webBootstrap, /apply_knowledgebase_web_framework/);
  });

  it('wires tenant isolation and manifest authorization across HTTP surfaces', () => {
    const assembly = readRepoFile(
      'crates/sdkwork-routes-knowledgebase-backend-api/src/web_framework_assembly.rs',
    );
    assert.match(assembly, /EnforcePrincipalTenantIsolationPolicy/);
    const appBootstrap = readRepoFile(
      'crates/sdkwork-routes-knowledgebase-app-api/src/web_bootstrap.rs',
    );
    assert.match(appBootstrap, /ManifestAuthorizationPolicy/);
    assert.match(appBootstrap, /apply_knowledgebase_web_framework/);
    const rateLimit = readRepoFile(
      'crates/sdkwork-routes-knowledgebase-backend-api/src/web_rate_limit_store.rs',
    );
    assert.match(rateLimit, /is_production_like_environment/);
    assert.match(rateLimit, /std::process::exit\(1\)/);
  });

  it('persists durable kb_audit_event records', () => {
    const baseline = readRepoFile('database/ddl/baseline/postgres/0001_knowledgebase_baseline.sql');
    assert.match(baseline, /kb_audit_event/);
    const audit = readRepoFile('crates/sdkwork-knowledgebase-observability/src/audit.rs');
    assert.match(audit, /install_audit_persistence/);
  });

  it('wires framework web_audit_event persistence across HTTP surfaces', () => {
    const webAuditStore = readRepoFile(
      'crates/sdkwork-routes-knowledgebase-backend-api/src/web_audit_store.rs',
    );
    assert.match(webAuditStore, /shared_audit_emitter_pg/);
    assert.match(webAuditStore, /shared_audit_emitter\(/);
    assert.match(webAuditStore, /attach_knowledgebase_audit_emitter/);
    const appBootstrap = readRepoFile(
      'crates/sdkwork-routes-knowledgebase-app-api/src/web_bootstrap.rs',
    );
    assert.match(appBootstrap, /attach_knowledgebase_audit_emitter/);
    const baseline = readRepoFile('database/ddl/baseline/postgres/0001_knowledgebase_baseline.sql');
    assert.match(baseline, /web_audit_event/);
    const openBootstrap = readRepoFile(
      'crates/sdkwork-routes-knowledgebase-open-api/src/web_bootstrap.rs',
    );
    assert.match(openBootstrap, /attach_knowledgebase_audit_emitter/);
  });

  it('exposes stable SDK error codes and problem+json mapping in pc-core', () => {
    const codes = readRepoFile(
      'apps/sdkwork-knowledgebase-pc/packages/sdkwork-knowledgebase-pc-core/src/errors/knowledgebaseErrorCodes.ts',
    );
    assert.match(codes, /API_UNAVAILABLE/);
    const resolver = readRepoFile(
      'apps/sdkwork-knowledgebase-pc/packages/sdkwork-knowledgebase-pc-core/src/errors/resolveUserFacingError.ts',
    );
    assert.match(resolver, /parseSdkProblemDetails/);
    assert.match(resolver, /resolveKnowledgebaseErrorCode/);
  });

  it('propagates organization_id through open-api hosted bridge', () => {
    const hostedOpen = readRepoFile(
      'crates/sdkwork-routes-knowledgebase-app-api/src/hosted_open.rs',
    );
    assert.match(hostedOpen, /organization_id: context\.organization_id/);
    const openBootstrap = readRepoFile(
      'crates/sdkwork-routes-knowledgebase-open-api/src/web_bootstrap.rs',
    );
    assert.match(openBootstrap, /organization_id/);
  });

  it('gates AI demo fallbacks behind shouldUseKnowledgebaseDemoFallback', () => {
    const aiService = readRepoFile(
      'apps/sdkwork-knowledgebase-pc/packages/sdkwork-knowledgebase-pc-knowledgebase/src/services/ai.ts',
    );
    assert.match(aiService, /streamRewrite[\s\S]*shouldUseKnowledgebaseDemoFallback/);
    assert.match(aiService, /speechToText[\s\S]*shouldUseKnowledgebaseDemoFallback/);
    assert.match(aiService, /generateImage[\s\S]*shouldUseKnowledgebaseDemoFallback/);
  });

  it('avoids block_in_place in agent chat runtime bridges', () => {
    const agentRuntime = readRepoFile(
      'crates/sdkwork-routes-knowledgebase-app-api/src/agent_chat_runtime.rs',
    );
    assert.doesNotMatch(agentRuntime, /block_in_place/);
    assert.match(agentRuntime, /block_on_async/);
  });

  it('enforces agent profile space access helpers', () => {
    const hostedAccess = readRepoFile(
      'crates/sdkwork-routes-knowledgebase-app-api/src/hosted_access.rs',
    );
    assert.match(hostedAccess, /require_agent_profile_space_access/);
    assert.match(hostedAccess, /require_enabled_agent_bindings_space_access/);
  });

  it('avoids raw Error throws in PC bootstrap and package services', () => {
    const bootstrapFiles = [
      'apps/sdkwork-knowledgebase-pc/src/bootstrap/knowledgebaseIamRuntime.ts',
      'apps/sdkwork-knowledgebase-pc/src/bootstrap/sdkworkCorePcReactShim.ts',
    ];
    for (const relativePath of bootstrapFiles) {
      const source = readRepoFile(relativePath);
      assert.doesNotMatch(source, /throw new Error\(/);
      assert.match(source, /throwKnowledgebaseError/);
    }
  });

  it('uses i18n toasts for WeChat publish and preview outcomes', () => {
    const wechatPage = readRepoFile(
      'apps/sdkwork-knowledgebase-pc/packages/sdkwork-knowledgebase-pc-knowledgebase/src/WechatPublishPage.tsx',
    );
    const sendPreviewModal = readRepoFile(
      'apps/sdkwork-knowledgebase-pc/packages/sdkwork-knowledgebase-pc-knowledgebase/src/WechatSendPreviewModal.tsx',
    );
    const publishSidebar = readRepoFile(
      'apps/sdkwork-knowledgebase-pc/packages/sdkwork-knowledgebase-pc-knowledgebase/src/components/WechatPublishSidebar.tsx',
    );
    assert.match(wechatPage, /toast\.success\(t\('wechatPublishSuccess'\)\)/);
    assert.match(wechatPage, /toast\.success\(t\('wechatPreviewSuccess'\)\)/);
    assert.match(sendPreviewModal, /toast\.error\(t\('wechatPreviewRecipientRequired'/);
    assert.match(sendPreviewModal, /toast\.success\(\s*\n?\s*t\('wechatPreviewSentSuccess'/);
    assert.match(publishSidebar, /toast\.success\(t\('wechatDigestGenerateSuccess'\)\)/);
    assert.doesNotMatch(wechatPage, /toast\.(success|error)\(result\.message\)/);
    for (const source of [sendPreviewModal, publishSidebar]) {
      assert.doesNotMatch(source, /toast\.(success|error|info)\(['"][\u4e00-\u9fff]/);
    }
  });

  it('aligns production ingress hosts with cloud.split-services topology', () => {
    const ingress = readRepoFile('deployments/kubernetes/ingress.yaml');
    const productionEnv = readRepoFile('configs/topology/cloud.split-services.production.env');
    assert.match(ingress, /knowledgebase\.sdkwork\.com/);
    assert.match(ingress, /knowledgebase-admin\.sdkwork\.com/);
    assert.match(ingress, /knowledge\.sdkwork\.com/);
    assert.match(productionEnv, /knowledgebase\.sdkwork\.com/);
    assert.match(ingress, /cert-manager\.io\/cluster-issuer/);
    assert.doesNotMatch(ingress, /path: \/metrics/);
  });

  it('declares knowledge.platform.manage on backend OpenAPI operations', () => {
    const backendOpenApi = JSON.parse(
      readRepoFile(
        'sdks/sdkwork-knowledgebase-backend-sdk/openapi/knowledgebase-backend-api.openapi.json',
      ),
    );
    const httpMethods = new Set(['get', 'put', 'post', 'delete', 'patch', 'head', 'options', 'trace']);
    for (const pathItem of Object.values(backendOpenApi.paths ?? {})) {
      for (const [method, operation] of Object.entries(pathItem)) {
        if (!httpMethods.has(method) || !operation || typeof operation !== 'object') {
          continue;
        }
        assert.equal(
          operation['x-sdkwork-permission'],
          'knowledge.platform.manage',
          `backend operation ${operation.operationId ?? method} must declare knowledge.platform.manage`,
        );
      }
    }
  });

  it('requires manifest permissions on knowledge app-api spaces.create route', () => {
    const routeManifest = readRepoFile(
      'crates/sdkwork-routes-knowledgebase-app-api/src/http_route_manifest.rs',
    );
    assert.match(routeManifest, /"spaces\.create"[\s\S]*?"knowledge\.spaces\.write"/u);
    assert.match(routeManifest, /with_required_permission\(permission\)/u);
  });

  it('wires hosted WeChat service in full app runtime instead of stub ports', () => {
    const runtime = readRepoFile('crates/sdkwork-routes-knowledgebase-app-api/src/runtime.rs');
    assert.match(runtime, /HostedWechatService::new/);
    const hostedWechat = readRepoFile(
      'crates/sdkwork-routes-knowledgebase-app-api/src/hosted_wechat.rs',
    );
    assert.match(hostedWechat, /publish_articles/);
    assert.match(hostedWechat, /preview_articles/);
    assert.match(hostedWechat, /ensure_runtime_tenant/);
  });
});
