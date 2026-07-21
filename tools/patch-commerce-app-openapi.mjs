#!/usr/bin/env node
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  commandEnvelope,
  createdResponse,
  jsonResponse,
  listEnvelope,
  listPaginationQueryParams,
  resourceEnvelope,
} from './lib/openapi-envelope.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const checkOnly = process.argv.includes('--check');
let drifted = false;
const targets = [
  path.join(root, 'sdks/sdkwork-knowledgebase-app-sdk/openapi/knowledgebase-app-api.openapi.json'),
  path.join(root, 'apis/app-api/knowledgebase-app-api.openapi.json'),
];

const problemRef = { $ref: '#/components/schemas/ProblemDetails' };
const errorResponses = {
  400: { description: 'Error', content: { 'application/json': { schema: problemRef }, 'application/problem+json': { schema: problemRef } } },
  401: { description: 'Error', content: { 'application/json': { schema: problemRef }, 'application/problem+json': { schema: problemRef } } },
  403: { description: 'Error', content: { 'application/json': { schema: problemRef }, 'application/problem+json': { schema: problemRef } } },
  404: { description: 'Error', content: { 'application/json': { schema: problemRef }, 'application/problem+json': { schema: problemRef } } },
  409: { description: 'Error', content: { 'application/json': { schema: problemRef }, 'application/problem+json': { schema: problemRef } } },
  429: { description: 'Error', content: { 'application/json': { schema: problemRef }, 'application/problem+json': { schema: problemRef } } },
  500: { description: 'Error', content: { 'application/json': { schema: problemRef }, 'application/problem+json': { schema: problemRef } } },
};

const sdkworkExtensions = {
  'x-sdkwork-owner': 'sdkwork-knowledgebase',
  'x-sdkwork-api-authority': 'sdkwork-knowledgebase-app-api',
  'x-sdkwork-request-context': 'WebRequestContext',
  'x-sdkwork-api-surface': 'app-api',
  'x-sdkwork-source-route-crate': 'sdkwork-routes-knowledgebase-app-api',
  'x-sdkwork-rate-limit-tier': 'auth-critical',
  'x-sdkwork-auth-mode': 'dual-token',
};

const int64StringSchema = {
  type: 'string',
  format: 'uint64',
  pattern: '^[0-9]+$',
  'x-sdkwork-int64-string': true,
};
const nullableInt64StringSchema = {
  type: ['string', 'null'],
  format: 'uint64',
  pattern: '^[0-9]+$',
  'x-sdkwork-int64-string': true,
};

const commercePaths = {
  '/app/v3/api/knowledge/market/listings': {
    get: {
      operationId: 'market.listings.list',
      tags: ['knowledge'],
      summary: 'List knowledge market catalog listings',
      security: [{ AuthToken: [], AccessToken: [] }],
      parameters: [...listPaginationQueryParams],
      responses: {
        ...errorResponses,
        200: jsonResponse(listEnvelope('#/components/schemas/KnowledgeMarketCatalogItem')),
      },
      ...sdkworkExtensions,
    },
  },
  '/app/v3/api/knowledge/market/subscriptions': {
    post: {
      operationId: 'market.subscriptions.create',
      tags: ['knowledge'],
      summary: 'Subscribe to a knowledge market listing',
      security: [{ AuthToken: [], AccessToken: [] }],
      requestBody: {
        required: true,
        content: {
          'application/json': {
            schema: { $ref: '#/components/schemas/KnowledgeMarketSubscriptionRequest' },
          },
        },
      },
      responses: {
        ...errorResponses,
        201: createdResponse(commandEnvelope('#/components/schemas/KnowledgeMarketSubscriptionResult')),
      },
      ...sdkworkExtensions,
    },
  },
  '/app/v3/api/knowledge/market/subscriptions/{listingId}': {
    delete: {
      operationId: 'market.subscriptions.delete',
      tags: ['knowledge'],
      summary: 'Unsubscribe from a knowledge market listing',
      security: [{ AuthToken: [], AccessToken: [] }],
      parameters: [
        {
          name: 'listingId',
          in: 'path',
          required: true,
          schema: int64StringSchema,
        },
      ],
      responses: {
        ...errorResponses,
        204: { description: 'No Content' },
      },
      ...sdkworkExtensions,
    },
  },
  '/app/v3/api/knowledge/spaces/{spaceId}/site': {
    get: {
      operationId: 'sites.retrieve',
      tags: ['knowledge'],
      summary: 'Retrieve a knowledge site by space',
      security: [{ AuthToken: [], AccessToken: [] }],
      parameters: [{ name: 'spaceId', in: 'path', required: true, schema: int64StringSchema }],
      responses: {
        ...errorResponses,
        200: jsonResponse(resourceEnvelope('#/components/schemas/KnowledgeSite')),
      },
      ...sdkworkExtensions,
    },
    put: {
      operationId: 'sites.update',
      tags: ['knowledge'],
      summary: 'Create or update a knowledge site',
      security: [{ AuthToken: [], AccessToken: [] }],
      parameters: [{ name: 'spaceId', in: 'path', required: true, schema: int64StringSchema }],
      requestBody: {
        required: true,
        content: {
          'application/json': {
            schema: { $ref: '#/components/schemas/UpsertKnowledgeSiteRequest' },
          },
        },
      },
      responses: {
        ...errorResponses,
        200: jsonResponse(resourceEnvelope('#/components/schemas/KnowledgeSite')),
      },
      ...sdkworkExtensions,
    },
  },
  '/app/v3/api/knowledge/sites/{siteId}/releases': {
    get: {
      operationId: 'siteReleases.list',
      tags: ['knowledge'],
      summary: 'List immutable knowledge site releases',
      security: [{ AuthToken: [], AccessToken: [] }],
      parameters: [
        { name: 'siteId', in: 'path', required: true, schema: int64StringSchema },
        ...listPaginationQueryParams,
      ],
      responses: {
        ...errorResponses,
        200: jsonResponse(listEnvelope('#/components/schemas/KnowledgeSiteRelease')),
      },
      ...sdkworkExtensions,
    },
    post: {
      operationId: 'siteReleases.create',
      tags: ['knowledge'],
      summary: 'Build and atomically publish a knowledge site release',
      security: [{ AuthToken: [], AccessToken: [] }],
      parameters: [{ name: 'siteId', in: 'path', required: true, schema: int64StringSchema }],
      requestBody: {
        required: true,
        content: { 'application/json': { schema: { $ref: '#/components/schemas/PublishKnowledgeSiteReleaseRequest' } } },
      },
      responses: {
        ...errorResponses,
        201: createdResponse(commandEnvelope('#/components/schemas/KnowledgeSitePublicationResult')),
      },
      ...sdkworkExtensions,
    },
  },
  '/app/v3/api/knowledge/site_releases/{releaseId}': {
    get: {
      operationId: 'siteReleases.retrieve',
      tags: ['knowledge'],
      summary: 'Retrieve an immutable knowledge site release',
      security: [{ AuthToken: [], AccessToken: [] }],
      parameters: [{ name: 'releaseId', in: 'path', required: true, schema: int64StringSchema }],
      responses: {
        ...errorResponses,
        200: jsonResponse(resourceEnvelope('#/components/schemas/KnowledgeSiteRelease')),
      },
      ...sdkworkExtensions,
    },
  },
  '/app/v3/api/knowledge/sites/{siteId}/rollbacks': {
    post: {
      operationId: 'siteReleases.rollback',
      tags: ['knowledge'],
      summary: 'Atomically activate a prior ready knowledge site release',
      security: [{ AuthToken: [], AccessToken: [] }],
      parameters: [{ name: 'siteId', in: 'path', required: true, schema: int64StringSchema }],
      requestBody: {
        required: true,
        content: { 'application/json': { schema: { $ref: '#/components/schemas/RollbackKnowledgeSiteReleaseRequest' } } },
      },
      responses: {
        ...errorResponses,
        200: jsonResponse(commandEnvelope('#/components/schemas/KnowledgeSite')),
      },
      ...sdkworkExtensions,
    },
  },
  '/app/v3/api/knowledge/sites/{siteId}/host_bindings': {
    get: {
      operationId: 'siteHostBindings.list',
      tags: ['knowledge'],
      summary: 'List knowledge site host bindings',
      security: [{ AuthToken: [], AccessToken: [] }],
      parameters: [
        { name: 'siteId', in: 'path', required: true, schema: int64StringSchema },
        ...listPaginationQueryParams,
      ],
      responses: {
        ...errorResponses,
        200: jsonResponse(listEnvelope('#/components/schemas/KnowledgeSiteHostBinding')),
      },
      ...sdkworkExtensions,
    },
    post: {
      operationId: 'siteHostBindings.create',
      tags: ['knowledge'],
      summary: 'Create a custom-prefix or external-domain host binding',
      security: [{ AuthToken: [], AccessToken: [] }],
      parameters: [{ name: 'siteId', in: 'path', required: true, schema: int64StringSchema }],
      requestBody: {
        required: true,
        content: { 'application/json': { schema: { $ref: '#/components/schemas/CreateKnowledgeSiteHostBindingRequest' } } },
      },
      responses: {
        ...errorResponses,
        201: createdResponse(commandEnvelope('#/components/schemas/KnowledgeSiteHostBinding')),
      },
      ...sdkworkExtensions,
    },
  },
  '/app/v3/api/knowledge/sites/{siteId}/host_bindings/{bindingId}': {
    delete: {
      operationId: 'siteHostBindings.delete',
      tags: ['knowledge'],
      summary: 'Delete a non-system knowledge site host binding',
      security: [{ AuthToken: [], AccessToken: [] }],
      parameters: [
        { name: 'siteId', in: 'path', required: true, schema: int64StringSchema },
        { name: 'bindingId', in: 'path', required: true, schema: int64StringSchema },
        { name: 'expected_version', in: 'query', required: true, schema: int64StringSchema },
      ],
      responses: {
        ...errorResponses,
        204: { description: 'No Content' },
      },
      ...sdkworkExtensions,
    },
  },
  '/app/v3/api/knowledge/media_tasks': {
    post: {
      operationId: 'mediaTasks.create',
      tags: ['knowledge'],
      summary: 'Create a knowledge media task (image generation or speech-to-text)',
      security: [{ AuthToken: [], AccessToken: [] }],
      requestBody: {
        required: true,
        content: {
          'application/json': {
            schema: { $ref: '#/components/schemas/KnowledgeMediaTaskRequest' },
          },
        },
      },
      responses: {
        ...errorResponses,
        201: createdResponse(commandEnvelope('#/components/schemas/KnowledgeMediaTaskResult')),
      },
      ...sdkworkExtensions,
    },
  },
};

const commerceSchemas = {
  KnowledgeDriveImportRequest: {
    type: 'object',
    required: ['spaceId', 'title', 'driveSpaceId', 'driveNodeId', 'idempotencyKey'],
    properties: {
      spaceId: int64StringSchema,
      title: { type: 'string', minLength: 1 },
      driveSpaceId: { type: 'string', minLength: 1, maxLength: 128 },
      driveNodeId: { type: 'string', minLength: 1, maxLength: 128 },
      idempotencyKey: { type: 'string', minLength: 1, maxLength: 128 },
      language: { type: ['string', 'null'] },
    },
  },
  KnowledgeDriveObjectRef: {
    type: 'object',
    required: ['id', 'spaceId', 'driveSpaceId', 'driveNodeId', 'sizeBytes', 'objectRole', 'accessMode'],
    properties: {
      id: int64StringSchema,
      spaceId: int64StringSchema,
      driveSpaceId: { type: ['string', 'null'] },
      driveNodeId: { type: ['string', 'null'] },
      logicalPath: { type: ['string', 'null'] },
      contentType: { type: ['string', 'null'] },
      sizeBytes: int64StringSchema,
      checksumSha256Hex: { type: ['string', 'null'] },
      objectRole: { type: 'string', minLength: 1 },
      accessMode: { type: 'string', minLength: 1 },
    },
  },
  KnowledgeMarketCatalogItem: {
    type: 'object',
    required: [
      'id',
      'title',
      'icon',
      'description',
      'author',
      'tags',
      'subscribersCount',
      'documentsCount',
      'provider',
      'modelName',
      'isSubscribed',
    ],
    properties: {
      id: { type: 'string', minLength: 1 },
      title: { type: 'string', minLength: 1 },
      icon: { type: 'string', minLength: 1 },
      description: { type: 'string' },
      author: { type: 'string', minLength: 1 },
      tags: { type: 'array', items: { type: 'string' } },
      subscribersCount: { type: 'integer', format: 'uint32', minimum: 0 },
      documentsCount: { type: 'integer', format: 'uint32', minimum: 0 },
      provider: { type: 'string', minLength: 1 },
      modelName: { type: 'string', minLength: 1 },
      isSubscribed: { type: 'boolean' },
    },
  },
  KnowledgeMarketSubscriptionRequest: {
    type: 'object',
    required: ['listingId'],
    properties: {
      listingId: int64StringSchema,
    },
  },
  KnowledgeMarketSubscriptionResult: {
    type: 'object',
    required: ['accepted', 'status'],
    properties: {
      accepted: { type: 'boolean', const: true },
      status: { type: 'string', enum: ['completed'] },
    },
  },
  KnowledgeSiteVisibility: { type: 'string', enum: ['private', 'unlisted', 'public'] },
  KnowledgeSitePublishMode: { type: 'string', enum: ['manual', 'automatic'] },
  KnowledgeSiteState: { type: 'string', enum: ['draft', 'active', 'paused'] },
  KnowledgeSiteReleaseState: { type: 'string', enum: ['building', 'ready', 'failed'] },
  KnowledgeSiteHostBindingType: { type: 'string', enum: ['system_id', 'custom_prefix', 'external_domain'] },
  KnowledgeSiteHostBindingState: { type: 'string', enum: ['pending', 'verified', 'active', 'failed'] },
  UpsertKnowledgeSiteRequest: {
    type: 'object',
    required: ['spaceId', 'title', 'visibility', 'themeId', 'publishMode'],
    properties: {
      spaceId: int64StringSchema,
      title: { type: 'string', minLength: 1, maxLength: 256 },
      visibility: { $ref: '#/components/schemas/KnowledgeSiteVisibility' },
      homepageConceptId: { type: ['string', 'null'], maxLength: 512 },
      themeId: { type: 'string', minLength: 1, maxLength: 64 },
      publishMode: { $ref: '#/components/schemas/KnowledgeSitePublishMode' },
      expectedVersion: nullableInt64StringSchema,
    },
  },
  KnowledgeSite: {
    type: 'object',
    required: ['id', 'uuid', 'tenantId', 'organizationId', 'spaceId', 'title', 'visibility', 'themeId', 'publishMode', 'lifecycleState', 'createdAt', 'updatedAt', 'version'],
    properties: {
      id: int64StringSchema,
      uuid: { type: 'string', minLength: 1 },
      tenantId: int64StringSchema,
      organizationId: int64StringSchema,
      spaceId: int64StringSchema,
      title: { type: 'string', minLength: 1 },
      visibility: { $ref: '#/components/schemas/KnowledgeSiteVisibility' },
      homepageConceptId: { type: ['string', 'null'] },
      themeId: { type: 'string', minLength: 1 },
      publishMode: { $ref: '#/components/schemas/KnowledgeSitePublishMode' },
      lifecycleState: { $ref: '#/components/schemas/KnowledgeSiteState' },
      canonicalHostBindingId: nullableInt64StringSchema,
      currentReleaseId: nullableInt64StringSchema,
      createdAt: { type: 'string', format: 'date-time' },
      updatedAt: { type: 'string', format: 'date-time' },
      version: int64StringSchema,
    },
  },
  PublishKnowledgeSiteReleaseRequest: {
    type: 'object',
    required: ['expectedSiteVersion'],
    properties: { expectedSiteVersion: int64StringSchema },
  },
  RollbackKnowledgeSiteReleaseRequest: {
    type: 'object',
    required: ['releaseId', 'expectedSiteVersion'],
    properties: { releaseId: int64StringSchema, expectedSiteVersion: int64StringSchema },
  },
  KnowledgeSiteRelease: {
    type: 'object',
    required: ['id', 'uuid', 'siteId', 'lifecycleState', 'sourceContentHash', 'pageCount', 'assetCount', 'createdAt', 'version'],
    properties: {
      id: int64StringSchema,
      uuid: { type: 'string', minLength: 1 },
      siteId: int64StringSchema,
      lifecycleState: { $ref: '#/components/schemas/KnowledgeSiteReleaseState' },
      sourceContentHash: { type: 'string', pattern: '^[a-f0-9]{64}$' },
      manifestDriveUri: { type: ['string', 'null'] },
      manifestDriveSpaceId: { type: ['string', 'null'] },
      manifestDriveNodeId: { type: ['string', 'null'] },
      manifestChecksumSha256Hex: { type: ['string', 'null'], pattern: '^[a-f0-9]{64}$' },
      pageCount: { type: 'integer', format: 'uint32', minimum: 0 },
      assetCount: { type: 'integer', format: 'uint32', minimum: 0 },
      previousReleaseId: nullableInt64StringSchema,
      errorCode: { type: ['string', 'null'] },
      createdAt: { type: 'string', format: 'date-time' },
      completedAt: { type: ['string', 'null'], format: 'date-time' },
      version: int64StringSchema,
    },
  },
  KnowledgeSitePublicationResult: {
    type: 'object',
    required: ['site', 'release', 'publicUrl'],
    properties: {
      site: { $ref: '#/components/schemas/KnowledgeSite' },
      release: { $ref: '#/components/schemas/KnowledgeSiteRelease' },
      publicUrl: { type: 'string', format: 'uri' },
    },
  },
  CreateKnowledgeSiteHostBindingRequest: {
    type: 'object',
    required: ['bindingType', 'host', 'canonical', 'expectedSiteVersion'],
    properties: {
      bindingType: { $ref: '#/components/schemas/KnowledgeSiteHostBindingType' },
      host: { type: 'string', minLength: 1, maxLength: 253 },
      canonical: { type: 'boolean' },
      expectedSiteVersion: int64StringSchema,
    },
  },
  KnowledgeSiteHostBinding: {
    type: 'object',
    required: ['id', 'uuid', 'siteId', 'bindingType', 'normalizedHost', 'canonical', 'lifecycleState', 'createdAt', 'updatedAt', 'version'],
    properties: {
      id: int64StringSchema,
      uuid: { type: 'string', minLength: 1 },
      siteId: int64StringSchema,
      bindingType: { $ref: '#/components/schemas/KnowledgeSiteHostBindingType' },
      normalizedHost: { type: 'string', minLength: 1, maxLength: 253 },
      canonical: { type: 'boolean' },
      lifecycleState: { $ref: '#/components/schemas/KnowledgeSiteHostBindingState' },
      webServerSiteId: { type: ['string', 'null'] },
      webServerDomainId: { type: ['string', 'null'] },
      webServerDeploymentId: { type: ['string', 'null'] },
      createdAt: { type: 'string', format: 'date-time' },
      updatedAt: { type: 'string', format: 'date-time' },
      version: int64StringSchema,
    },
  },
  KnowledgeMediaTaskType: {
    type: 'string',
    enum: ['generate_image', 'speech_to_text'],
  },
  KnowledgeMediaTaskRequest: {
    type: 'object',
    required: ['spaceId', 'taskType'],
    properties: {
      spaceId: int64StringSchema,
      taskType: { $ref: '#/components/schemas/KnowledgeMediaTaskType' },
      prompt: { type: ['string', 'null'] },
      aspectMode: { type: ['string', 'null'] },
      styleMode: { type: ['string', 'null'] },
      sourceUrl: { type: ['string', 'null'] },
      documentId: nullableInt64StringSchema,
    },
  },
  KnowledgeMediaTaskResult: {
    type: 'object',
    required: ['accepted', 'status', 'suggestions', 'similars'],
    properties: {
      accepted: { type: 'boolean', const: true },
      status: { type: 'string', enum: ['completed'] },
      url: { type: ['string', 'null'] },
      resolution: { type: ['string', 'null'] },
      text: { type: ['string', 'null'] },
      suggestions: { type: 'array', items: { type: 'string' } },
      similars: { type: 'array', items: { type: 'string' } },
    },
  },
};

for (const openapiPath of targets) {
  const before = await readFile(openapiPath, 'utf8');
  const spec = JSON.parse(before);
  delete spec.paths['/app/v3/api/knowledge/site_deployments'];
  delete spec.paths['/app/v3/api/knowledge/site_deployments/{deploymentId}/preview'];
  delete spec.paths['/app/v3/api/knowledge/upload_sessions'];
  delete spec.paths['/app/v3/api/knowledge/upload_sessions/{sessionId}/complete'];
  for (const schemaName of [
    'KnowledgeSiteDeploymentRequest',
    'KnowledgeSiteDeploymentResult',
    'KnowledgeSiteDeploymentPreview',
    'CreateKnowledgeUploadSessionRequest',
    'CompleteKnowledgeUploadSessionRequest',
    'KnowledgeUploadSession',
    'KnowledgeUploadSessionStatus',
  ]) {
    delete spec.components.schemas[schemaName];
  }
  Object.assign(spec.paths, commercePaths);
  Object.assign(spec.components.schemas, commerceSchemas);
  const desired = `${JSON.stringify(spec, null, 2)}\n`;
  if (desired === before) {
    continue;
  }
  if (checkOnly) {
    drifted = true;
    console.error(`Commerce App OpenAPI drift: ${path.relative(root, openapiPath)}`);
  } else {
    await writeFile(openapiPath, desired, 'utf8');
    console.log(`patched ${openapiPath}`);
  }
}

if (drifted) process.exit(1);
