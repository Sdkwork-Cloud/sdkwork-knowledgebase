import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const workspaceRoot = path.resolve(scriptDir, '..');

const targets = [
  {
    manifestRs: 'crates/sdkwork-routes-knowledgebase-app-api/src/manifest.rs',
    jsonFile: 'sdks/_route-manifests/app-api/sdkwork-routes-knowledgebase-app-api.route-manifest.json',
    apiSurface: 'app-api',
  },
  {
    manifestRs: 'crates/sdkwork-routes-knowledgebase-backend-api/src/manifest.rs',
    jsonFile: 'sdks/_route-manifests/backend-api/sdkwork-routes-knowledgebase-backend-api.route-manifest.json',
    apiSurface: 'backend-api',
  },
  {
    manifestRs: 'crates/sdkwork-routes-knowledgebase-open-api/src/manifest.rs',
    jsonFile: 'sdks/_route-manifests/open-api/sdkwork-routes-knowledgebase-open-api.route-manifest.json',
    apiSurface: 'open-api',
  },
];

function parseRoutes(manifestSource) {
  const routes = [];
  const entryPattern =
    /method:\s*"([A-Z]+)"[\s\S]*?path:\s*"([^"]+)"[\s\S]*?operation_id:\s*"([^"]+)"/g;
  for (const match of manifestSource.matchAll(entryPattern)) {
    routes.push({
      method: match[1],
      path: match[2],
      operationId: match[3],
    });
  }
  return routes;
}

function buildRoute(route, manifest, apiSurface) {
  return {
    method: route.method,
    path: route.path,
    operationId: route.operationId,
    tags: ['knowledge'],
    auth: {
      mode: apiSurface === 'open-api' ? 'bearer' : 'dual-token',
      required: true,
    },
    handler: {
      module: 'crate::routes',
      name: null,
    },
    ownership: {
      owner: manifest.owner,
      apiAuthority: manifest.apiAuthority,
    },
    requestContext: 'WebRequestContext',
    apiSurface,
  };
}

for (const target of targets) {
  const manifestSource = await readFile(path.join(workspaceRoot, target.manifestRs), 'utf8');
  const jsonPath = path.join(workspaceRoot, target.jsonFile);
  const manifest = JSON.parse(await readFile(jsonPath, 'utf8'));
  const routes = parseRoutes(manifestSource);
  if (routes.length === 0) {
    throw new Error(`No routes parsed from ${target.manifestRs}`);
  }
  manifest.routes = routes.map((route) => buildRoute(route, manifest, target.apiSurface));
  await writeFile(jsonPath, `${JSON.stringify(manifest, null, 2)}\n`, 'utf8');
  console.log(`Synced ${routes.length} routes to ${target.jsonFile}`);
}
