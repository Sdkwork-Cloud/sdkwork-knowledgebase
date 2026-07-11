#!/usr/bin/env node

import { spawn, spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';

import {
  API_GATEWAY_REPO,
  DEFAULT_DEV_PROFILE_ID,
  listHealthSurfaces,
  listOrchestrationProcesses,
  loadEnvFile,
  loadProfile,
  mergeRuntimeEnv,
  normalizeText,
  REPO_ROOT,
  resolveCloudGatewayConfigPath,
  resolveDevProfileId,
  IAM_APPLICATION_BOOTSTRAP_ENV,
  resolveGatewayBind,
  resolveIamDevEnv,
  resolveSurfaceBind,
  resolveSurfaceHttpUrl,
  shouldAutostartGateway,
  waitForHttpHealthy,
} from './lib/knowledgebase-topology.mjs';
import { mergeRepoDevBootstrapAccessTokenEnv, readApplicationManifest, resolveRepoApplicationManifestPath } from './lib/knowledgebase-dev-bootstrap-access-token-env.mjs';

const HEALTH_PATH = '/healthz';
const HEALTH_TIMEOUT_MS = 2000;
const STARTUP_WAIT_MS = 500;
const MAX_STARTUP_ATTEMPTS = 60;

const PC_APP_ROOT = path.join(REPO_ROOT, 'apps/sdkwork-knowledgebase-pc');
const DESKTOP_ROOT = path.join(PC_APP_ROOT, 'packages/sdkwork-knowledgebase-pc-desktop');
const DEFAULT_API_SERVER_CRATE = 'sdkwork-knowledgebase-standalone-gateway';

function cargoCommand() {
  return process.platform === 'win32' ? 'cargo.exe' : 'cargo';
}

function pnpmCommand() {
  return process.platform === 'win32' ? 'pnpm.cmd' : 'pnpm';
}

function pnpmShell() {
  return process.platform === 'win32';
}

function sanitizeSpawnEnv(env) {
  const sanitized = { ...process.env };
  for (const [key, value] of Object.entries(env ?? {})) {
    if (value === undefined) {
      continue;
    }
    sanitized[key] = String(value);
  }
  return sanitized;
}

function parseArgs(argv) {
  const settings = {
    database: 'postgres',
    deploymentProfile: 'standalone',
    devEnvFile: undefined,
    dryRun: false,
    help: false,
    target: 'browser',
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--help' || arg === '-h') {
      settings.help = true;
      continue;
    }
    if (arg === '--deployment-profile') {
      settings.deploymentProfile = argv[index + 1] ?? settings.deploymentProfile;
      index += 1;
      continue;
    }
    if (arg === '--database') {
      settings.database = argv[index + 1] ?? settings.database;
      index += 1;
      continue;
    }
    if (arg === '--target') {
      settings.target = argv[index + 1] ?? settings.target;
      index += 1;
      continue;
    }
    if (arg === '--dev-env-file') {
      settings.devEnvFile = argv[index + 1];
      index += 1;
      continue;
    }
    if (arg === '--dry-run') {
      settings.dryRun = true;
      continue;
    }
    throw new Error(`Unsupported option: ${arg}`);
  }

  if (!['browser', 'desktop'].includes(settings.target)) {
    throw new Error('target must be one of: browser, desktop');
  }
  if (!['postgres', 'sqlite'].includes(settings.database)) {
    throw new Error('database must be one of: postgres, sqlite');
  }

  return settings;
}

function printHelp() {
  console.log(`Usage: node scripts/knowledgebase-dev.mjs [options]

Topology-aware Knowledgebase dev entry. Loads configs/topology profile env via @sdkwork/app-topology.

Database profiles:
  postgres (default)  IAM/login and Knowledgebase HTTP handlers share PostgreSQL from .env.postgres.
  sqlite              Portable SQLite knowledge metadata profile without loading .env.postgres.

Options:
  --deployment-profile <standalone|cloud>           Default: standalone
  --database <postgres|sqlite>                      Default: postgres
  --target <browser|desktop>                        Default: browser
  --dev-env-file <path>                             Optional PostgreSQL override for IAM/login
  --dry-run                                         Print plan without executing
  --help, -h
`);
}

function resolvePostgresDevEnvFile(settings) {
  if (settings.devEnvFile) {
    return settings.devEnvFile;
  }
  return fs.existsSync(path.join(REPO_ROOT, '.env.postgres')) ? '.env.postgres' : '.env.postgres.example';
}

function resolveDefaultSqliteDatabaseUrl() {
  ensureKnowledgebaseDataDir();
  const sqliteFile = path.join(REPO_ROOT, '.sdkwork', 'runtime', 'knowledgebase', 'knowledgebase.sqlite');
  return `sqlite:///${sqliteFile.split(path.sep).join('/')}?mode=rwc`;
}

function resolveKnowledgebaseRuntimeTenantEnv(env = {}) {
  const manifest = readApplicationManifest(resolveRepoApplicationManifestPath(REPO_ROOT));
  return {
    SDKWORK_KNOWLEDGEBASE_TENANT_ID:
      normalizeText(env.SDKWORK_KNOWLEDGEBASE_TENANT_ID)
      ?? normalizeText(manifest?.backend?.tenantId)
      ?? '100001',
    SDKWORK_KNOWLEDGEBASE_ORGANIZATION_ID:
      normalizeText(env.SDKWORK_KNOWLEDGEBASE_ORGANIZATION_ID)
      ?? normalizeText(manifest?.backend?.organizationId)
      ?? '0',
  };
}

function materializePostgresDatabaseSearchPath(env, urlKey, servicePrefix) {
  const databaseUrl = normalizeText(env[urlKey]);
  if (!databaseUrl || !/^postgres(?:ql)?:\/\//i.test(databaseUrl)) {
    return env;
  }
  if (/[?&]options=/i.test(databaseUrl)) {
    return env;
  }

  const schema =
    normalizeText(env[`${servicePrefix}_DATABASE_SCHEMA`])
    || normalizeText(env.SDKWORK_CLAW_DATABASE_SCHEMA)
    || 'sdkwork_ai_dev';
  const optionValue = encodeURIComponent(`-c search_path=${schema},public`);
  const separator = databaseUrl.includes('?') ? '&' : '?';
  return {
    ...env,
    [urlKey]: `${databaseUrl}${separator}options=${optionValue}`,
  };
}

function materializeIamDatabaseSearchPath(env) {
  return materializePostgresDatabaseSearchPath(env, 'SDKWORK_IAM_DATABASE_URL', 'SDKWORK_IAM');
}

function materializeKnowledgebaseDevDatabaseEnv(env) {
  return materializePostgresDatabaseSearchPath(
    materializeIamDatabaseSearchPath(env),
    'SDKWORK_KNOWLEDGEBASE_DATABASE_URL',
    'SDKWORK_KNOWLEDGEBASE',
  );
}

function resolvePostgresKnowledgebaseDatabaseUrl(sourceEnv) {
  const direct =
    normalizeText(sourceEnv.SDKWORK_KNOWLEDGEBASE_DATABASE_URL)
    || normalizeText(sourceEnv.SDKWORK_IAM_DATABASE_URL)
    || normalizeText(sourceEnv.SDKWORK_CLAW_DATABASE_URL)
    || normalizeText(sourceEnv.SDKWORK_DATABASE_URL);
  if (!direct) {
    throw new Error(
      'PostgreSQL dev profile requires SDKWORK_CLAW_DATABASE_* in .env.postgres or SDKWORK_KNOWLEDGEBASE_DATABASE_URL.',
    );
  }
  if (!/^postgres(?:ql)?:\/\//i.test(direct)) {
    throw new Error(`Knowledgebase PostgreSQL dev profile requires a postgres URL, got: ${direct}`);
  }
  return direct;
}

function resolveKnowledgebaseAppDatabaseEnv() {
  return {
    SDKWORK_KNOWLEDGEBASE_DATABASE_ENGINE: 'sqlite',
    SDKWORK_KNOWLEDGEBASE_DATABASE_FILE: './.sdkwork/runtime/knowledgebase/knowledgebase.sqlite',
    SDKWORK_KNOWLEDGEBASE_DATABASE_URL: resolveDefaultSqliteDatabaseUrl(),
    SDKWORK_KNOWLEDGEBASE_DATABASE_MAX_CONNECTIONS: '1',
  };
}

function databaseEnv(settings, sourceEnv = {}) {
  if (settings.database === 'postgres') {
    const databaseUrl = resolvePostgresKnowledgebaseDatabaseUrl(sourceEnv);
    return {
      SDKWORK_KNOWLEDGEBASE_DATABASE_ENGINE: 'postgresql',
      SDKWORK_KNOWLEDGEBASE_DATABASE_URL: databaseUrl,
      SDKWORK_KNOWLEDGEBASE_DATABASE_MAX_CONNECTIONS:
        sourceEnv.SDKWORK_KNOWLEDGEBASE_DATABASE_MAX_CONNECTIONS
        || sourceEnv.SDKWORK_CLAW_DATABASE_MAX_CONNECTIONS
        || '10',
    };
  }
  return resolveKnowledgebaseAppDatabaseEnv();
}

function createBrowserRendererProcess(env) {
  const browserEnv = sanitizeSpawnEnv({
    ...env,
    SDKWORK_KNOWLEDGEBASE_RUNTIME_TARGET: 'browser',
    VITE_SDKWORK_KNOWLEDGEBASE_RUNTIME_TARGET: 'browser',
    VITE_SDKWORK_KNOWLEDGEBASE_DEV_SAME_ORIGIN_API:
      env.VITE_SDKWORK_KNOWLEDGEBASE_DEV_SAME_ORIGIN_API ?? 'true',
    VITE_SDKWORK_APPBASE_APP_API_BASE_URL:
      env.VITE_SDKWORK_APPBASE_APP_API_BASE_URL ?? 'http://127.0.0.1:18081',
    VITE_SDKWORK_IAM_APP_API_BASE_URL:
      env.VITE_SDKWORK_IAM_APP_API_BASE_URL ?? 'http://127.0.0.1:18081',
    VITE_SDKWORK_KNOWLEDGEBASE_PLATFORM_API_GATEWAY_HTTP_URL:
      env.VITE_SDKWORK_KNOWLEDGEBASE_PLATFORM_API_GATEWAY_HTTP_URL ?? 'http://127.0.0.1:3900',
  });

  return {
    label: 'sdkwork-knowledgebase-pc-browser',
    command: pnpmCommand(),
    args: ['run', 'dev'],
    cwd: PC_APP_ROOT,
    env: browserEnv,
    shell: pnpmShell(),
  };
}

function createDesktopProcess(env) {
  const desktopEnv = sanitizeSpawnEnv({
    ...env,
    SDKWORK_KNOWLEDGEBASE_RUNTIME_TARGET: 'desktop',
    VITE_SDKWORK_KNOWLEDGEBASE_RUNTIME_TARGET: 'desktop',
    VITE_SDKWORK_KNOWLEDGEBASE_DEV_SAME_ORIGIN_API:
      env.VITE_SDKWORK_KNOWLEDGEBASE_DEV_SAME_ORIGIN_API ?? 'true',
    VITE_SDKWORK_APPBASE_APP_API_BASE_URL:
      env.VITE_SDKWORK_APPBASE_APP_API_BASE_URL ?? 'http://127.0.0.1:18081',
    VITE_SDKWORK_IAM_APP_API_BASE_URL:
      env.VITE_SDKWORK_IAM_APP_API_BASE_URL ?? 'http://127.0.0.1:18081',
    VITE_SDKWORK_KNOWLEDGEBASE_PLATFORM_API_GATEWAY_HTTP_URL:
      env.VITE_SDKWORK_KNOWLEDGEBASE_PLATFORM_API_GATEWAY_HTTP_URL ?? 'http://127.0.0.1:3900',
  });

  return {
    label: 'sdkwork-knowledgebase-pc-desktop',
    command: pnpmCommand(),
    args: ['run', 'dev:desktop'],
    cwd: DESKTOP_ROOT,
    env: desktopEnv,
    shell: pnpmShell(),
  };
}

function spawnProcessEntry(entry) {
  return spawn(entry.command, entry.args, {
    cwd: entry.cwd ?? REPO_ROOT,
    env: sanitizeSpawnEnv(entry.env),
    stdio: 'inherit',
    shell: entry.shell ?? false,
    windowsHide: true,
  });
}

function terminateProcessTree(child) {
  if (!child?.pid) {
    return;
  }
  if (process.platform === 'win32') {
    spawnSync('taskkill.exe', ['/PID', String(child.pid), '/T', '/F'], {
      stdio: 'ignore',
      windowsHide: true,
    });
    return;
  }
  child.kill();
}

function ensureKnowledgebaseDataDir() {
  const dataDir = path.join(REPO_ROOT, '.sdkwork', 'runtime', 'knowledgebase');
  if (!fs.existsSync(dataDir)) {
    fs.mkdirSync(dataDir, { recursive: true });
  }
}

function createApiServerBinaryProcess(crate, binary, label, env) {
  ensureKnowledgebaseDataDir();
  return {
    label,
    command: cargoCommand(),
    args: ['run', '-p', crate, '--bin', binary],
    cwd: REPO_ROOT,
    env,
  };
}

function createPlatformGatewayProcess(env) {
  const deploymentProfile = env.SDKWORK_KNOWLEDGEBASE_DEPLOYMENT_PROFILE ?? 'cloud';
  const bind =
    resolveSurfaceBind(env, 'platform.api-gateway') ?? resolveGatewayBind(env, deploymentProfile);
  const gatewayConfig = resolveCloudGatewayConfigPath(env, env.SDKWORK_KNOWLEDGEBASE_ENVIRONMENT ?? 'development');
  const args = [
    'run',
    '-p',
    'sdkwork-api-cloud-gateway',
    '--bin',
    'sdkwork-api-cloud-gateway',
    '--',
    '--config',
    gatewayConfig,
  ];

  return {
    label: 'sdkwork-api-cloud-gateway',
    command: cargoCommand(),
    args,
    cwd: API_GATEWAY_REPO,
    env: {
      ...env,
      ...IAM_APPLICATION_BOOTSTRAP_ENV,
      SDKWORK_API_CLOUD_GATEWAY_BIND: bind,
      SDKWORK_API_CLOUD_GATEWAY_CONFIG: gatewayConfig,
    },
  };
}

function buildProcessesFromOrchestration(profileId, env) {
  const processes = [];
  let gatewayScheduled = false;

  for (const processDef of listOrchestrationProcesses(profileId)) {
    if (processDef.id === 'platform.api-gateway') {
      gatewayScheduled = true;
      if (!shouldAutostartGateway(env)) {
        continue;
      }
      processes.push(createPlatformGatewayProcess(env));
      continue;
    }

    const crate = processDef.crate ?? DEFAULT_API_SERVER_CRATE;
    const binary = processDef.binary ?? processDef.id;
    processes.push(createApiServerBinaryProcess(crate, binary, binary, env));
  }

  if (!gatewayScheduled && shouldAutostartGateway(env)) {
    processes.unshift(createPlatformGatewayProcess(env));
  }

  return processes;
}

async function waitForSurfaceHealth(profileId, env) {
  const surfaces = [...listHealthSurfaces(profileId)];
  if (shouldAutostartGateway(env) && !surfaces.includes('platform.api-gateway')) {
    surfaces.unshift('platform.api-gateway');
  }
  for (const surfaceId of surfaces) {
    const url = resolveSurfaceHttpUrl(env, surfaceId);
    if (!url) {
      continue;
    }
    const healthUrl = `${url.replace(/\/+$/u, '')}${HEALTH_PATH}`;
    let ready = false;
    for (let attempt = 0; attempt < MAX_STARTUP_ATTEMPTS; attempt += 1) {
      ready = await waitForHttpHealthy(healthUrl, HEALTH_TIMEOUT_MS);
      if (ready) {
        console.log(`[sdkwork-knowledgebase] healthy ${surfaceId} (${healthUrl})`);
        break;
      }
      await new Promise((resolve) => setTimeout(resolve, STARTUP_WAIT_MS));
    }
    if (!ready) {
      throw new Error(`timed out waiting for ${surfaceId} health at ${healthUrl}`);
    }
  }
}

async function main() {
  const settings = parseArgs(process.argv.slice(2));
  if (settings.help) {
    printHelp();
    process.exit(0);
  }

  const profileId =
    resolveDevProfileId(settings.deploymentProfile) || DEFAULT_DEV_PROFILE_ID;
  const profileEnv = loadProfile(profileId);
  const postgresDevEnv =
    settings.database === 'postgres' ? loadEnvFile(resolvePostgresDevEnvFile(settings)) : {};
  const iamSourceEnv = mergeRuntimeEnv(process.env, profileEnv, postgresDevEnv);
  const iamResolvedEnv = resolveIamDevEnv(iamSourceEnv);
  const runtimeEnv = materializeKnowledgebaseDevDatabaseEnv(mergeRepoDevBootstrapAccessTokenEnv({
    repoRoot: REPO_ROOT,
    appId: 'sdkwork-knowledgebase-pc',
    env: mergeRuntimeEnv(
      iamSourceEnv,
      iamResolvedEnv,
      databaseEnv(settings, iamResolvedEnv),
      IAM_APPLICATION_BOOTSTRAP_ENV,
      resolveKnowledgebaseRuntimeTenantEnv(iamSourceEnv),
      {
        SDKWORK_KNOWLEDGEBASE_DEPLOYMENT_PROFILE: settings.deploymentProfile,
        SDKWORK_KNOWLEDGEBASE_DATABASE_PROFILE: settings.database,
        SDKWORK_KNOWLEDGEBASE_PROFILE_ID: profileId,
        SDKWORK_KNOWLEDGEBASE_DEV_MODE: '1',
        SDKWORK_KNOWLEDGEBASE_RUNTIME_TARGET: settings.target === 'desktop' ? 'desktop' : 'browser',
        VITE_SDKWORK_KNOWLEDGEBASE_DEPLOYMENT_PROFILE: settings.deploymentProfile,
        VITE_SDKWORK_KNOWLEDGEBASE_RUNTIME_TARGET: settings.target === 'desktop' ? 'desktop' : 'browser',
        ...((settings.target === 'browser' || settings.target === 'desktop') && settings.deploymentProfile === 'cloud'
          ? { SDKWORK_KNOWLEDGEBASE_PLATFORM_API_GATEWAY_AUTOSTART: 'true' }
          : {}),
        ...(settings.target === 'desktop'
          ? { SDKWORK_KNOWLEDGEBASE_RUNTIME_TARGET: 'desktop' }
          : {}),
      },
    ),
  }));

  const backendProcesses = buildProcessesFromOrchestration(profileId, runtimeEnv);
  const processes =
    settings.target === 'desktop'
      ? [...backendProcesses, createDesktopProcess(runtimeEnv)]
      : backendProcesses;

  if (settings.dryRun) {
    console.log(
      `[sdkwork-knowledgebase] profile=${profileId} deploymentProfile=${settings.deploymentProfile} database=${settings.database} target=${settings.target} knowledgeDatabase=${runtimeEnv.SDKWORK_KNOWLEDGEBASE_DATABASE_URL} iamDatabase=${runtimeEnv.SDKWORK_IAM_DATABASE_URL ?? runtimeEnv.SDKWORK_CLAW_DATABASE_URL ?? 'unknown'}`,
    );
    for (const entry of processes) {
      console.log(`[${entry.label}] ${entry.command} ${entry.args.join(' ')}`);
    }
    process.exit(0);
  }

  const children = [];
  let shuttingDown = false;

  function shutdown(exceptChild) {
    if (shuttingDown) {
      return;
    }
    shuttingDown = true;
    for (const child of children) {
      if (child !== exceptChild && child.exitCode == null && child.signalCode == null) {
        terminateProcessTree(child);
      }
    }
  }

  function attachProcessLifecycle(entry, child) {
    child.on('error', (error) => {
      process.stderr.write(
        `[${entry.label}] ${error instanceof Error ? error.message : String(error)}\n`,
      );
      shutdown(child);
      process.exitCode = 1;
    });
    child.on('exit', (code, signal) => {
      if (shuttingDown) {
        return;
      }
      shutdown(child);
      if (code && code !== 0) {
        process.stderr.write(`[${entry.label}] exited with code ${code}\n`);
        process.exitCode = code;
        return;
      }
      if (signal) {
        process.stderr.write(`[${entry.label}] exited with signal ${signal}\n`);
        process.exitCode = 1;
      }
    });
  }

  for (const entry of backendProcesses) {
    const child = spawnProcessEntry(entry);
    children.push(child);
    attachProcessLifecycle(entry, child);
  }

  try {
    await waitForSurfaceHealth(profileId, runtimeEnv);
  } catch (error) {
    shutdown();
    throw error;
  }

  console.log(`[sdkwork-knowledgebase] dev stack ready (profile=${profileId})`);

  const stop = () => shutdown();
  process.once('SIGINT', stop);
  process.once('SIGTERM', stop);

  if (settings.target === 'browser') {
    const browserEntry = createBrowserRendererProcess(runtimeEnv);
    console.log('[sdkwork-knowledgebase] browser renderer starting (Vite on :5184)');
    const browserChild = spawnProcessEntry(browserEntry);
    children.push(browserChild);
    attachProcessLifecycle(browserEntry, browserChild);
    return;
  }

  if (settings.target !== 'desktop') {
    return;
  }

  const desktopEntry = createDesktopProcess(runtimeEnv);
  console.log('[sdkwork-knowledgebase] desktop renderer starting (Tauri + Vite on :5184)');
  const desktopChild = spawnProcessEntry(desktopEntry);
  children.push(desktopChild);

  await new Promise((resolve, reject) => {
    desktopChild.on('error', reject);
    desktopChild.on('exit', (code, signal) => {
      shutdown(desktopChild);
      if (code === 0 || signal === 'SIGINT' || signal === 'SIGTERM') {
        resolve();
        return;
      }
      reject(new Error(`desktop renderer exited with code ${code ?? 1}`));
    });
  });
}

main().catch((error) => {
  console.error(
    `[sdkwork-knowledgebase] ${error instanceof Error ? error.message : String(error)}`,
  );
  process.exit(1);
});
