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

const memoryAppPackages = {
  typescript: "@sdkwork/memory-app-sdk",
  rust: "sdkwork-memory-app-sdk",
  java: "com.sdkwork:sdkwork-memory-app-sdk",
  python: "sdkwork-memory-app-sdk",
  go: "github.com/sdkwork/sdkwork-memory-app-sdk",
};

const memoryBackendPackages = {
  typescript: "@sdkwork/memory-backend-sdk",
  rust: "sdkwork-memory-backend-sdk",
  java: "com.sdkwork:sdkwork-memory-backend-sdk",
  python: "sdkwork-memory-backend-sdk",
  go: "github.com/sdkwork/sdkwork-memory-backend-sdk",
};

const families = [
  {
    root: "sdkwork-knowledgebase-sdk",
    title: "SDKWork Knowledgebase Open API SDK",
    apiVersion: "0.1.0",
    authority: "sdkwork-knowledgebase-open-api",
    sdkTarget: "open",
    apiPrefix: "/knowledge/v3/api",
    schemaUrl: "/knowledge/v3/openapi.json",
    input: "openapi/knowledgebase-open-api.openapi.json",
    packageName: "@sdkwork/knowledgebase-sdk",
    generatedPath: "sdkwork-knowledgebase-sdk-typescript/generated/server-openapi",
    generatedWorkspace: "sdkwork-knowledgebase-sdk-typescript",
    primaryClient: "SdkworkKnowledgebaseClient",
    dependencies: [],
    materializeFrom: {
      sourceFamilyRoot: "sdkwork-knowledgebase-app-sdk",
      sourceInput: "openapi/knowledgebase-app-api.openapi.json",
      operations: [
        openOperation("post", "/app/v3/api/knowledge/retrievals", "/knowledge/v3/api/retrievals"),
        openOperation("get", "/app/v3/api/knowledge/retrievals/{retrievalId}", "/knowledge/v3/api/retrievals/{retrievalId}"),
        openOperation("post", "/app/v3/api/knowledge/context_packs", "/knowledge/v3/api/context_packs"),
        openOperation("post", "/app/v3/api/knowledge/ingests", "/knowledge/v3/api/ingests"),
        openOperation("get", "/app/v3/api/knowledge/ingests/{ingestId}", "/knowledge/v3/api/ingests/{ingestId}"),
        openOperation("get", "/app/v3/api/knowledge/documents", "/knowledge/v3/api/documents"),
        openOperation("get", "/app/v3/api/knowledge/documents/{documentId}", "/knowledge/v3/api/documents/{documentId}"),
        openOperation("get", "/app/v3/api/knowledge/spaces/{spaceId}/browser", "/knowledge/v3/api/spaces/{spaceId}/browser"),
      ],
    },
    forbiddenPathPrefixes: [
      "/app/v3/api/",
      "/backend/v3/api/",
      "/mem/v3/api/",
      "/open/v3/api/drive/",
    ],
  },
  {
    root: "sdkwork-knowledgebase-app-sdk",
    title: "SDKWork Knowledgebase App API SDK",
    apiVersion: "0.1.0",
    authority: "sdkwork-knowledgebase-app-api",
    sdkTarget: "app",
    apiPrefix: "/app/v3/api",
    schemaUrl: "/app/v3/openapi.json",
    input: "openapi/knowledgebase-app-api.openapi.json",
    packageName: "@sdkwork/knowledgebase-app-sdk",
    generatedPath: "sdkwork-knowledgebase-app-sdk-typescript/generated/server-openapi",
    generatedWorkspace: "sdkwork-knowledgebase-app-sdk-typescript",
    primaryClient: "SdkworkAppClient",
    dependencies: [
      dependency("sdkwork-appbase-app-sdk", "appbase-identity-and-session-capability", "/app/v3/api", "sdkwork-appbase-app-api", appbasePackages),
      dependency("sdkwork-drive-app-sdk", "drive-file-and-media-capability", "/app/v3/api", "sdkwork-drive.app", driveAppPackages),
      dependency("sdkwork-memory-app-sdk", "memory-context-capability", "/app/v3/api", "sdkwork-memory.app", memoryAppPackages),
    ],
    forbiddenPathPrefixes: [
      "/app/v3/api/auth/",
      "/app/v3/api/iam/",
      "/app/v3/api/open_platform/",
      "/app/v3/api/system/iam/",
      "/app/v3/api/drive/",
      "/app/v3/api/memory/",
      "/mem/v3/api/",
    ],
  },
  {
    root: "sdkwork-knowledgebase-backend-sdk",
    title: "SDKWork Knowledgebase Backend API SDK",
    apiVersion: "0.1.0",
    authority: "sdkwork-knowledgebase-backend-api",
    sdkTarget: "backend",
    apiPrefix: "/backend/v3/api",
    schemaUrl: "/backend/v3/openapi.json",
    input: "openapi/knowledgebase-backend-api.openapi.json",
    packageName: "@sdkwork/knowledgebase-backend-sdk",
    generatedPath: "sdkwork-knowledgebase-backend-sdk-typescript/generated/server-openapi",
    generatedWorkspace: "sdkwork-knowledgebase-backend-sdk-typescript",
    primaryClient: "SdkworkBackendClient",
    dependencies: [
      dependency("sdkwork-appbase-backend-sdk", "appbase-backend-management-capability", "/backend/v3/api", "sdkwork-appbase-backend-api", appbaseBackendPackages),
      dependency("sdkwork-drive-backend-sdk", "drive-backend-management-capability", "/backend/v3/api", "sdkwork-drive.backend", driveBackendPackages),
      dependency("sdkwork-memory-backend-sdk", "memory-backend-management-capability", "/backend/v3/api", "sdkwork-memory.backend", memoryBackendPackages),
    ],
    forbiddenPathPrefixes: [
      "/backend/v3/api/auth/",
      "/backend/v3/api/iam/",
      "/backend/v3/api/open_platform/",
      "/backend/v3/api/system/iam/",
      "/backend/v3/api/drive/",
      "/backend/v3/api/memory/",
      "/mem/v3/api/",
    ],
  },
];

function openOperation(method, sourcePath, targetPath) {
  return { method, sourcePath, targetPath };
}

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
  await materializeFamilyOpenApi(family);

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

async function materializeFamilyOpenApi(family) {
  if (!family.materializeFrom) {
    return;
  }

  const sourcePath = path.join(
    sdksRoot,
    family.materializeFrom.sourceFamilyRoot,
    family.materializeFrom.sourceInput,
  );
  const source = await readJson(sourcePath);
  const schemas = {};
  const target = {
    openapi: source.openapi || "3.1.0",
    info: {
      title: family.title,
      version: family.apiVersion,
    },
    servers: [
      {
        url: family.apiPrefix,
      },
    ],
    paths: {},
    components: {
      securitySchemes: {
        ApiKey: {
          type: "apiKey",
          in: "header",
          name: "X-API-Key",
        },
      },
      schemas,
    },
  };

  for (const operationMapping of family.materializeFrom.operations) {
    const sourceOperation =
      source.paths?.[operationMapping.sourcePath]?.[operationMapping.method];
    if (!sourceOperation) {
      throw new Error(
        `Missing source operation for ${operationMapping.method.toUpperCase()} ${operationMapping.sourcePath}`,
      );
    }

    const operation = structuredClone(sourceOperation);
    operation.security = [{ ApiKey: [] }];
    operation["x-sdkwork-auth-mode"] = "api-key";
    operation["x-sdkwork-owner"] = owner;
    operation["x-sdkwork-api-authority"] = family.authority;
    operation["x-sdkwork-source"] = "sdks/standardize-knowledgebase-sdk-family.mjs";
    operation["x-sdkwork-source-route-crate"] = "sdkwork-router-knowledgebase-open-api";

    target.paths[operationMapping.targetPath] = {
      ...(target.paths[operationMapping.targetPath] || {}),
      [operationMapping.method]: operation,
    };

    collectReferencedSchemas(operation, source, schemas);
  }

  target["x-sdkwork-owner"] = owner;
  target["x-sdkwork-api-authority"] = family.authority;
  target["x-sdkwork-sdk-family"] = family.root;
  target["x-sdkwork-owner-only-input"] = true;
  target["x-sdkwork-standard-version"] = standardVersion;

  await writeJson(path.join(sdksRoot, family.root, family.input), target);
}

function collectReferencedSchemas(value, source, targetSchemas) {
  if (!value || typeof value !== "object") {
    return;
  }

  if (typeof value.$ref === "string") {
    const schemaName = schemaNameFromRef(value.$ref);
    if (schemaName && !targetSchemas[schemaName]) {
      const sourceSchema = source.components?.schemas?.[schemaName];
      if (!sourceSchema) {
        throw new Error(`Missing referenced schema ${schemaName}`);
      }
      targetSchemas[schemaName] = structuredClone(sourceSchema);
      collectReferencedSchemas(targetSchemas[schemaName], source, targetSchemas);
    }
  }

  if (Array.isArray(value)) {
    for (const item of value) {
      collectReferencedSchemas(item, source, targetSchemas);
    }
    return;
  }

  for (const child of Object.values(value)) {
    collectReferencedSchemas(child, source, targetSchemas);
  }
}

function schemaNameFromRef(ref) {
  const prefix = "#/components/schemas/";
  return ref.startsWith(prefix) ? ref.slice(prefix.length) : null;
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
        path: "../sdkwork-specs/API_SPEC.md",
        purpose: "HTTP/OpenAPI and generated SDK contract rules.",
      },
      {
        file: "SDK_SPEC.md",
        path: "../sdkwork-specs/SDK_SPEC.md",
        purpose: "SDK generation, SDK dependency, and SDK integration rules.",
      },
      {
        file: "SDK_WORKSPACE_GENERATION_SPEC.md",
        path: "../sdkwork-specs/SDK_WORKSPACE_GENERATION_SPEC.md",
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
        standard: "../sdkwork-specs/SDK_WORKSPACE_GENERATION_SPEC.md",
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
      dependencyPolicy: "Appbase, drive, and memory capabilities are consumed through declared dependency SDKs, not copied into generated knowledgebase transports.",
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
