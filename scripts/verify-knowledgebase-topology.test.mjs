import assert from 'node:assert/strict';
import { readdir, readFile, stat } from 'node:fs/promises';
import path from 'node:path';
import test from 'node:test';

const ROOT = process.cwd();

async function exists(relativePath) {
  try {
    await stat(path.join(ROOT, relativePath));
    return true;
  } catch (error) {
    if (error?.code === 'ENOENT') {
      return false;
    }
    throw error;
  }
}

async function read(relativePath) {
  return readFile(path.join(ROOT, relativePath), 'utf8');
}

async function readJson(relativePath) {
  return JSON.parse((await read(relativePath)).replace(/^\uFEFF/u, ''));
}

async function listFiles(relativePath) {
  const absolutePath = path.join(ROOT, relativePath);
  const entries = await readdir(absolutePath, { withFileTypes: true });
  const files = [];
  for (const entry of entries) {
    const childPath = path.join(relativePath, entry.name);
    if (entry.isDirectory()) {
      files.push(...await listFiles(childPath));
    } else if (entry.isFile()) {
      files.push(childPath);
    }
  }
  return files;
}

function schemaNameFromRef(ref) {
  return ref?.split('/').pop();
}

function schemaHasTenantInput(schema, schemas, seen = new Set()) {
  if (!schema || typeof schema !== 'object') {
    return false;
  }
  if (schema.$ref) {
    const schemaName = schemaNameFromRef(schema.$ref);
    if (!schemaName || seen.has(schemaName)) {
      return false;
    }
    seen.add(schemaName);
    return schemaHasTenantInput(schemas[schemaName], schemas, seen);
  }
  if (schema.properties?.tenantId || schema.properties?.tenant_id) {
    return true;
  }
  for (const key of ['allOf', 'anyOf', 'oneOf']) {
    if (Array.isArray(schema[key]) && schema[key].some((item) => schemaHasTenantInput(item, schemas, seen))) {
      return true;
    }
  }
  if (schema.items && schemaHasTenantInput(schema.items, schemas, seen)) {
    return true;
  }
  return false;
}

test('declares SDKWork v3 deployment topology spec and profile env files for sdkwork-knowledgebase', async () => {
  assert.equal(await exists('specs/topology.spec.json'), true);
  assert.equal(await exists('scripts/lib/knowledgebase-topology.mjs'), true);
  assert.equal(await exists('scripts/knowledgebase-dev.mjs'), true);
  assert.equal(await exists('docs/architecture/tech/TECH-topology-standard.md'), true);

  const spec = await readJson('specs/topology.spec.json');
  assert.equal(spec.schemaVersion, 2);
  assert.equal(spec.kind, 'sdkwork.app.topology');
  assert.equal(spec.appId, 'sdkwork-knowledgebase');
  assert.equal(spec.archetype, 'application-http-gateway');
  assert.deepEqual(spec.vocabulary.deploymentProfile.allowed, ['standalone', 'cloud']);
  assert.equal(spec.defaults.developmentProfileId, 'standalone.unified-process.development');
  assert.equal(spec.defaults.productionProfileId, 'cloud.split-services.production');
  assert.ok(spec.surfaces['application.public-ingress']);
  assert.ok(spec.surfaces['application.backend-http']);
  assert.ok(spec.surfaces['application.open-http']);
  assert.ok(spec.surfaces['platform.api-gateway']);

  for (const profileId of [
    'standalone.unified-process.development',
    'standalone.unified-process.production',
    'cloud.split-services.development',
    'cloud.split-services.production',
  ]) {
    const profilePath = spec.profileFiles[profileId];
    assert.equal(await exists(profilePath), true, `${profilePath} should exist`);
    const profileEnv = await read(profilePath);
    assert.match(profileEnv, /SDKWORK_KNOWLEDGEBASE_PROFILE_ID=/);
    assert.match(profileEnv, /SDKWORK_KNOWLEDGEBASE_DEPLOYMENT_PROFILE=/);
    assert.doesNotMatch(profileEnv, /HOSTING|self-hosted|cloud-hosted/);
    assert.match(profileEnv, /VITE_SDKWORK_KNOWLEDGEBASE_APPLICATION_PUBLIC_HTTP_URL=/);
    assert.match(profileEnv, /VITE_SDKWORK_KNOWLEDGEBASE_PLATFORM_API_GATEWAY_HTTP_URL=/);
  }
});

test('root package.json wires @sdkwork/app-topology and standard dev scripts', async () => {
  const packageJson = await readJson('package.json');
  assert.equal(packageJson.dependencies['@sdkwork/app-topology'], 'file:../sdkwork-app-topology');
  assert.match(packageJson.scripts['dev:browser'], /dev:browser:postgres:unified-process:standalone/);
  assert.match(packageJson.scripts['dev:desktop'], /dev:desktop:postgres:unified-process:standalone/);
  assert.match(packageJson.scripts['dev:browser:postgres:unified-process:standalone'], /--database postgres/);
  assert.match(packageJson.scripts['dev:browser:postgres:unified-process:standalone'], /--deployment-profile standalone/);
  assert.match(packageJson.scripts['dev:browser:postgres:unified-process:standalone'], /--service-layout unified-process/);
  assert.equal(packageJson.scripts['knowledgebase:dev'], undefined);
  assert.equal(packageJson.scripts['knowledgebase:dev:cloud'], undefined);
  assert.match(packageJson.scripts['topology:validate'], /sdkwork-topology\.mjs validate/);
});

test('declares cloud gateway config bundles referenced by topology spec', async () => {
  const spec = await readJson('specs/topology.spec.json');
  for (const configFile of spec.packaging.cloudConfigFiles) {
    const configPath = path.join('configs', configFile);
    assert.equal(await exists(configPath), true, `${configPath} should exist`);
  }
});

test('knowledgebase dev orchestrator uses orchestration spec and gateway config', async () => {
  const devScript = await read('scripts/knowledgebase-dev.mjs');
  assert.match(devScript, /listOrchestrationProcesses/);
  assert.match(devScript, /buildProcessesFromOrchestration/);
  assert.match(devScript, /resolveCloudGatewayConfigPath/);
  assert.match(devScript, /resolveIamDevEnv/);
  assert.match(devScript, /IAM_APPLICATION_BOOTSTRAP_ENV/);
  assert.match(devScript, /--config/);
  assert.match(devScript, /--deployment-profile/);
  assert.match(devScript, /--database/);
  assert.doesNotMatch(devScript, /--hosting/);
  assert.doesNotMatch(devScript, /self-hosted|cloud-hosted/);
});

test('knowledgebase topology adapter exports IAM application bootstrap env aliases', async () => {
  const topologyAdapter = await read('scripts/lib/knowledgebase-topology.mjs');
  assert.match(topologyAdapter, /export const IAM_APPLICATION_BOOTSTRAP_ENV/u);
  assert.match(topologyAdapter, /SDKWORK_KNOWLEDGEBASE_APP_ROOT/u);
});

test('default PostgreSQL development path is not blocked by SQLite-only runtime wiring', async () => {
  const runtimeSource = await read('crates/sdkwork-routes-knowledgebase-app-api/src/runtime.rs');
  const appMain = await read('crates/sdkwork-knowledgebase-api-server/src/bin/app_main.rs');
  const backendMain = await read('crates/sdkwork-knowledgebase-api-server/src/bin/backend_main.rs');
  const openMain = await read('crates/sdkwork-knowledgebase-api-server/src/bin/open_main.rs');

  assert.doesNotMatch(
    runtimeSource,
    /postgresql SDKWORK_KNOWLEDGEBASE_DATABASE_URL is not wired to HTTP handlers yet|use sqlite/i,
  );
  assert.doesNotMatch(
    `${appMain}\n${backendMain}\n${openMain}`,
    /KnowledgebaseSqliteRuntime::connect|initialize knowledgebase sqlite runtime/,
  );
});

test('pc runtime config does not expose retired application deployment fields', async () => {
  const runtimeFiles = [
    'apps/sdkwork-knowledgebase-pc/packages/sdkwork-knowledgebase-pc-core/src/config/runtimeConfig.ts',
    'apps/sdkwork-knowledgebase-pc/packages/sdkwork-knowledgebase-pc-shell/src/SettingsModal.tsx',
    'apps/sdkwork-knowledgebase-pc/src/bootstrap/knowledgebaseIamRuntime.ts',
    'apps/sdkwork-knowledgebase-pc/src/i18n/locales/en/shell.json',
    'apps/sdkwork-knowledgebase-pc/src/i18n/locales/zh/shell.json',
  ];

  for (const relativePath of runtimeFiles) {
    const text = await read(relativePath);
    assert.doesNotMatch(text, /KnowledgebaseRuntimeConfig\['deploymentMode'\]/, relativePath);
    assert.doesNotMatch(text, /\bconfig\.(?:deploymentMode|hosting)\b/, relativePath);
    assert.doesNotMatch(
      text,
      /VITE_SDKWORK_KNOWLEDGEBASE_(?:DEPLOYMENT_MODE|HOSTING)/,
      relativePath,
    );
    assert.doesNotMatch(text, /"hosting"\s*:/, relativePath);
    assert.doesNotMatch(text, /\b(?:self-hosted|cloud-hosted)\b|HOSTING|DEPLOYMENT_MODE/, relativePath);
  }
});

test('OpenAPI and generated SDK inputs do not expose current tenant selectors', async () => {
  const openapiFiles = [
    'apis/open-api/knowledgebase-open-api.openapi.json',
    'apis/app-api/knowledgebase-app-api.openapi.json',
    'apis/backend-api/knowledgebase-backend-api.openapi.json',
    'sdks/sdkwork-knowledgebase-sdk/openapi/knowledgebase-open-api.openapi.json',
    'sdks/sdkwork-knowledgebase-app-sdk/openapi/knowledgebase-app-api.openapi.json',
    'sdks/sdkwork-knowledgebase-backend-sdk/openapi/knowledgebase-backend-api.openapi.json',
  ];

  const openapiViolations = [];
  for (const relativePath of openapiFiles) {
    const spec = await readJson(relativePath);
    const schemas = spec.components?.schemas ?? {};
    for (const [routePath, pathItem] of Object.entries(spec.paths ?? {})) {
      const pathParameters = pathItem.parameters ?? [];
      for (const [method, operation] of Object.entries(pathItem)) {
        if (!['get', 'put', 'post', 'delete', 'patch', 'head', 'options', 'trace'].includes(method)) {
          continue;
        }

        for (const parameter of [...pathParameters, ...(operation.parameters ?? [])]) {
          if (parameter?.name === 'tenantId' || parameter?.name === 'tenant_id') {
            openapiViolations.push(`${relativePath} ${method.toUpperCase()} ${routePath} parameter ${parameter.in}.${parameter.name}`);
          }
        }

        const content = operation.requestBody?.content ?? {};
        for (const [contentType, media] of Object.entries(content)) {
          if (schemaHasTenantInput(media.schema, schemas)) {
            const schemaName = schemaNameFromRef(media.schema?.$ref) ?? '<inline>';
            openapiViolations.push(`${relativePath} ${method.toUpperCase()} ${routePath} ${contentType} requestBody ${schemaName}`);
          }
        }
      }
    }
  }

  assert.deepEqual(openapiViolations, []);

  const generatedRoots = [
    'sdks/sdkwork-knowledgebase-sdk/sdkwork-knowledgebase-sdk-typescript/generated/server-openapi',
    'sdks/sdkwork-knowledgebase-app-sdk/sdkwork-knowledgebase-app-sdk-typescript/generated/server-openapi',
    'sdks/sdkwork-knowledgebase-backend-sdk/sdkwork-knowledgebase-backend-sdk-typescript/generated/server-openapi',
  ];
  const generatedViolations = [];
  for (const generatedRoot of generatedRoots) {
    if (!await exists(generatedRoot)) {
      continue;
    }
    const generatedFiles = (await listFiles(generatedRoot)).filter((relativePath) => {
      const normalized = relativePath.replaceAll('\\', '/');
      return (
        normalized.includes('/src/api/')
        || /\/src\/types\/.+request\.ts$/u.test(normalized)
        || /\/src\/types\/.+params\.ts$/u.test(normalized)
        || /\/src\/types\/.+options\.ts$/u.test(normalized)
      );
    });
    for (const relativePath of generatedFiles) {
      const text = await read(relativePath);
      if (/\btenantId\b|\btenant_id\b/u.test(text)) {
        generatedViolations.push(relativePath);
      }
    }
  }

  assert.deepEqual(generatedViolations, []);
});

test('production cloud topology orchestrates background worker and health probes', async () => {
  const spec = await readJson('specs/topology.spec.json');
  const productionProfile = spec.orchestration.profiles['cloud.split-services.production'];
  assert.ok(productionProfile, 'cloud.split-services.production orchestration profile must exist');

  const processIds = productionProfile.processes.map((process) => process.id);
  assert.ok(processIds.includes('application.background-worker'));
  assert.ok(processIds.includes('application.public-ingress'));
  assert.ok(processIds.includes('application.backend-http'));
  assert.ok(processIds.includes('application.open-http'));

  const bootstrapSource = await read('crates/sdkwork-routes-knowledgebase-app-api/src/bootstrap.rs');
  assert.match(bootstrapSource, /validate_snowflake_node_id_for_production/);
  assert.match(
    bootstrapSource,
    /SDKWORK_KNOWLEDGEBASE_ORGANIZATION_ID must be set when SDKWORK_KNOWLEDGEBASE_ENVIRONMENT is not development/,
  );
  assert.doesNotMatch(bootstrapSource, /DEV_AUTH_BYPASS/);

  const workerSource = await read('crates/sdkwork-knowledgebase-worker/src/lib.rs');
  assert.match(workerSource, /shutdown_signal/);

  for (const manifestPath of [
    'deployments/kubernetes/app-api-deployment.yaml',
    'deployments/kubernetes/backend-api-deployment.yaml',
    'deployments/kubernetes/open-api-deployment.yaml',
  ]) {
    const manifest = await read(manifestPath);
    assert.match(manifest, /path: \/livez/);
    assert.match(manifest, /path: \/readyz/);
  }

  const workerManifest = await read('deployments/kubernetes/worker-deployment.yaml');
  assert.match(workerManifest, /terminationGracePeriodSeconds/);
  assert.match(workerManifest, /SDKWORK_KNOWLEDGEBASE_SNOWFLAKE_NODE_ID/);
});

test('route binaries read topology bind env keys', async () => {
  const appMain = await read('crates/sdkwork-knowledgebase-api-server/src/bin/app_main.rs');
  const backendMain = await read('crates/sdkwork-knowledgebase-api-server/src/bin/backend_main.rs');
  const openMain = await read('crates/sdkwork-knowledgebase-api-server/src/bin/open_main.rs');

  assert.match(appMain, /SDKWORK_KNOWLEDGEBASE_APPLICATION_PUBLIC_INGRESS_BIND/);
  assert.doesNotMatch(appMain, /SDKWORK_KNOWLEDGEBASE_APP_LISTEN/);

  assert.match(backendMain, /SDKWORK_KNOWLEDGEBASE_APPLICATION_BACKEND_HTTP_BIND/);
  assert.doesNotMatch(backendMain, /SDKWORK_KNOWLEDGEBASE_BACKEND_LISTEN/);

  assert.match(openMain, /SDKWORK_KNOWLEDGEBASE_APPLICATION_OPEN_HTTP_BIND/);
  assert.doesNotMatch(openMain, /SDKWORK_KNOWLEDGEBASE_OPEN_LISTEN/);
});
