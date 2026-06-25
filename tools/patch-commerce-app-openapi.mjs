#!/usr/bin/env node
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
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
  'x-sdkwork-source-route-crate': 'sdkwork-router-knowledgebase-app-api',
  'x-sdkwork-rate-limit-tier': 'auth-critical',
  'x-sdkwork-auth-mode': 'dual-token',
};

const commercePaths = {
  '/app/v3/api/knowledge/market/listings': {
    get: {
      operationId: 'market.listings.list',
      tags: ['knowledge'],
      summary: 'List knowledge market catalog listings',
      security: [{ AuthToken: [], AccessToken: [] }],
      responses: {
        ...errorResponses,
        200: {
          description: 'OK',
          content: {
            'application/json': {
              schema: { $ref: '#/components/schemas/KnowledgeMarketCatalogList' },
            },
          },
        },
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
        201: {
          description: 'Created',
          content: {
            'application/json': {
              schema: { $ref: '#/components/schemas/KnowledgeMarketSubscriptionResult' },
            },
          },
        },
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
          schema: { type: 'integer', format: 'uint64' },
        },
      ],
      responses: {
        ...errorResponses,
        200: {
          description: 'OK',
          content: {
            'application/json': {
              schema: { $ref: '#/components/schemas/KnowledgeMarketSubscriptionResult' },
            },
          },
        },
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
        201: {
          description: 'Created',
          content: {
            'application/json': {
              schema: { $ref: '#/components/schemas/KnowledgeSiteDeploymentResult' },
            },
          },
        },
      },
      ...sdkworkExtensions,
    },
  },
  '/app/v3/api/knowledge/site_deployments/{deploymentId}/preview': {
    get: {
      operationId: 'siteDeployments.preview.retrieve',
      tags: ['knowledge'],
      summary: 'Retrieve site deployment preview HTML',
      security: [{ AuthToken: [], AccessToken: [] }],
      parameters: [
        {
          name: 'deploymentId',
          in: 'path',
          required: true,
          schema: { type: 'integer', format: 'uint64' },
        },
      ],
      responses: {
        ...errorResponses,
        200: {
          description: 'OK',
          content: {
            'application/json': {
              schema: { $ref: '#/components/schemas/KnowledgeSiteDeploymentPreview' },
            },
          },
        },
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
        201: {
          description: 'Created',
          content: {
            'application/json': {
              schema: { $ref: '#/components/schemas/KnowledgeMediaTaskResult' },
            },
          },
        },
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
  KnowledgeMarketCatalogList: {
    type: 'object',
    required: ['items'],
    properties: {
      items: {
        type: 'array',
        items: { $ref: '#/components/schemas/KnowledgeMarketCatalogItem' },
      },
    },
  },
  KnowledgeMarketSubscriptionRequest: {
    type: 'object',
    required: ['listingId'],
    properties: {
      listingId: { type: 'integer', format: 'uint64' },
    },
  },
  KnowledgeMarketSubscriptionResult: {
    type: 'object',
    required: ['success'],
    properties: {
      success: { type: 'boolean' },
    },
  },
  KnowledgeSiteDeploymentRequest: {
    type: 'object',
    required: ['spaceId', 'platform'],
    properties: {
      spaceId: { type: 'integer', format: 'uint64' },
      platform: { type: 'string', minLength: 1 },
      siteName: { type: ['string', 'null'] },
      customDomain: { type: ['string', 'null'] },
      siteLogoDataUrl: { type: ['string', 'null'] },
    },
  },
  KnowledgeSiteDeploymentResult: {
    type: 'object',
    required: ['success', 'deploymentId', 'url'],
    properties: {
      success: { type: 'boolean' },
      deploymentId: { type: 'integer', format: 'uint64' },
      url: { type: 'string', minLength: 1 },
    },
  },
  KnowledgeSiteDeploymentPreview: {
    type: 'object',
    required: ['deploymentId', 'contentType', 'html'],
    properties: {
      deploymentId: { type: 'integer', format: 'uint64' },
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
      spaceId: { type: 'integer', format: 'uint64' },
      taskType: { $ref: '#/components/schemas/KnowledgeMediaTaskType' },
      prompt: { type: ['string', 'null'] },
      aspectMode: { type: ['string', 'null'] },
      styleMode: { type: ['string', 'null'] },
      sourceUrl: { type: ['string', 'null'] },
      documentId: { type: ['integer', 'null'], format: 'uint64' },
    },
  },
  KnowledgeMediaTaskResult: {
    type: 'object',
    required: ['success', 'suggestions', 'similars'],
    properties: {
      success: { type: 'boolean' },
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
