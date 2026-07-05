import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  createdResponse,
  resourceEnvelope,
} from "./lib/openapi-envelope.mjs";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const workspaceRoot = path.resolve(scriptDir, "..");

const openApiPath = path.join(
  workspaceRoot,
  "sdks/sdkwork-knowledgebase-app-sdk/openapi/knowledgebase-app-api.openapi.json",
);
const routeManifestPath = path.join(
  workspaceRoot,
  "sdks/_route-manifests/app-api/sdkwork-routes-knowledgebase-app-api.route-manifest.json",
);

const problemResponse = {
  description: "Error",
  content: {
    "application/json": {
      schema: { $ref: "#/components/schemas/ProblemDetails" },
    },
    "application/problem+json": {
      schema: { $ref: "#/components/schemas/ProblemDetails" },
    },
  },
};

const security = [{ AuthToken: [], AccessToken: [] }];
const extensions = {
  "x-sdkwork-owner": "sdkwork-knowledgebase",
  "x-sdkwork-api-authority": "sdkwork-knowledgebase-app-api",
  "x-sdkwork-request-context": "WebRequestContext",
  "x-sdkwork-api-surface": "app-api",
  "x-sdkwork-source-route-crate": "sdkwork-routes-knowledgebase-app-api",
};

const int64PathParam = (name) => ({
  name,
  in: "path",
  required: true,
  schema: {
    type: "string",
    pattern: "^[0-9]+$",
    "x-sdkwork-int64-string": true,
  },
});

const int64Property = {
  type: "string",
  pattern: "^[0-9]+$",
  "x-sdkwork-int64-string": true,
};

const schemas = {
  KnowledgeUploadSessionStatus: {
    type: "string",
    enum: ["pending", "completed", "expired"],
  },
  CreateKnowledgeUploadSessionRequest: {
    type: "object",
    required: ["spaceId", "title"],
    properties: {
      spaceId: int64Property,
      title: { type: "string" },
      contentType: { type: "string" },
    },
  },
  CompleteKnowledgeUploadSessionRequest: {
    type: "object",
    required: ["spaceId", "title", "idempotencyKey"],
    properties: {
      spaceId: int64Property,
      title: { type: "string" },
      idempotencyKey: { type: "string" },
      payloadMarkdown: { type: "string" },
    },
  },
  KnowledgeUploadSession: {
    type: "object",
    required: [
      "id",
      "spaceId",
      "title",
      "uploadLogicalPath",
      "status",
      "expiresAt",
    ],
    properties: {
      id: int64Property,
      spaceId: int64Property,
      title: { type: "string" },
      uploadLogicalPath: { type: "string" },
      status: { $ref: "#/components/schemas/KnowledgeUploadSessionStatus" },
      expiresAt: { type: "string", format: "date-time" },
    },
  },
};

const newPaths = {
  "/app/v3/api/knowledge/upload_sessions": {
    post: {
      operationId: "uploadSessions.create",
      tags: ["knowledge"],
      summary: "Create a drive-delegated knowledge upload session",
      security,
      requestBody: {
        required: true,
        content: {
          "application/json": {
            schema: {
              $ref: "#/components/schemas/CreateKnowledgeUploadSessionRequest",
            },
          },
        },
      },
      responses: {
        201: createdResponse(
          resourceEnvelope("#/components/schemas/KnowledgeUploadSession"),
        ),
        400: problemResponse,
      },
      ...extensions,
    },
  },
  "/app/v3/api/knowledge/upload_sessions/{sessionId}/complete": {
    post: {
      operationId: "uploadSessions.complete",
      tags: ["knowledge"],
      summary: "Complete a knowledge upload session and start ingestion",
      parameters: [int64PathParam("sessionId")],
      security,
      requestBody: {
        required: true,
        content: {
          "application/json": {
            schema: {
              $ref: "#/components/schemas/CompleteKnowledgeUploadSessionRequest",
            },
          },
        },
      },
      responses: {
        201: createdResponse(resourceEnvelope("#/components/schemas/IngestionJob")),
        400: problemResponse,
        404: problemResponse,
      },
      ...extensions,
    },
  },
};

const newRoutes = [
  {
    method: "POST",
    path: "/app/v3/api/knowledge/upload_sessions",
    operationId: "uploadSessions.create",
  },
  {
    method: "POST",
    path: "/app/v3/api/knowledge/upload_sessions/{sessionId}/complete",
    operationId: "uploadSessions.complete",
  },
];

const spec = JSON.parse(await readFile(openApiPath, "utf8"));
Object.assign(spec.paths, newPaths);
Object.assign(spec.components.schemas, schemas);
await writeFile(openApiPath, `${JSON.stringify(spec, null, 2)}\n`, "utf8");

const manifest = JSON.parse(await readFile(routeManifestPath, "utf8"));
const existing = new Set(
  (manifest.routes || []).map((route) => `${route.method} ${route.path}`),
);
for (const route of newRoutes) {
  const key = `${route.method} ${route.path}`;
  if (existing.has(key)) {
    continue;
  }
  manifest.routes.push({
    method: route.method,
    path: route.path,
    operationId: route.operationId,
    tags: ["knowledge"],
    auth: { mode: "dual-token", required: true },
    handler: { module: "crate::routes", name: null },
    ownership: {
      owner: "sdkwork-knowledgebase",
      apiAuthority: "sdkwork-knowledgebase-app-api",
    },
    requestContext: "WebRequestContext",
    apiSurface: "app-api",
  });
}
manifest.routes.sort((left, right) => {
  const byMethod = left.method.localeCompare(right.method);
  if (byMethod !== 0) {
    return byMethod;
  }
  return left.path.localeCompare(right.path);
});
await writeFile(
  routeManifestPath,
  `${JSON.stringify(manifest, null, 2)}\n`,
  "utf8",
);

console.log("Synced upload session OpenAPI paths and route manifest entries.");
