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
    name: "sdkwork-knowledgebase-sdk",
    type: "custom",
    apiPrefix: "/knowledge/v3/api",
    targets: [
      {
        language: "typescript",
        output: "sdks/sdkwork-knowledgebase-sdk/sdkwork-knowledgebase-sdk-typescript/generated/server-openapi",
        packageName: "@sdkwork/knowledgebase-sdk",
        clientName: "SdkworkKnowledgebaseOpenClient",
      },
    ],
  },
  {
    input: "sdks/sdkwork-knowledgebase-app-sdk/openapi/knowledgebase-app-api.openapi.json",
    name: "sdkwork-knowledgebase-app-sdk",
    type: "app",
    apiPrefix: "/app/v3/api",
    targets: [
      {
        language: "typescript",
        output: "sdks/sdkwork-knowledgebase-app-sdk/sdkwork-knowledgebase-app-sdk-typescript/generated/server-openapi",
        packageName: "@sdkwork/knowledgebase-app-sdk",
        clientName: "SdkworkKnowledgebaseAppClient",
      },
    ],
  },
  {
    input: "sdks/sdkwork-knowledgebase-backend-sdk/openapi/knowledgebase-backend-api.openapi.json",
    name: "sdkwork-knowledgebase-backend-sdk",
    type: "backend",
    apiPrefix: "/backend/v3/api",
    targets: [
      {
        language: "typescript",
        output: "sdks/sdkwork-knowledgebase-backend-sdk/sdkwork-knowledgebase-backend-sdk-typescript/generated/server-openapi",
        packageName: "@sdkwork/knowledgebase-backend-sdk",
        clientName: "SdkworkKnowledgebaseBackendClient",
      },
      {
        language: "rust",
        output: "sdks/sdkwork-knowledgebase-backend-sdk/sdkwork-knowledgebase-backend-sdk-rust/generated/server-openapi",
        packageName: "sdkwork-knowledgebase-backend-sdk-generated-rust",
      },
    ],
  },
];

function runGenerate(family, target) {
  const args = [
    sdkgen,
    "generate",
    "-i",
    path.join(workspaceRoot, family.input),
    "-o",
    path.join(workspaceRoot, target.output),
    "-n",
    family.name,
    "-t",
    family.type,
    "-l",
    target.language,
    "--package-name",
    target.packageName,
    "--api-prefix",
    family.apiPrefix,
    "--standard-profile",
    "sdkwork-v3",
    "--fixed-sdk-version",
    "0.1.0",
  ];

  if (target.clientName) {
    args.push("--client-name", target.clientName);
  }

  const result = spawnSync("node", args, { stdio: "inherit", cwd: workspaceRoot });
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

for (const family of families) {
  for (const target of family.targets) {
    console.log(`Generating ${target.language} SDK for ${family.name}`);
    runGenerate(family, target);
  }
}

console.log("Knowledgebase SDK generation completed.");
