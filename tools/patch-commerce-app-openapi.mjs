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
const targets = [
  path.join(root, 'sdks/sdkwork-knowledgebase-app-sdk/openapi/knowledgebase-app-api.openapi.json'),
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
  '/app/v3/api/knowledge/site_deployments': {
    post: {
      operationId: 'siteDeployments.create',
      tags: ['knowledge'],
      summary: 'Deploy a knowledge space as a static website',
      security: [{ AuthToken: [], AccessToken: [] }],
      requestBody: {
        required: true,
        content: {
          'application/json': {
            schema: { $ref: '#/components/schemas/KnowledgeSiteDeploymentRequest' },
          },
        },
      },
      responses: {
        ...errorResponses,
        201: createdResponse(commandEnvelope('#/components/schemas/KnowledgeSiteDeploymentResult')),
      },
      ...sdkworkExtensions,
    },
  },
  '/app/v3/api/knowledge/site_deployments/{deploymentId}/preview': {
    get: {
      operationId: 'siteDeployments.preview.list',
      tags: ['knowledge'],
      summary: 'Retrieve site deployment preview HTML',
      security: [{ AuthToken: [], AccessToken: [] }],
      parameters: [
        {
          name: 'deploymentId',
          in: 'path',
          required: true,
          schema: int64StringSchema,
        },
      ],
      responses: {
        ...errorResponses,
        200: jsonResponse(resourceEnvelope('#/components/schemas/KnowledgeSiteDeploymentPreview')),
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
  KnowledgeSiteDeploymentRequest: {
    type: 'object',
    required: ['spaceId', 'platform'],
    properties: {
      spaceId: int64StringSchema,
      platform: { type: 'string', minLength: 1 },
      siteName: { type: ['string', 'null'] },
      customDomain: { type: ['string', 'null'] },
      siteLogoDataUrl: { type: ['string', 'null'] },
    },
  },
  KnowledgeSiteDeploymentResult: {
    type: 'object',
    required: ['accepted', 'status', 'deploymentId', 'url'],
    properties: {
      accepted: { type: 'boolean', const: true },
      status: { type: 'string', enum: ['completed'] },
      deploymentId: int64StringSchema,
      url: { type: 'string', minLength: 1 },
    },
  },
  KnowledgeSiteDeploymentPreview: {
    type: 'object',
    required: ['deploymentId', 'contentType', 'html'],
    properties: {
      deploymentId: int64StringSchema,
      contentType: { type: 'string', minLength: 1 },
      html: { type: 'string' },
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
  const spec = JSON.parse(await readFile(openapiPath, 'utf8'));
  Object.assign(spec.paths, commercePaths);
  Object.assign(spec.components.schemas, commerceSchemas);
  await writeFile(openapiPath, `${JSON.stringify(spec, null, 2)}\n`, 'utf8');
  console.log(`patched ${openapiPath}`);
}
