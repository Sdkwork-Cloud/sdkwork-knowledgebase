import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const workspaceRoot = path.resolve(scriptDir, '..');
const checkOnly = process.argv.includes('--check');

const targets = [
  {
    relativePath: 'sdks/sdkwork-knowledgebase-app-sdk/openapi/knowledgebase-app-api.openapi.json',
    apiPrefix: '/app/v3/api',
    protectedAuthMode: 'dual-token',
  },
  {
    relativePath: 'sdks/sdkwork-knowledgebase-backend-sdk/openapi/knowledgebase-backend-api.openapi.json',
    apiPrefix: '/backend/v3/api',
    protectedAuthMode: 'dual-token',
  },
];

const httpMethods = new Set(['get', 'put', 'post', 'delete', 'options', 'head', 'patch', 'trace']);

function resolveAuthMode(operation) {
  const security = operation.security;
  if (!Array.isArray(security) || security.length === 0) {
    return 'anonymous';
  }
  const requirement = security[0];
  if (requirement && typeof requirement === 'object') {
    if ('ApiKey' in requirement) {
      return 'api-key';
    }
    if ('AuthToken' in requirement && 'AccessToken' in requirement) {
      return 'dual-token';
    }
  }
  return 'dual-token';
}

function applyAuthMode(document, protectedAuthMode, apiPrefix) {
  let changed = false;

  if (!Array.isArray(document.servers) || document.servers.length === 0) {
    document.servers = [{ url: apiPrefix }];
    changed = true;
  }

  for (const pathItem of Object.values(document.paths ?? {})) {
    if (!pathItem || typeof pathItem !== 'object') {
      continue;
    }
    for (const method of httpMethods) {
      const operation = pathItem[method];
      if (!operation || typeof operation !== 'object') {
        continue;
      }
      const authMode = resolveAuthMode(operation);
      const expectedMode = authMode === 'anonymous' ? 'anonymous' : protectedAuthMode;
      if (operation['x-sdkwork-auth-mode'] !== expectedMode) {
        operation['x-sdkwork-auth-mode'] = expectedMode;
        changed = true;
      }
    }
  }

  return changed;
}

let drifted = false;

for (const target of targets) {
  const filePath = path.join(workspaceRoot, target.relativePath);
  const raw = await readFile(filePath, 'utf8');
  const document = JSON.parse(raw);
  const changed = applyAuthMode(document, target.protectedAuthMode, target.apiPrefix);

  if (checkOnly) {
    if (changed) {
      drifted = true;
      console.error(`OpenAPI auth-mode drift: ${target.relativePath}`);
    }
    continue;
  }

  if (changed) {
    await writeFile(filePath, `${JSON.stringify(document, null, 2)}\n`, 'utf8');
    console.log(`Applied auth-mode metadata: ${target.relativePath}`);
  }
}

if (checkOnly && drifted) {
  process.exit(1);
}
