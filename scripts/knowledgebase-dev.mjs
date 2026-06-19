#!/usr/bin/env node

import { spawn, spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import {
  API_GATEWAY_REPO,
  DEFAULT_DEV_PROFILE_ID,
  listHealthSurfaces,
  listOrchestrationProcesses,
  loadProfile,
  mergeRuntimeEnv,
  REPO_ROOT,
  resolveCloudGatewayConfigPath,
  resolveDevProfileId,
  resolveGatewayBind,
  resolveIamDevEnv,
  resolveSurfaceBind,
  resolveSurfaceHttpUrl,
  shouldAutostartGateway,
  waitForHttpHealthy,
} from './lib/knowledgebase-topology.mjs';

const HEALTH_PATH = '/healthz';
const HEALTH_TIMEOUT_MS = 2000;
const STARTUP_WAIT_MS = 500;
const MAX_STARTUP_ATTEMPTS = 60;

function cargoCommand() {
  return process.platform === 'win32' ? 'cargo.exe' : 'cargo';
}

const PC_APP_ROOT = path.join(REPO_ROOT, 'apps/sdkwork-knowledgebase-pc');
const DESKTOP_ROOT = path.join(PC_APP_ROOT, 'packages/sdkwork-knowledgebase-pc-desktop');

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
    hosting: 'self-hosted',
    serviceLayout: 'split-services',
    target: 'backend',
    dryRun: false,
    help: false,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--help' || arg === '-h') {
      settings.help = true;
      continue;
    }
    if (arg === '--hosting') {
      settings.hosting = argv[index + 1] ?? settings.hosting;
      index += 1;
      continue;
    }
    if (arg === '--service-layout') {
      settings.serviceLayout = argv[index + 1] ?? settings.serviceLayout;
      index += 1;
      continue;
    }
    if (arg === '--target') {
      settings.target = argv[index + 1] ?? settings.target;
      index += 1;
      continue;
    }
    if (arg === '--topology') {
      throw new Error(
        '--topology is retired; use --hosting (standalone -> self-hosted, cloud -> cloud-hosted)',
      );
    }
    if (arg === '--dry-run') {
      settings.dryRun = true;
    }
  }

  return settings;
}

function printHelp() {
  console.log(`Usage: node scripts/knowledgebase-dev.mjs [options]

Topology-aware Knowledgebase dev entry. Loads configs/topology profile env via @sdkwork/app-topology.

Options:
  --hosting <self-hosted|cloud-hosted>              Default: self-hosted
  --service-layout <split-services|unified-process> Default: split-services
  --target <backend|desktop>                        Default: backend
  --dry-run                                         Print plan without executing
  --help, -h
`);
}

function createDesktopProcess(env) {
  const desktopEnv = sanitizeSpawnEnv({
    ...env,
    VITE_SDKWORK_KNOWLEDGEBASE_DEV_SAME_ORIGIN_API:
      env.VITE_SDKWORK_KNOWLEDGEBASE_DEV_SAME_ORIGIN_API ?? 'true',
    VITE_SDKWORK_APPBASE_APP_API_BASE_URL:
      env.VITE_SDKWORK_APPBASE_APP_API_BASE_URL ?? 'http://127.0.0.1:3900',
    VITE_SDKWORK_IAM_APP_API_BASE_URL:
      env.VITE_SDKWORK_IAM_APP_API_BASE_URL ?? 'http://127.0.0.1:3900',
    VITE_SDKWORK_KNOWLEDGEBASE_PLATFORM_API_GATEWAY_HTTP_URL:
      env.VITE_SDKWORK_KNOWLEDGEBASE_PLATFORM_API_GATEWAY_HTTP_URL ?? 'http://127.0.0.1:3900',
  });

  return {
    label: 'sdkwork-knowledgebase-pc-desktop',
    command: pnpmCommand(),
    args: ['run', 'desktop:dev'],
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
  const dataDir = path.join(REPO_ROOT, '.sdkwork', 'knowledgebase');
  if (!fs.existsSync(dataDir)) {
    fs.mkdirSync(dataDir, { recursive: true });
  }
}

const DEFAULT_API_SERVER_CRATE = 'sdkwork-knowledgebase-api-server';

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
  const hosting = env.SDKWORK_KNOWLEDGEBASE_HOSTING ?? 'self-hosted';
  const bind =
    resolveSurfaceBind(env, 'platform.api-gateway') ?? resolveGatewayBind(env, hosting);
  const gatewayConfig = resolveCloudGatewayConfigPath(env, 'development');
  const args = [
    'run',
    '-p',
    'sdkwork-api-gateway-api-server',
    '--bin',
    'sdkwork-api-gateway',
    '--',
    '--config',
    gatewayConfig,
  ];

  return {
    label: 'sdkwork-api-gateway',
    command: cargoCommand(),
    args,
    cwd: API_GATEWAY_REPO,
    env: {
      ...env,
      SDKWORK_API_GATEWAY_BIND: bind,
      SDKWORK_API_GATEWAY_CONFIG: gatewayConfig,
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
    resolveDevProfileId(settings.hosting, settings.serviceLayout) || DEFAULT_DEV_PROFILE_ID;
  const profileEnv = loadProfile(profileId);
  const runtimeEnv = mergeRuntimeEnv(
    process.env,
    profileEnv,
    resolveIamDevEnv(process.env),
    {
      SDKWORK_KNOWLEDGEBASE_PROFILE_ID: profileId,
      SDKWORK_KNOWLEDGEBASE_DEV_MODE: '1',
      ...(settings.target === 'desktop'
        ? { SDKWORK_KNOWLEDGEBASE_PLATFORM_API_GATEWAY_AUTOSTART: 'true' }
        : {}),
    },
  );

  const backendProcesses = buildProcessesFromOrchestration(profileId, runtimeEnv);
  const processes =
    settings.target === 'desktop'
      ? [...backendProcesses, createDesktopProcess(runtimeEnv)]
      : backendProcesses;

  if (settings.dryRun) {
    console.log(`[sdkwork-knowledgebase] profile=${profileId} target=${settings.target}`);
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
