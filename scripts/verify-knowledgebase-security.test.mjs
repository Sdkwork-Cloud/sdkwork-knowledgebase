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
    assert.match(bootstrap, /ALLOW_STATIC_SNOWFLAKE_NODE_ID/);
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

  it('enforces backend knowledge.platform.manage authorization policy', () => {
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

  it('fails closed when AI APIs are unavailable', () => {
    const aiService = readRepoFile(
      'apps/sdkwork-knowledgebase-pc/packages/sdkwork-knowledgebase-pc-knowledgebase/src/services/ai.ts',
    );
    assert.match(aiService, /function requireAiApi/);
    assert.match(aiService, /isKnowledgebaseApiAvailable/);
    assert.match(aiService, /throwKnowledgebaseError/);
    assert.doesNotMatch(aiService, /shouldUseKnowledgebaseDemoFallback/);
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
    assert.doesNotMatch(wechatPage, /toast\.success\(t\('wechatPreviewSuccess'\)\)/);
    assert.match(sendPreviewModal, /toast\.error\(t\('wechatPreviewRecipientRequired'/);
    assert.match(sendPreviewModal, /toast\.success\(\s*\n?\s*t\('wechatPreviewSentSuccess'/);
    assert.match(publishSidebar, /toast\.success\(t\('wechatDigestGenerateSuccess'\)\)/);
    assert.doesNotMatch(wechatPage, /toast\.(success|error)\(result\.message\)/);
    for (const source of [sendPreviewModal, publishSidebar]) {
      assert.doesNotMatch(source, /toast\.(success|error|info)\(['"][\u4e00-\u9fff]/);
    }
  });

  it('aligns production ingress hosts with cloud.production topology', () => {
    const ingress = readRepoFile('deployments/kubernetes/ingress.yaml');
    const productionEnv = readRepoFile('configs/topology/cloud.production.env');
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
        const permission = operation['x-sdkwork-permission'];
        assert.ok(
          permission === 'knowledge.platform.manage',
          `backend operation ${operation.operationId ?? method} must declare knowledge.platform.manage (got ${permission ?? 'missing'})`,
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

  it('enforces upload session space ACL in hosted upload service', () => {
    const hostedUpload = readRepoFile(
      'crates/sdkwork-routes-knowledgebase-app-api/src/hosted_upload.rs',
    );
    assert.match(hostedUpload, /require_space_access/);
    assert.match(hostedUpload, /create_upload_session[\s\S]*context: KnowledgeAppRequestContext/u);
    assert.match(hostedUpload, /complete_upload_session[\s\S]*require_space_access/u);
  });

  it('fail-closes space access when drive binding is missing', () => {
    const spaceService = readRepoFile(
      'crates/sdkwork-intelligence-knowledgebase-service/src/space.rs',
    );
    assert.match(
      spaceService,
      /access_control[\s\S]*drive_space_id[\s\S]*AccessDenied/,
    );
  });

  it('sanitizes WeChat editor HTML before insertion', () => {
    const wechatPage = readRepoFile(
      'apps/sdkwork-knowledgebase-pc/packages/sdkwork-knowledgebase-pc-knowledgebase/src/WechatPublishPage.tsx',
    );
    assert.match(wechatPage, /sanitizeEditorHtml/);
    assert.match(wechatPage, /insertHtmlToEditor[\s\S]*sanitizeEditorHtml\(html\)/);
  });

  it('gates synthetic search media behind demo fallback', () => {
    const buildRelatedMedia = readRepoFile(
      'apps/sdkwork-knowledgebase-pc/packages/sdkwork-knowledgebase-pc-search/src/services/buildRelatedMedia.ts',
    );
    assert.match(buildRelatedMedia, /shouldUseKnowledgebaseDemoFallback/);
    assert.match(buildRelatedMedia, /buildKbMediaItems\(docs, allowDemo\)/);
  });

  it('uses sdkwork_utils_rust is_blank in gateway bootstrap', () => {
    const gatewayBootstrap = readRepoFile(
      'crates/sdkwork-api-knowledgebase-assembly/src/bootstrap.rs',
    );
    assert.match(gatewayBootstrap, /sdkwork_utils_rust::is_blank/);
    assert.doesNotMatch(gatewayBootstrap, /\.trim\(\)\.is_empty\(\)/);
  });

  it('does not ship synthetic third-party asset library demo content', () => {
    const assetLibrary = readRepoFile(
      'apps/sdkwork-knowledgebase-pc/packages/sdkwork-knowledgebase-pc-knowledgebase/src/components/AssetLibraryModal.tsx',
    );
    assert.doesNotMatch(assetLibrary, /MOCK_IMAGES|MOCK_AUDIOS|MOCK_VIDEOS/);
    assert.doesNotMatch(assetLibrary, /images\.unsplash\.com|soundhelix\.com|sample-videos\.com/);
    assert.match(assetLibrary, /listAssetLibraryItemsPage/);
    assert.match(assetLibrary, /useApiAssets/);
  });

  it('blocks WeChat scan and AI cover synthetic flows', () => {
    const wechatPage = readRepoFile(
      'apps/sdkwork-knowledgebase-pc/packages/sdkwork-knowledgebase-pc-knowledgebase/src/WechatPublishPage.tsx',
    );
    assert.doesNotMatch(wechatPage, /triggerScanSimulation|triggerAiCoverGeneration/);
    assert.match(wechatPage, /handleInsertConfirm/);
    assert.match(wechatPage, /toastKnowledgebaseError/);
  });

  it('wires tenant-scoped dynamic rate limit policy from web store', () => {
    const webAuditStore = readRepoFile(
      'crates/sdkwork-routes-knowledgebase-backend-api/src/web_audit_store.rs',
    );
    assert.match(webAuditStore, /shared_dynamic_policy_bundle/);
    assert.match(webAuditStore, /with_dynamic_rate_limit_policy_source/);
    assert.match(webAuditStore, /with_dynamic_tenant_runtime_profile_source/);
  });

  it('uses API-backed WeChat applets and settings without demo fallback', () => {
    const appletModal = readRepoFile(
      'apps/sdkwork-knowledgebase-pc/packages/sdkwork-knowledgebase-pc-knowledgebase/src/components/WechatAppletModal.tsx',
    );
    assert.match(appletModal, /WechatService\.getApplets/);
    assert.doesNotMatch(appletModal, /shouldUseKnowledgebaseDemoFallback/);
    const settingsModal = readRepoFile(
      'apps/sdkwork-knowledgebase-pc/packages/sdkwork-knowledgebase-pc-knowledgebase/src/KnowledgeBaseSettingsModal.tsx',
    );
    assert.match(settingsModal, /isKnowledgebaseApiAvailable|toastKnowledgebaseError/);
    const musicPlayer = readRepoFile(
      'apps/sdkwork-knowledgebase-pc/packages/sdkwork-knowledgebase-pc-knowledgebase/src/components/players/MusicPlayer.tsx',
    );
    assert.match(musicPlayer, /isKnowledgebaseApiAvailable|toastKnowledgebaseError/);
    const widgetTemplates = readRepoFile(
      'apps/sdkwork-knowledgebase-pc/packages/sdkwork-knowledgebase-pc-knowledgebase/src/utils/wechatWidgetTemplates.ts',
    );
    assert.doesNotMatch(widgetTemplates, /images\.unsplash\.com/);
  });

  it('wires PC admin console to generated backend SDK', () => {
    const adminConsole = readRepoFile(
      'apps/sdkwork-knowledgebase-pc/packages/sdkwork-knowledgebase-pc-shell/src/KnowledgebaseAdminConsole.tsx',
    );
    assert.match(adminConsole, /sdkwork-knowledgebase-pc-admin-core/);
    assert.match(adminConsole, /canAccessKnowledgebaseAdminConsole/);
    assert.match(adminConsole, /providerHealth\.list/);
    assert.match(adminConsole, /retrievalTraces\.list/);
    assert.match(adminConsole, /spaces\.list/);
    assert.match(adminConsole, /spaces\.members\.list/);
    assert.match(adminConsole, /loadAdminSpaceMembers/);
    const adminService = readRepoFile(
      'apps/sdkwork-knowledgebase-pc/packages/sdkwork-knowledgebase-pc-admin-core/src/api/knowledgebaseBackendAdminService.ts',
    );
    assert.match(adminService, /getPath/);
    const globalNav = readRepoFile(
      'apps/sdkwork-knowledgebase-pc/packages/sdkwork-knowledgebase-pc-shell/src/GlobalNav.tsx',
    );
    assert.match(globalNav, /knowledgebase-pc-nav-admin/);
    const backendRegistry = readRepoFile(
      'apps/sdkwork-knowledgebase-pc/packages/sdkwork-knowledgebase-pc-admin-core/src/api/knowledgebaseBackendApiRegistry.ts',
    );
    assert.match(backendRegistry, /knowledge\.platform\.manage/);
    const appRoutes = readRepoFile('apps/sdkwork-knowledgebase-pc/src/App.tsx');
    assert.match(appRoutes, /path="\/admin"/);
  });

  it('implements backend spaces admin APIs in hosted runtime', () => {
    const hostedBackend = readRepoFile(
      'crates/sdkwork-routes-knowledgebase-app-api/src/hosted_backend.rs',
    );
    assert.match(hostedBackend, /async fn list_spaces/);
    assert.match(hostedBackend, /async fn list_space_members/);
    assert.match(hostedBackend, /list_space_members_admin_with_runtime/);
    assert.match(hostedBackend, /async fn list_indexes/);
    const spaceStore = readRepoFile(
      'crates/sdkwork-intelligence-knowledgebase-repository-sqlx/src/sqlite_space_stores.rs',
    );
    assert.match(spaceStore, /list_active_spaces/);
    const policyBootstrap = readRepoFile(
      'crates/sdkwork-routes-knowledgebase-backend-api/src/web_policy_bootstrap.rs',
    );
    assert.match(policyBootstrap, /web_rate_limit_policy/);
    assert.match(policyBootstrap, /web_tenant_runtime_profile/);
  });
});
