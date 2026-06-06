import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const workspaceRoot = path.resolve(scriptDir, "..");
const sdksRoot = path.resolve(workspaceRoot, "sdks");
const owner = "sdkwork-knowledgebase";
const standardVersion = "2026-06-06";
const checkOnly = process.argv.includes("--check");
const pendingChanges = [];

const httpMethods = new Set(["get", "put", "post", "delete", "options", "head", "patch", "trace"]);

const appbasePackages = {
  typescript: "@sdkwork/appbase-app-sdk",
  rust: "sdkwork-appbase-app-sdk",
  java: "com.sdkwork:sdkwork-appbase-app-sdk",
  python: "sdkwork-appbase-app-sdk",
  go: "github.com/sdkwork/sdkwork-appbase-app-sdk",
};

const appbaseBackendPackages = {
  typescript: "@sdkwork/appbase-backend-sdk",
  rust: "sdkwork-appbase-backend-sdk",
  java: "com.sdkwork:sdkwork-appbase-backend-sdk",
  python: "sdkwork-appbase-backend-sdk",
  go: "github.com/sdkwork/sdkwork-appbase-backend-sdk",
};

const driveAppPackages = {
  typescript: "@sdkwork/drive-app-sdk",
  rust: "sdkwork-drive-app-sdk",
  java: "com.sdkwork:sdkwork-drive-app-sdk",
  python: "sdkwork-drive-app-sdk",
  go: "github.com/sdkwork/sdkwork-drive-app-sdk",
};

const driveBackendPackages = {
  typescript: "@sdkwork/drive-backend-sdk",
  rust: "sdkwork-drive-backend-sdk",
  java: "com.sdkwork:sdkwork-drive-backend-sdk",
  python: "sdkwork-drive-backend-sdk",
  go: "github.com/sdkwork/sdkwork-drive-backend-sdk",
};

const families = [
  {
    root: "sdkwork-knowledgebase-app-sdk",
    title: "SDKWork Knowledgebase App API SDK",
    apiVersion: "0.1.0",
    authority: "sdkwork-knowledgebase.app",
    sdkTarget: "app",
    apiPrefix: "/app/v3/api",
    schemaUrl: "/app/v3/openapi.json",
    input: "openapi/knowledgebase-app-api.openapi.json",
    packageName: "@sdkwork/knowledgebase-app-sdk",
    generatedPath: "sdkwork-knowledgebase-app-sdk-typescript/generated/server-openapi",
    generatedWorkspace: "sdkwork-knowledgebase-app-sdk-typescript",
    primaryClient: "SdkworkAppClient",
    dependencies: [
      dependency("sdkwork-appbase-app-sdk", "appbase-identity-and-session-capability", "/app/v3/api", "sdkwork-appbase.app", appbasePackages),
      dependency("sdkwork-drive-app-sdk", "drive-file-and-media-capability", "/app/v3/api", "sdkwork-drive.app", driveAppPackages),
    ],
    forbiddenPathPrefixes: [
      "/app/v3/api/auth/",
      "/app/v3/api/iam/",
      "/app/v3/api/open_platform/",
      "/app/v3/api/system/iam/",
      "/app/v3/api/drive/",
    ],
  },
  {
    root: "sdkwork-knowledgebase-backend-sdk",
    title: "SDKWork Knowledgebase Backend API SDK",
    apiVersion: "0.1.0",
    authority: "sdkwork-knowledgebase.backend",
    sdkTarget: "backend",
    apiPrefix: "/backend/v3/api",
    schemaUrl: "/backend/v3/openapi.json",
    input: "openapi/knowledgebase-backend-api.openapi.json",
    packageName: "@sdkwork/knowledgebase-backend-sdk",
    generatedPath: "sdkwork-knowledgebase-backend-sdk-typescript/generated/server-openapi",
    generatedWorkspace: "sdkwork-knowledgebase-backend-sdk-typescript",
    primaryClient: "SdkworkBackendClient",
    dependencies: [
      dependency("sdkwork-appbase-backend-sdk", "appbase-backend-management-capability", "/backend/v3/api", "sdkwork-appbase.backend", appbaseBackendPackages),
      dependency("sdkwork-drive-backend-sdk", "drive-backend-management-capability", "/backend/v3/api", "sdkwork-drive.backend", driveBackendPackages),
    ],
    forbiddenPathPrefixes: [
      "/backend/v3/api/auth/",
      "/backend/v3/api/iam/",
      "/backend/v3/api/open_platform/",
      "/backend/v3/api/system/iam/",
      "/backend/v3/api/drive/",
    ],
  },
];

function dependency(workspace, role, apiPrefix, apiAuthority, packageByLanguage) {
  return {
    workspace,
    role,
    required: true,
    dependencyMode: "consumer-sdk",
    apiPrefix,
    apiAuthority,
    generatedTransportImportPolicy: "forbidden",
    packageByLanguage,
  };
}

function jsonText(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}

async function readJson(filePath) {
  return JSON.parse(await readFile(filePath, "utf8"));
}

async function writeJson(filePath, value) {
  const desiredText = jsonText(value);
  let currentText = "";
  let exists = true;
  try {
    currentText = await readFile(filePath, "utf8");
  } catch {
    exists = false;
  }

  if (exists && currentText === desiredText) {
    return;
  }

  const relativePath = path.relative(workspaceRoot, filePath).replaceAll("\\", "/");
  if (checkOnly) {
    pendingChanges.push(relativePath);
    return;
  }

  await mkdir(path.dirname(filePath), { recursive: true });
  await writeFile(filePath, desiredText, "utf8");
}

function operationEntries(openapi) {
  const entries = [];
  for (const [pathKey, pathItem] of Object.entries(openapi.paths || {})) {
    for (const [method, operation] of Object.entries(pathItem || {})) {
      if (!httpMethods.has(method.toLowerCase()) || !operation || typeof operation !== "object") {
        continue;
      }
      entries.push({ pathKey, method: method.toLowerCase(), operation });
    }
  }
  return entries;
}

function removeDependencyOwnedOperations(openapi, family) {
  const removed = [];
  for (const [pathKey, pathItem] of Object.entries(openapi.paths || {})) {
    if (!pathItem || typeof pathItem !== "object") {
      continue;
    }
    for (const [method, operation] of Object.entries({ ...pathItem })) {
      if (!httpMethods.has(method.toLowerCase()) || !operation || typeof operation !== "object") {
        continue;
      }
      const explicitOwner = operation["x-sdkwork-owner"];
      const forbiddenByOwner = explicitOwner && explicitOwner !== owner;
      const forbiddenByPrefix = family.forbiddenPathPrefixes.some((prefix) => pathKey.startsWith(prefix));
      if (!forbiddenByOwner && !forbiddenByPrefix) {
        continue;
      }
      removed.push({
        path: pathKey,
        method: method.toLowerCase(),
        operationId: operation.operationId || "",
        owner: explicitOwner || "",
      });
      delete pathItem[method];
    }

    const remainingMethods = Object.keys(pathItem).filter((method) => httpMethods.has(method.toLowerCase()));
    if (remainingMethods.length === 0) {
      delete openapi.paths[pathKey];
    }
  }
  return removed;
}

async function standardizeOpenApi(family) {
  const filePath = path.join(sdksRoot, family.root, family.input);
  const openapi = await readJson(filePath);
  const removedOperations = removeDependencyOwnedOperations(openapi, family);

  openapi["x-sdkwork-owner"] = owner;
  openapi["x-sdkwork-api-authority"] = family.authority;
  openapi["x-sdkwork-sdk-family"] = family.root;
  openapi["x-sdkwork-owner-only-input"] = true;
  openapi["x-sdkwork-standard-version"] = standardVersion;
  openapi.info = {
    ...(openapi.info || {}),
    title: openapi.info?.title || family.title,
    version: openapi.info?.version || family.apiVersion,
  };

  if (removedOperations.length > 0) {
    openapi["x-sdkwork-dependency-exclusions"] = [
      ...(Array.isArray(openapi["x-sdkwork-dependency-exclusions"]) ? openapi["x-sdkwork-dependency-exclusions"] : []),
      {
        standardVersion,
        reason: "dependency-owned operations are consumed through sdkDependencies and excluded from owner SDK generation input",
        removedOperations,
      },
    ];
  }

  for (const { operation } of operationEntries(openapi)) {
    operation["x-sdkwork-owner"] = owner;
    operation["x-sdkwork-api-authority"] = family.authority;
  }

  await writeJson(filePath, openapi);
  return {
    openapi,
    operationCount: operationEntries(openapi).length,
    removedOperations,
  };
}

function assemblyFor(family, openapi, operationCount) {
  return {
    workspace: family.root,
    title: family.title,
    apiVersion: family.apiVersion,
    openapiVersion: openapi.openapi || "3.1.0",
    authoritySpec: family.input,
    generationInputSpec: family.input,
    derivedSpecs: {
      default: family.input,
    },
    apiAuthority: family.authority,
    discoverySurface: {
      sdkTarget: family.sdkTarget,
      apiPrefix: family.apiPrefix,
      schemaUrl: family.schemaUrl,
      generatedProtocols: ["http-openapi"],
      manualTransports: [],
    },
    languages: [
      {
        language: "typescript",
        workspace: family.generatedWorkspace,
        generationState: "materialized",
        releaseState: "not_published",
        generatedPath: family.generatedPath,
        manifestPath: `${family.generatedPath}/package.json`,
        name: family.packageName,
        version: family.apiVersion,
        description: `Generator-owned TypeScript transport SDK for ${family.authority}.`,
        consumerSurface: {
          primaryClient: family.primaryClient,
          apiPrefix: family.apiPrefix,
        },
      },
    ],
    sdkOwner: owner,
    sdkDependencies: family.dependencies,
    metadata: {
      standardVersion,
      ownerOnlyOperationCount: operationCount,
      managedBy: "sdks/standardize-knowledgebase-sdk-family.mjs",
    },
  };
}

function componentSpecFor(family) {
  return {
    schemaVersion: 1,
    kind: "sdkwork.component.spec",
    component: {
      name: family.root,
      displayName: family.title,
      version: family.apiVersion,
      type: "sdk-family",
      root: `sdkwork-knowledgebase/sdks/${family.root}`,
      domain: "knowledgebase",
      capability: "knowledgebase",
      status: "standardized",
      languages: ["typescript"],
      generated: true,
      private: false,
      manifests: [".sdkwork-assembly.json", "sdk-manifest.json"],
    },
    canonicalSpecs: [
      {
        file: "API_SPEC.md",
        path: "../../../../../javasource/spring-ai-plus/spring-ai-plus-business/specs/API_SPEC.md",
        purpose: "HTTP/OpenAPI and generated SDK contract rules.",
      },
      {
        file: "SDK_SPEC.md",
        path: "../../../../../javasource/spring-ai-plus/spring-ai-plus-business/specs/SDK_SPEC.md",
        purpose: "SDK generation, SDK dependency, and SDK integration rules.",
      },
      {
        file: "SDK_WORKSPACE_GENERATION_SPEC.md",
        path: "../../../../../javasource/spring-ai-plus/spring-ai-plus-business/specs/SDK_WORKSPACE_GENERATION_SPEC.md",
        purpose: "SDK workspace, SDK family naming, API authority naming, and OpenAPI generation rules.",
      },
    ],
    contracts: {
      apiAuthority: {
        name: family.authority,
        prefix: family.apiPrefix,
        authorityOpenApi: family.input,
        derivedOpenApi: [family.input],
        owner,
        standard: "../../../../../javasource/spring-ai-plus/spring-ai-plus-business/specs/SDK_WORKSPACE_GENERATION_SPEC.md",
      },
      publicExports: [],
      runtimeEntrypoints: [".sdkwork-assembly.json"],
      sdkDependencies: family.dependencies,
      sdkClients: [family.primaryClient],
      events: [],
      configKeys: [".sdkwork-assembly.json", "sdk-manifest.json"],
    },
    integration: {
      authority: "Root SDKWork specs remain authoritative. Local specs may extend but must not contradict them.",
      dependencyPolicy: "Appbase and drive capabilities are consumed through declared dependency SDKs, not copied into generated knowledgebase transports.",
      sdkPolicy: "Generated SDK clients are injected through service/runtime boundaries; consumers must not create raw HTTP clients or manual auth headers.",
      languagePolicy: "TypeScript is the current generated package for this SDK family; additional languages must use the same owner-only OpenAPI input and sdkDependencies.",
    },
    verification: {
      commands: [
        "node sdks/standardize-knowledgebase-sdk-family.mjs --check",
        "node sdks/test/verify-sdk-ownership-boundaries.test.mjs",
        "powershell -ExecutionPolicy Bypass -File tools/verify_openapi_operation_ids.ps1",
      ],
    },
    metadata: {
      managedBy: "sdks/standardize-knowledgebase-sdk-family.mjs",
      standardVersion,
    },
  };
}

function sdkManifestFor(family, operationCount) {
  return {
    schemaVersion: 1,
    sdkName: family.root,
    packageName: family.packageName,
    sdkOwner: owner,
    apiAuthority: family.authority,
    sdkFamily: family.root,
    sdkType: family.sdkTarget,
    sdkSurface: family.sdkTarget,
    language: "typescript",
    apiPrefix: family.apiPrefix,
    generationInputSpec: family.input,
    generatedOutput: family.generatedPath,
    standardProfile: "sdkwork-v3",
    sdkDependencies: family.dependencies,
    ownerOnlyOperationCount: operationCount,
    standardVersion,
    managedBy: "sdks/standardize-knowledgebase-sdk-family.mjs",
  };
}

async function standardizeFamily(family) {
  const { openapi, operationCount, removedOperations } = await standardizeOpenApi(family);
  await writeJson(path.join(sdksRoot, family.root, ".sdkwork-assembly.json"), assemblyFor(family, openapi, operationCount));
  await writeJson(path.join(sdksRoot, family.root, "sdk-manifest.json"), sdkManifestFor(family, operationCount));
  await writeJson(path.join(sdksRoot, family.root, "specs", "component.spec.json"), componentSpecFor(family));
  return {
    family: family.root,
    authority: family.authority,
    operationCount,
    removedDependencyOperations: removedOperations.length,
  };
}

const summary = [];
for (const family of families) {
  summary.push(await standardizeFamily(family));
}

if (checkOnly && pendingChanges.length > 0) {
  console.error(JSON.stringify({ ok: false, mode: "check", owner, standardVersion, pendingChanges, families: summary }, null, 2));
  process.exit(1);
}

console.log(JSON.stringify({ ok: true, mode: checkOnly ? "check" : "apply", owner, standardVersion, families: summary }, null, 2));
