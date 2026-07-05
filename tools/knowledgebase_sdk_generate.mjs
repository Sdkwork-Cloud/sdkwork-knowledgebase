#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const workspaceRoot = path.resolve(scriptDir, "..");
const sdkgen = path.resolve(workspaceRoot, "../sdkwork-sdk-generator/bin/sdkgen.js");

const families = [
  {
    input: "sdks/sdkwork-knowledgebase-sdk/openapi/knowledgebase-open-api.openapi.json",
    output: "sdks/sdkwork-knowledgebase-sdk/sdkwork-knowledgebase-sdk-typescript/generated/server-openapi",
    name: "sdkwork-knowledgebase-sdk",
    type: "custom",
    packageName: "@sdkwork/knowledgebase-sdk",
    apiPrefix: "/knowledge/v3/api",
    clientName: "SdkworkKnowledgebaseOpenClient",
  },
  {
    input: "sdks/sdkwork-knowledgebase-app-sdk/openapi/knowledgebase-app-api.openapi.json",
    output: "sdks/sdkwork-knowledgebase-app-sdk/sdkwork-knowledgebase-app-sdk-typescript/generated/server-openapi",
    name: "sdkwork-knowledgebase-app-sdk",
    type: "app",
    packageName: "@sdkwork/knowledgebase-app-sdk",
    apiPrefix: "/app/v3/api",
    clientName: "SdkworkKnowledgebaseAppClient",
  },
  {
    input: "sdks/sdkwork-knowledgebase-backend-sdk/openapi/knowledgebase-backend-api.openapi.json",
    output: "sdks/sdkwork-knowledgebase-backend-sdk/sdkwork-knowledgebase-backend-sdk-typescript/generated/server-openapi",
    name: "sdkwork-knowledgebase-backend-sdk",
    type: "backend",
    packageName: "@sdkwork/knowledgebase-backend-sdk",
    apiPrefix: "/backend/v3/api",
    clientName: "SdkworkKnowledgebaseBackendClient",
  },
];

function runGenerate(family) {
  const args = [
    sdkgen,
    "generate",
    "-i",
    path.join(workspaceRoot, family.input),
    "-o",
    path.join(workspaceRoot, family.output),
    "-n",
    family.name,
    "-t",
    family.type,
    "-l",
    "typescript",
    "--package-name",
    family.packageName,
    "--api-prefix",
    family.apiPrefix,
    "--standard-profile",
    "sdkwork-v3",
    "--fixed-sdk-version",
    "0.1.0",
    "--client-name",
    family.clientName,
  ];

  const result = spawnSync("node", args, { stdio: "inherit", cwd: workspaceRoot });
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

for (const family of families) {
  console.log(`Generating TypeScript SDK for ${family.name}`);
  runGenerate(family);
}

console.log("Knowledgebase SDK generation completed.");
