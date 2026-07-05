#!/usr/bin/env node
/**
 * Aligns knowledgebase app-api OpenAPI success responses with SdkWorkApiResponse envelopes.
 */
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

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const workspaceRoot = path.resolve(scriptDir, '..');
const openApiPath = path.join(
  workspaceRoot,
  'sdks/sdkwork-knowledgebase-app-sdk/openapi/knowledgebase-app-api.openapi.json',
);

const resourceOperations = [
  ['spaces.contextBindings.create', '201', '#/components/schemas/KnowledgeSpaceContextBinding'],
  ['contextBindings.retrieve', '200', '#/components/schemas/KnowledgeSpaceContextBinding'],
  ['contextBindings.update', '200', '#/components/schemas/KnowledgeSpaceContextBinding'],
  ['uploadSessions.create', '201', '#/components/schemas/KnowledgeUploadSession'],
  ['uploadSessions.complete', '201', '#/components/schemas/IngestionJob'],
  ['siteDeployments.preview.retrieve', '200', '#/components/schemas/KnowledgeSiteDeploymentPreview'],
];

const listOperations = [
  [
    'spaces.contextBindings.list',
    '200',
    '#/components/schemas/KnowledgeSpaceContextBinding',
    true,
  ],
  [
    'market.listings.list',
    '200',
    '#/components/schemas/KnowledgeMarketCatalogItem',
    true,
  ],
];

const commandOperations = [
  ['gitSyncs.create', '201', '#/components/schemas/KnowledgeGitSyncResult'],
  ['market.subscriptions.create', '201', '#/components/schemas/KnowledgeMarketSubscriptionResult'],
  ['market.subscriptions.delete', '200', '#/components/schemas/KnowledgeMarketSubscriptionResult'],
  ['siteDeployments.create', '201', '#/components/schemas/KnowledgeSiteDeploymentResult'],
  ['mediaTasks.create', '201', '#/components/schemas/KnowledgeMediaTaskResult'],
];

function findOperation(spec, operationId) {
  for (const methods of Object.values(spec.paths)) {
    for (const operation of Object.values(methods)) {
      if (operation?.operationId === operationId) {
        return operation;
      }
    }
  }
  throw new Error(`operation not found: ${operationId}`);
}

function applyResource(operationId, statusCode, itemRef, spec) {
  const operation = findOperation(spec, operationId);
  const schema = resourceEnvelope(itemRef);
  if (statusCode === '201') {
    operation.responses['201'] = createdResponse(schema);
  } else {
    operation.responses[statusCode] = jsonResponse(schema);
  }
}

function applyList(operationId, statusCode, itemRef, withPagination, spec) {
  const operation = findOperation(spec, operationId);
  operation.responses[statusCode] = jsonResponse(listEnvelope(itemRef));
  if (withPagination) {
    const existing = new Set((operation.parameters ?? []).map((param) => param.name));
    operation.parameters = operation.parameters ?? [];
    for (const param of listPaginationQueryParams) {
      if (!existing.has(param.name)) {
        operation.parameters.push(param);
      }
    }
  }
}

function applyCommand(operationId, statusCode, payloadRef, spec) {
  const operation = findOperation(spec, operationId);
  const schema = commandEnvelope(payloadRef);
  if (statusCode === '201') {
    operation.responses['201'] = createdResponse(schema);
  } else {
    operation.responses[statusCode] = jsonResponse(schema);
  }
}

const spec = JSON.parse(await readFile(openApiPath, 'utf8'));

for (const [operationId, statusCode, itemRef] of resourceOperations) {
  applyResource(operationId, statusCode, itemRef, spec);
}

for (const [operationId, statusCode, itemRef, withPagination] of listOperations) {
  applyList(operationId, statusCode, itemRef, withPagination, spec);
}

for (const [operationId, statusCode, payloadRef] of commandOperations) {
  applyCommand(operationId, statusCode, payloadRef, spec);
}

await writeFile(openApiPath, `${JSON.stringify(spec, null, 2)}\n`, 'utf8');
console.log('Aligned knowledgebase app-api OpenAPI success envelopes.');
