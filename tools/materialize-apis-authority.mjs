import { copyFile, mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { createHash } from "node:crypto";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const workspaceRoot = path.resolve(scriptDir, "..");
const checkOnly = process.argv.includes("--check");

const mappings = [
  {
    authority: "apis/open-api/knowledgebase-open-api.openapi.json",
    sdk: "sdks/sdkwork-knowledgebase-sdk/openapi/knowledgebase-open-api.openapi.json",
  },
  {
    authority: "apis/app-api/knowledgebase-app-api.openapi.json",
    sdk: "sdks/sdkwork-knowledgebase-app-sdk/openapi/knowledgebase-app-api.openapi.json",
  },
  {
    authority: "apis/backend-api/knowledgebase-backend-api.openapi.json",
    sdk: "sdks/sdkwork-knowledgebase-backend-sdk/openapi/knowledgebase-backend-api.openapi.json",
  },
];

async function sha256(filePath) {
  const content = await readFile(filePath);
  return createHash("sha256").update(content).digest("hex");
}

let drifted = false;

for (const mapping of mappings) {
  const authorityPath = path.join(workspaceRoot, mapping.authority);
  const sdkPath = path.join(workspaceRoot, mapping.sdk);
  await mkdir(path.dirname(authorityPath), { recursive: true });

  if (checkOnly) {
    const [authorityHash, sdkHash] = await Promise.all([
      sha256(authorityPath),
      sha256(sdkPath),
    ]);
    if (authorityHash !== sdkHash) {
      drifted = true;
      console.error(`API authority drift: ${mapping.authority} != ${mapping.sdk}`);
    }
    continue;
  }

  await copyFile(sdkPath, authorityPath);
  console.log(`Materialized authority ${mapping.authority}`);
}

if (checkOnly && drifted) {
  process.exit(1);
}

if (!checkOnly) {
  const manifestPath = path.join(workspaceRoot, "apis", "authority-manifest.json");
  await writeFile(
    manifestPath,
    `${JSON.stringify(
      {
        schemaVersion: 1,
        kind: "sdkwork.api.authority.manifest",
        surfaces: mappings.map((mapping) => ({
          authorityPath: mapping.authority,
          sdkPath: mapping.sdk,
        })),
      },
      null,
      2,
    )}\n`,
    "utf8",
  );
  console.log("Wrote apis/authority-manifest.json");
}
