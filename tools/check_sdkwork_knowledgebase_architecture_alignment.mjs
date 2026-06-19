#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const failures = [];
const warnings = [];

function readText(relativePath) {
  const absolutePath = path.join(repoRoot, relativePath);
  if (!fs.existsSync(absolutePath)) {
    failures.push(`${relativePath} must exist`);
    return '';
  }
  return fs.readFileSync(absolutePath, 'utf8');
}

function readJson(relativePath) {
  return JSON.parse(readText(relativePath));
}

function assert(condition, message) {
  if (!condition) {
    failures.push(message);
  }
}

function assertDirectory(relativePath) {
  assert(fs.existsSync(path.join(repoRoot, relativePath)), `${relativePath}/ must exist`);
}

function assertCargoDependsOnWebFramework(relativeCrateToml) {
  const text = readText(relativeCrateToml);
  assert(
    text.includes('sdkwork-web-axum.workspace = true')
      || text.includes('sdkwork-web-axum = {'),
    `${relativeCrateToml} must depend on sdkwork-web-axum per WEB_FRAMEWORK_SPEC.md`,
  );
}

const requiredDirectories = [
  'apis',
  'apps',
  'crates',
  'sdks',
  'deployments',
  'configs',
  'scripts',
  'docs',
  'tests',
  '.sdkwork',
  'specs',
];

for (const directory of requiredDirectories) {
  assertDirectory(directory);
}

assert(fs.existsSync(path.join(repoRoot, 'sdkwork.app.config.json')), 'sdkwork.app.config.json must exist');
assert(fs.existsSync(path.join(repoRoot, 'sdkwork.workflow.json')), 'sdkwork.workflow.json must exist');
assert(fs.existsSync(path.join(repoRoot, 'package.json')), 'package.json must exist per PNPM_SCRIPT_SPEC.md');
assert(
  fs.existsSync(path.join(repoRoot, '.github/workflows/package.yml')),
  '.github/workflows/package.yml must exist per GITHUB_WORKFLOW_SPEC.md',
);

const packageJson = readJson('package.json');
for (const script of ['dev', 'build', 'test', 'check', 'verify', 'clean']) {
  assert(packageJson.scripts?.[script], `package.json must expose pnpm ${script}`);
}

for (const script of [
  'dev:browser',
  'dev:browser:postgres:unified-process:standalone',
  'dev:desktop',
  'dev:desktop:postgres:unified-process:standalone',
  'check:pnpm-script-standard',
  'check:agent-workflow-standard',
]) {
  assert(packageJson.scripts?.[script], `package.json must expose pnpm ${script}`);
}

for (const [scriptName, command] of Object.entries(packageJson.scripts ?? {})) {
  const firstSegment = scriptName.split(':')[0];
  assert(
    !['knowledgebase', 'kb', 'desktop', 'tauri'].includes(firstSegment),
    `package.json script ${scriptName} must use SDKWork action-first naming`,
  );
  assert(
    !/(--hosting|\bself-hosted\b|\bcloud-hosted\b)/u.test(String(command)),
    `package.json script ${scriptName} must not use retired deployment command values`,
  );
}

for (const capabilityScript of ['api:materialize:check', 'sdk:check', 'db:validate', 'topology:validate']) {
  assert(packageJson.scripts?.[capabilityScript], `package.json must expose pnpm ${capabilityScript}`);
}

const cargoToml = readText('Cargo.toml');
assert(cargoToml.includes('sdkwork-web-core'), 'Cargo.toml must declare sdkwork-web-core');
assert(cargoToml.includes('sdkwork-web-axum'), 'Cargo.toml must declare sdkwork-web-axum');
assert(cargoToml.includes('sdkwork-iam-web-adapter'), 'Cargo.toml must declare sdkwork-iam-web-adapter');
assert(cargoToml.includes('sdkwork-database-config'), 'Cargo.toml must declare sdkwork-database-config');
assert(cargoToml.includes('sdkwork-database-sqlx'), 'Cargo.toml must declare sdkwork-database-sqlx');
assert(cargoToml.includes('sdkwork-database-repository'), 'Cargo.toml must declare sdkwork-database-repository');
assert(cargoToml.includes('sdkwork-knowledgebase-api-server'), 'Cargo.toml must include sdkwork-knowledgebase-api-server');
assert(
  cargoToml.includes('sdkwork-intelligence-knowledgebase-repository-sqlx'),
  'Cargo.toml must include repository-sqlx crate',
);
assert(!cargoToml.includes('sdkwork-discovery'), 'sdkwork-discovery is not required until RPC services exist');

const workflow = readJson('sdkwork.workflow.json');
const dependencyIds = new Set((workflow.dependencies || []).map((dependency) => dependency.id));
for (const dependencyId of [
  'sdkwork-appbase',
  'sdkwork-database',
  'sdkwork-web-framework',
  'sdkwork-sdk-generator',
  'sdkwork-app-topology',
  'sdkwork-drive',
  'sdkwork-kernel',
  'sdkwork-memory',
]) {
  assert(dependencyIds.has(dependencyId), `sdkwork.workflow.json must declare ${dependencyId}`);
}
assert(!dependencyIds.has('sdkwork-discovery'), 'sdkwork.workflow.json must not declare sdkwork-discovery until RPC exists');
const targetIds = new Set((workflow.targets || []).map((target) => target.id));
for (const targetId of [
  'linux-x64-standalone-server-tar-gz',
  'web-universal-cloud-browser-zip',
]) {
  assert(targetIds.has(targetId), `sdkwork.workflow.json must declare target ${targetId}`);
}
for (const target of workflow.targets || []) {
  assert(
    ['standalone', 'cloud'].includes(target.deploymentProfile),
    `sdkwork.workflow.json target ${target.id} must declare canonical deploymentProfile`,
  );
  assert(target.runtimeTarget, `sdkwork.workflow.json target ${target.id} must declare runtimeTarget`);
  assert(target.runner, `sdkwork.workflow.json target ${target.id} must declare runner`);
  assert(
    Array.isArray(target.outputGlobs) && target.outputGlobs.length > 0,
    `sdkwork.workflow.json target ${target.id} must declare outputGlobs`,
  );
}

const topologySpec = readJson('specs/topology.spec.json');
assert(
  topologySpec.vocabulary?.deploymentProfile?.allowed?.join(',') === 'standalone,cloud',
  'specs/topology.spec.json must use deploymentProfile standalone/cloud vocabulary',
);
assert(
  topologySpec.defaults?.developmentProfileId === 'standalone.unified-process.development',
  'specs/topology.spec.json must default development to standalone.unified-process.development',
);
assert(
  topologySpec.defaults?.productionProfileId === 'cloud.split-services.production',
  'specs/topology.spec.json must default production to cloud.split-services.production',
);

const routerCrates = [
  'crates/sdkwork-router-knowledgebase-open-api/Cargo.toml',
  'crates/sdkwork-router-knowledgebase-app-api/Cargo.toml',
  'crates/sdkwork-router-knowledgebase-backend-api/Cargo.toml',
];

for (const routerCrate of routerCrates) {
  assertCargoDependsOnWebFramework(routerCrate);
  const crateName = path.basename(path.dirname(routerCrate));
  assert(
    fs.existsSync(path.join(repoRoot, `crates/${crateName}/src/web_bootstrap.rs`)),
    `${crateName} must provide web_bootstrap.rs`,
  );
}

const repositorySqlxToml = readText('crates/sdkwork-intelligence-knowledgebase-repository-sqlx/Cargo.toml');
assert(
  repositorySqlxToml.includes('sdkwork-database-sqlx'),
  'repository-sqlx crate must depend on sdkwork-database-sqlx',
);
assert(
  repositorySqlxToml.includes('sdkwork-database-repository'),
  'repository-sqlx crate must depend on sdkwork-database-repository per DATABASE_SPEC.md',
);
assert(
  repositorySqlxToml.includes('migrate'),
  'repository-sqlx sqlx dependency must enable migrate feature',
);

const repositoryBootstrap = readText(
  'crates/sdkwork-intelligence-knowledgebase-repository-sqlx/src/db/bootstrap.rs',
);
assert(
  repositoryBootstrap.includes('sdkwork_database_config'),
  'repository bootstrap must use sdkwork-database-config',
);
assert(
  repositoryBootstrap.includes('sdkwork_database_sqlx'),
  'repository bootstrap must use sdkwork-database-sqlx',
);

const componentSpec = readJson('specs/component.spec.json');
const sdkDependencyIds = new Set((componentSpec.contracts?.sdkDependencies ?? []).map((item) => item.workspace));
for (const workspace of [
  'sdkwork-web-framework',
  'sdkwork-database',
  'sdkwork-appbase',
  'sdkwork-id',
  'sdkwork-sdk-generator',
  'sdkwork-drive',
  'sdkwork-memory',
  'sdkwork-kernel',
]) {
  assert(
    sdkDependencyIds.has(workspace),
    `specs/component.spec.json must declare sdkDependencies workspace ${workspace}`,
  );
}

const routeManifestPaths = [
  'sdks/_route-manifests/open-api/sdkwork-router-knowledgebase-open-api.route-manifest.json',
  'sdks/_route-manifests/app-api/sdkwork-router-knowledgebase-app-api.route-manifest.json',
  'sdks/_route-manifests/backend-api/sdkwork-router-knowledgebase-backend-api.route-manifest.json',
];

for (const relativePath of routeManifestPaths) {
  const manifest = readJson(relativePath);
  for (const route of manifest.routes ?? []) {
    assert(
      route.requestContext === 'WebRequestContext',
      `${relativePath} route ${route.method} ${route.path} must declare WebRequestContext`,
    );
    assert(
      ['open-api', 'app-api', 'backend-api'].includes(route.apiSurface),
      `${relativePath} route ${route.method} ${route.path} must declare canonical apiSurface`,
    );
  }
}

assert(componentSpec.component.type === 'web-backend-service', 'component type must be web-backend-service');
assert(componentSpec.component.domain === 'intelligence', 'component domain must be intelligence');
assert(componentSpec.component.capability === 'knowledgebase', 'component capability must be knowledgebase');

const canonicalSpecs = (componentSpec.canonicalSpecs || []).map((entry) => entry.file);
for (const specFile of [
  'WEB_FRAMEWORK_SPEC.md',
  'WEB_BACKEND_SPEC.md',
  'DATABASE_SPEC.md',
  'DEPLOYMENT_SPEC.md',
  'API_SPEC.md',
  'SDK_SPEC.md',
]) {
  assert(canonicalSpecs.includes(specFile), `specs/component.spec.json must reference ${specFile}`);
}

const openapiPaths = [
  'sdks/sdkwork-knowledgebase-sdk/openapi/knowledgebase-open-api.openapi.json',
  'sdks/sdkwork-knowledgebase-app-sdk/openapi/knowledgebase-app-api.openapi.json',
  'sdks/sdkwork-knowledgebase-backend-sdk/openapi/knowledgebase-backend-api.openapi.json',
];

for (const relativePath of openapiPaths) {
  const openapi = readJson(relativePath);
  let hasSurface = false;
  for (const pathItem of Object.values(openapi.paths ?? {})) {
    for (const operation of Object.values(pathItem ?? {})) {
      if (operation && typeof operation === 'object' && operation.operationId) {
        assert(
          operation['x-sdkwork-request-context'] === 'WebRequestContext',
          `${relativePath} operation ${operation.operationId} must declare WebRequestContext`,
        );
        assert(
          ['open-api', 'app-api', 'backend-api'].includes(operation['x-sdkwork-api-surface']),
          `${relativePath} operation ${operation.operationId} must declare canonical x-sdkwork-api-surface`,
        );
        hasSurface = true;
      }
    }
  }
  if (!hasSurface) {
    assert(false, `${relativePath} must declare x-sdkwork-api-surface on operations`);
  }
}

const requiredSkeletonPaths = [
  'apis/README.md',
  'apis/authority-manifest.json',
  'apis/rpc/README.md',
  'deployments/README.md',
  'configs/README.md',
  'scripts/README.md',
  'apps/README.md',
  'apps/sdkwork-knowledgebase-pc/AGENTS.md',
  'specs/topology.spec.json',
];

for (const relativePath of requiredSkeletonPaths) {
  assert(
    fs.existsSync(path.join(repoRoot, relativePath)),
    `${relativePath} must exist per SDKWORK_WORKSPACE_SPEC.md skeleton`,
  );
}

if (failures.length > 0) {
  process.stderr.write(
    `Architecture alignment failed:\n${failures.map((failure) => `- ${failure}`).join('\n')}\n`,
  );
  if (warnings.length > 0) {
    process.stderr.write(
      `Warnings:\n${warnings.map((warning) => `- ${warning}`).join('\n')}\n`,
    );
  }
  process.exit(1);
}

if (warnings.length > 0) {
  process.stdout.write(
    `Architecture alignment passed with warnings:\n${warnings.map((warning) => `- ${warning}`).join('\n')}\n`,
  );
} else {
  process.stdout.write('Architecture alignment passed\n');
}
