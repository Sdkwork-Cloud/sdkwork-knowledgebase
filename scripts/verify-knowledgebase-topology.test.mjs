import assert from 'node:assert/strict';
import { readFile, stat } from 'node:fs/promises';
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
  return JSON.parse(await read(relativePath));
}

test('declares v2 topology spec and profile env files for sdkwork-knowledgebase', async () => {
  assert.equal(await exists('specs/topology.spec.json'), true);
  assert.equal(await exists('scripts/lib/knowledgebase-topology.mjs'), true);
  assert.equal(await exists('scripts/knowledgebase-dev.mjs'), true);
  assert.equal(await exists('docs/topology-standard.md'), true);

  const spec = await readJson('specs/topology.spec.json');
  assert.equal(spec.schemaVersion, 2);
  assert.equal(spec.kind, 'sdkwork.app.topology');
  assert.equal(spec.appId, 'sdkwork-knowledgebase');
  assert.equal(spec.archetype, 'application-http-gateway');
  assert.equal(spec.defaults.developmentProfileId, 'self-hosted.split-services.development');
  assert.ok(spec.surfaces['application.public-ingress']);
  assert.ok(spec.surfaces['application.backend-http']);
  assert.ok(spec.surfaces['application.open-http']);
  assert.ok(spec.surfaces['platform.api-gateway']);

  for (const profileId of [
    'self-hosted.split-services.development',
    'self-hosted.unified-process.production',
    'cloud-hosted.split-services.development',
    'cloud-hosted.split-services.production',
  ]) {
    const profilePath = spec.profileFiles[profileId];
    assert.equal(await exists(profilePath), true, `${profilePath} should exist`);
    const profileEnv = await read(profilePath);
    assert.match(profileEnv, /SDKWORK_KNOWLEDGEBASE_PROFILE_ID=/);
    assert.match(profileEnv, /VITE_SDKWORK_KNOWLEDGEBASE_APPLICATION_PUBLIC_HTTP_URL=/);
    assert.match(profileEnv, /VITE_SDKWORK_KNOWLEDGEBASE_PLATFORM_API_GATEWAY_HTTP_URL=/);
  }
});

test('root package.json wires @sdkwork/app-topology and knowledgebase:dev scripts', async () => {
  const packageJson = await readJson('package.json');
  assert.equal(packageJson.dependencies['@sdkwork/app-topology'], 'file:../sdkwork-app-topology');
  assert.match(packageJson.scripts['knowledgebase:dev'], /scripts\/knowledgebase-dev\.mjs/);
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
  assert.match(devScript, /--config/);
  assert.match(devScript, /--topology is retired/);
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
