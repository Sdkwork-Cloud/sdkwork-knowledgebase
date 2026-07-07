#!/usr/bin/env node
/**
 * Aligns Knowledgebase SDK-owned OpenAPI authorities with SDKWork v3 HTTP rules.
 *
 * The SDK family OpenAPI files under SDK family openapi directories are the local authority.
 * Generated transports and apis/** copies are derived from these files.
 */
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  commandEnvelope,
  createdResponse,
  jsonResponse,
  listEnvelope,
  resourceEnvelope,
} from './lib/openapi-envelope.mjs';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const workspaceRoot = path.resolve(scriptDir, '..');
const checkOnly = process.argv.includes('--check');
const pendingChanges = [];

const appOpenApiPath = path.join(
  workspaceRoot,
  'sdks/sdkwork-knowledgebase-app-sdk/openapi/knowledgebase-app-api.openapi.json',
);
const backendOpenApiPath = path.join(
  workspaceRoot,
  'sdks/sdkwork-knowledgebase-backend-sdk/openapi/knowledgebase-backend-api.openapi.json',
);

const appOperations = [
  resource('get', '/app/v3/api/knowledge/documents/{documentId}/content', {
    operationId: 'documents.content.list',
    itemRef: '#/components/schemas/KnowledgeDocumentContent',
  }),
  resource('post', '/app/v3/api/knowledge/documents/{documentId}/versions', {
    operationId: 'documents.versions.versions',
    status: '200',
    itemRef: '#/components/schemas/KnowledgeDocumentVersion',
  }),
  resource('get', '/app/v3/api/knowledge/okf/index', {
    operationId: 'okf.bundle.index.list',
    itemRef: '#/components/schemas/OkfIndexDocument',
  }),
  resource('get', '/app/v3/api/knowledge/okf/log', {
    operationId: 'okf.bundle.log.list',
    itemRef: '#/components/schemas/OkfLogDocument',
  }),
  resource('get', '/app/v3/api/knowledge/okf/profile', {
    operationId: 'okf.bundle.profile.list',
    itemRef: '#/components/schemas/OkfProfileDocument',
  }),
  resource('post', '/app/v3/api/knowledge/okf/queries/{queryId}/file_answer', {
    operationId: 'okf.queries.fileAnswer',
    status: '200',
    itemRef: '#/components/schemas/OkfQueryResult',
  }),
  resource('put', '/app/v3/api/knowledge/okf/concepts/upsert', {
    operationId: 'okf.concepts.update',
    itemRef: '#/components/schemas/OkfConceptSummary',
  }),
  resource('get', '/app/v3/api/knowledge/agent_profiles/{profileId}/bindings', {
    operationId: 'agentProfiles.bindings.list',
    itemRef: '#/components/schemas/KnowledgeAgentBindingList',
  }),
  resource('post', '/app/v3/api/knowledge/agent_profiles/{profileId}/bindings', {
    operationId: 'agentProfiles.bindings.bindings',
    status: '200',
    itemRef: '#/components/schemas/KnowledgeAgentBinding',
  }),
  resource('post', '/app/v3/api/knowledge/agent_profiles/{profileId}/retrieval_preview', {
    operationId: 'agentProfiles.retrievalPreview.retrievalPreview',
    status: '200',
    itemRef: '#/components/schemas/KnowledgeRetrievalResult',
  }),
  resource('post', '/app/v3/api/knowledge/agent_profiles/{profileId}/chat', {
    operationId: 'agentProfiles.chat.chat',
    status: '200',
    itemRef: '#/components/schemas/KnowledgeAgentChatResponse',
  }),
  list('get', '/app/v3/api/knowledge/spaces/{spaceId}/context_bindings', {
    operationId: 'spaces.contextBindings.list',
    itemRef: '#/components/schemas/KnowledgeSpaceContextBinding',
  }),
  resource('post', '/app/v3/api/knowledge/spaces/{spaceId}/context_bindings', {
    operationId: 'spaces.contextBindings.contextBindings',
    status: '200',
    itemRef: '#/components/schemas/KnowledgeSpaceContextBinding',
  }),
  list('get', '/app/v3/api/knowledge/spaces/{spaceId}/members', {
    operationId: 'spaces.members.list',
    itemRef: '#/components/schemas/KnowledgeSpaceMember',
  }),
  command('post', '/app/v3/api/knowledge/spaces/{spaceId}/members', {
    operationId: 'spaces.members.members',
    status: '200',
    payloadRef: '#/components/schemas/SdkWorkCommandData',
  }),
  noContent('delete', '/app/v3/api/knowledge/spaces/{spaceId}/members', {
    operationId: 'spaces.members.delete',
  }),
  resource('post', '/app/v3/api/knowledge/upload_sessions/{sessionId}/complete', {
    operationId: 'uploadSessions.complete',
    status: '200',
    itemRef: '#/components/schemas/IngestionJob',
  }),
  resource('get', '/app/v3/api/knowledge/wechat/official_accounts', {
    operationId: 'wechat.officialAccounts.list',
    itemRef: '#/components/schemas/KnowledgeWechatOfficialAccountList',
  }),
  resource('put', '/app/v3/api/knowledge/wechat/official_accounts', {
    operationId: 'wechat.officialAccounts.update',
    itemRef: '#/components/schemas/KnowledgeWechatOfficialAccountList',
  }),
  resource('get', '/app/v3/api/knowledge/wechat/applets', {
    operationId: 'wechat.applets.list',
    itemRef: '#/components/schemas/KnowledgeWechatAppletList',
  }),
  resource('put', '/app/v3/api/knowledge/wechat/applets', {
    operationId: 'wechat.applets.update',
    itemRef: '#/components/schemas/KnowledgeWechatAppletList',
  }),
  resource('post', '/app/v3/api/knowledge/wechat/articles/publish', {
    operationId: 'wechat.articles.publish',
    status: '200',
    itemRef: '#/components/schemas/KnowledgeWechatOperationResult',
    command: true,
  }),
  resource('post', '/app/v3/api/knowledge/wechat/articles/preview', {
    operationId: 'wechat.articles.preview',
    status: '200',
    itemRef: '#/components/schemas/KnowledgeWechatOperationResult',
    command: true,
  }),
  command('post', '/app/v3/api/knowledge/git_syncs', {
    operationId: 'gitSyncs.create',
    status: '201',
    payloadRef: '#/components/schemas/KnowledgeGitSyncResult',
  }),
  command('post', '/app/v3/api/knowledge/market/subscriptions', {
    operationId: 'market.subscriptions.create',
    status: '201',
    payloadRef: '#/components/schemas/KnowledgeMarketSubscriptionResult',
  }),
  noContent('delete', '/app/v3/api/knowledge/market/subscriptions/{listingId}', {
    operationId: 'market.subscriptions.delete',
  }),
  command('post', '/app/v3/api/knowledge/site_deployments', {
    operationId: 'siteDeployments.create',
    status: '201',
    payloadRef: '#/components/schemas/KnowledgeSiteDeploymentResult',
  }),
  resource('get', '/app/v3/api/knowledge/site_deployments/{deploymentId}/preview', {
    operationId: 'siteDeployments.preview.list',
    itemRef: '#/components/schemas/KnowledgeSiteDeploymentPreview',
  }),
  command('post', '/app/v3/api/knowledge/media_tasks', {
    operationId: 'mediaTasks.create',
    status: '201',
    payloadRef: '#/components/schemas/KnowledgeMediaTaskResult',
  }),
];

const backendOperations = [
  resource('post', '/backend/v3/api/knowledge/okf/candidates/{candidateId}/approve', {
    operationId: 'okf.candidates.approve',
    status: '200',
    itemRef: '#/components/schemas/OkfCandidateResult',
  }),
  resource('post', '/backend/v3/api/knowledge/okf/candidates/{candidateId}/reject', {
    operationId: 'okf.candidates.reject',
    status: '200',
    itemRef: '#/components/schemas/OkfCandidateResult',
  }),
  resource('post', '/backend/v3/api/knowledge/okf/concepts/{conceptId}/publish', {
    operationId: 'okf.concepts.publish',
    status: '200',
    itemRef: '#/components/schemas/OkfConceptSummary',
  }),
  resource('post', '/backend/v3/api/knowledge/okf/index/rebuild', {
    operationId: 'okf.bundle.index.create',
    status: '201',
    itemRef: '#/components/schemas/OkfIndexDocument',
  }),
  resource('get', '/backend/v3/api/knowledge/provider_health', {
    operationId: 'providerHealth.list',
    itemRef: '#/components/schemas/KnowledgeProviderHealth',
  }),
  resource('get', '/backend/v3/api/knowledge/tenants/current', {
    operationId: 'tenants.current.list',
    itemRef: '#/components/schemas/KnowledgeTenantStatus',
  }),
  resource('post', '/backend/v3/api/knowledge/compliance/audit_events/export', {
    operationId: 'compliance.auditEvents.export.create',
    status: '201',
    itemRef: '#/components/schemas/KnowledgeAuditEventExport',
  }),
  resource('post', '/backend/v3/api/knowledge/compliance/audit_events/anonymize_actor', {
    operationId: 'compliance.auditEvents.anonymizeActor.create',
    status: '201',
    itemRef: '#/components/schemas/AnonymizeKnowledgeAuditSubjectResult',
  }),
];

function resource(method, routePath, options) {
  return {
    method,
    routePath,
    operationId: options.operationId,
    status: options.status ?? '200',
    schema: options.command
      ? commandEnvelope(options.itemRef)
      : resourceEnvelope(options.itemRef),
  };
}

function list(method, routePath, options) {
  return {
    method,
    routePath,
    operationId: options.operationId,
    status: '200',
    schema: listEnvelope(options.itemRef),
  };
}

function command(method, routePath, options) {
  return {
    method,
    routePath,
    operationId: options.operationId,
    status: options.status ?? '200',
    schema: commandEnvelope(options.payloadRef),
  };
}

function noContent(method, routePath, options) {
  return {
    method,
    routePath,
    operationId: options.operationId,
    status: '204',
    noContent: true,
  };
}

async function alignFile(filePath, operationAlignments) {
  const spec = JSON.parse(await readFile(filePath, 'utf8'));
  normalizeQueryParameterNames(spec);
  for (const alignment of operationAlignments) {
    applyOperationAlignment(spec, alignment);
  }
  alignCommandResultSchemas(spec);
  await writeJsonIfChanged(filePath, spec);
}

function alignCommandResultSchemas(spec) {
  const schemas = spec.components?.schemas;
  if (!schemas || typeof schemas !== 'object') {
    return;
  }

  const replacements = {
    KnowledgeWechatOperationResult: commandResultSchema(),
    KnowledgeGitSyncResult: {
      ...commandResultSchema(),
      required: ['accepted', 'status', 'hash', 'syncedCount'],
      properties: {
        ...commandResultProperties(),
        hash: { type: 'string', minLength: 1 },
        syncedCount: { type: 'integer', format: 'uint32', minimum: 0 },
      },
    },
    KnowledgeMarketSubscriptionResult: commandResultSchema(),
    KnowledgeSiteDeploymentResult: {
      ...commandResultSchema(),
      required: ['accepted', 'status', 'deploymentId', 'url'],
      properties: {
        ...commandResultProperties(),
        deploymentId: int64StringSchema(),
        url: { type: 'string', minLength: 1 },
      },
    },
    KnowledgeMediaTaskResult: {
      ...commandResultSchema(),
      required: ['accepted', 'status', 'suggestions', 'similars'],
      properties: {
        ...commandResultProperties(),
        url: { type: ['string', 'null'] },
        resolution: { type: ['string', 'null'] },
        text: { type: ['string', 'null'] },
        suggestions: { type: 'array', items: { type: 'string' } },
        similars: { type: 'array', items: { type: 'string' } },
      },
    },
  };

  for (const [schemaName, schema] of Object.entries(replacements)) {
    if (Object.hasOwn(schemas, schemaName)) {
      schemas[schemaName] = schema;
    }
  }
}

function commandResultSchema() {
  return {
    type: 'object',
    required: ['accepted', 'status'],
    properties: commandResultProperties(),
  };
}

function commandResultProperties() {
  return {
    accepted: { type: 'boolean', const: true },
    status: { type: 'string', enum: ['completed'] },
  };
}

function int64StringSchema() {
  return {
    type: 'string',
    format: 'uint64',
    pattern: '^[0-9]+$',
    'x-sdkwork-int64-string': true,
  };
}

function normalizeQueryParameterNames(spec) {
  for (const pathItem of Object.values(spec.paths ?? {})) {
    if (!pathItem || typeof pathItem !== 'object') {
      continue;
    }
    for (const operation of Object.values(pathItem)) {
      if (!operation || typeof operation !== 'object' || !Array.isArray(operation.parameters)) {
        continue;
      }
      let hasPageSize = false;
      operation.parameters = operation.parameters.filter((parameter) => {
        if (parameter?.in !== 'query') {
          return true;
        }
        if (parameter.name === 'pageSize') {
          parameter.name = 'page_size';
        }
        if (parameter.name !== 'page_size') {
          return true;
        }
        if (hasPageSize) {
          return false;
        }
        hasPageSize = true;
        return true;
      });
    }
  }
}

function applyOperationAlignment(spec, alignment) {
  const operation = spec.paths?.[alignment.routePath]?.[alignment.method];
  if (!operation) {
    throw new Error(
      `Missing OpenAPI operation: ${alignment.method.toUpperCase()} ${alignment.routePath}`,
    );
  }

  operation.operationId = alignment.operationId;
  operation.responses = operation.responses && typeof operation.responses === 'object'
    ? operation.responses
    : {};
  removeSuccessResponses(operation.responses);

  if (alignment.noContent) {
    operation.responses['204'] = { description: 'No Content' };
    return;
  }

  const response = alignment.status === '201'
    ? createdResponse(alignment.schema)
    : jsonResponse(alignment.schema);
  operation.responses[alignment.status] = response;
}

function removeSuccessResponses(responses) {
  for (const status of Object.keys(responses)) {
    if (/^2[0-9][0-9]$/u.test(status)) {
      delete responses[status];
    }
  }
}

async function writeJsonIfChanged(filePath, value) {
  const desired = `${JSON.stringify(value, null, 2)}\n`;
  const current = await readFile(filePath, 'utf8');
  if (current === desired) {
    return;
  }

  const relativePath = path.relative(workspaceRoot, filePath).replaceAll('\\', '/');
  if (checkOnly) {
    pendingChanges.push(relativePath);
    return;
  }

  await writeFile(filePath, desired, 'utf8');
  console.log(`Aligned ${relativePath}`);
}

await alignFile(appOpenApiPath, appOperations);
await alignFile(backendOpenApiPath, backendOperations);

if (checkOnly && pendingChanges.length > 0) {
  console.error(
    JSON.stringify(
      {
        ok: false,
        pendingChanges,
      },
      null,
      2,
    ),
  );
  process.exit(1);
}

console.log(
  JSON.stringify(
    {
      ok: true,
      mode: checkOnly ? 'check' : 'apply',
    },
    null,
    2,
  ),
);
