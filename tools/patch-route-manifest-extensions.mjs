import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const workspaceRoot = path.resolve(scriptDir, "..");

const manifests = [
  {
    file: "sdks/_route-manifests/app-api/sdkwork-router-knowledgebase-app-api.route-manifest.json",
    apiSurface: "app-api",
  },
  {
    file: "sdks/_route-manifests/backend-api/sdkwork-router-knowledgebase-backend-api.route-manifest.json",
    apiSurface: "backend-api",
  },
  {
    file: "sdks/_route-manifests/open-api/sdkwork-router-knowledgebase-open-api.route-manifest.json",
    apiSurface: "open-api",
  },
];

for (const manifestSpec of manifests) {
  const filePath = path.join(workspaceRoot, manifestSpec.file);
  const manifest = JSON.parse(await readFile(filePath, "utf8"));
  manifest.routes = (manifest.routes || []).map((route) => ({
    ...route,
    requestContext: "WebRequestContext",
    apiSurface: manifestSpec.apiSurface,
  }));
  await writeFile(filePath, `${JSON.stringify(manifest, null, 2)}\n`, "utf8");
}

console.log("Patched route manifests with requestContext and apiSurface.");
