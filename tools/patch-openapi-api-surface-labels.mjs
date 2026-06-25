import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const workspaceRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const files = [
  "sdks/sdkwork-knowledgebase-app-sdk/openapi/knowledgebase-app-api.openapi.json",
  "sdks/sdkwork-knowledgebase-backend-sdk/openapi/knowledgebase-backend-api.openapi.json",
  "sdks/sdkwork-knowledgebase-sdk/openapi/knowledgebase-open-api.openapi.json",
  "apis/app-api/knowledgebase-app-api.openapi.json",
  "apis/backend-api/knowledgebase-backend-api.openapi.json",
  "apis/open-api/knowledgebase-open-api.openapi.json",
];

const replacements = [
  ['"x-sdkwork-api-surface": "app"', '"x-sdkwork-api-surface": "app-api"'],
  ['"x-sdkwork-api-surface": "backend"', '"x-sdkwork-api-surface": "backend-api"'],
  ['"x-sdkwork-api-surface": "open"', '"x-sdkwork-api-surface": "open-api"'],
];

for (const relativePath of files) {
  const filePath = path.join(workspaceRoot, relativePath);
  let text = await readFile(filePath, "utf8");
  for (const [from, to] of replacements) {
    text = text.replaceAll(from, to);
  }
  await writeFile(filePath, text, "utf8");
}

console.log("Patched canonical x-sdkwork-api-surface labels.");
