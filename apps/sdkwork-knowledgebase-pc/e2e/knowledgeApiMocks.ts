import type { Page, Route } from '@playwright/test';

const E2E_SPACE = {
  id: 1,
  uuid: '00000000-0000-0000-0000-000000000001',
  name: 'E2E Knowledge Base',
  description: null,
  driveSpaceId: 'drv-e2e-1',
  status: 'active',
  okfBundleInitialized: true,
  knowledgeMode: 'hybrid',
};

export const E2E_SOURCE_DOCUMENT = {
  id: '1001',
  numericId: 1001,
  title: 'E2E Source Document',
  content: 'Launch readiness verification content for retrieval citations.',
};

export const E2E_SESSION_STORAGE_KEY = 'sdkwork-knowledgebase-pc-session';
export const E2E_SPACE_REGISTRY_KEY = 'sdkwork.knowledgebase.spaces.v1.1';

export interface E2eMockTelemetry {
  ingestPayloads: string[];
  createdDocumentIds: number[];
}

export function createE2eMockTelemetry(): E2eMockTelemetry {
  return {
    ingestPayloads: [],
    createdDocumentIds: [],
  };
}

export function buildE2eSessionPayload() {
  return {
    authToken: 'e2e-auth-token',
    accessToken: 'e2e-access-token',
    context: {
      tenantId: '1',
      userId: '42',
      actorId: '42',
    },
    user: {
      id: '42',
      displayName: 'E2E User',
      email: 'e2e@sdkwork.local',
    },
  };
}

export function buildE2eSpaceRegistryPayload() {
  return [
    {
      spaceId: 1,
      kbType: 'personal',
      icon: '📘',
      createdAt: '2026-01-01T00:00:00.000Z',
    },
  ];
}

export async function seedKnowledgebaseE2eSession(page: Page): Promise<void> {
  const session = buildE2eSessionPayload();
  const registry = buildE2eSpaceRegistryPayload();

  await page.addInitScript(
    ({ sessionKey, registryKey, sessionValue, registryValue }) => {
      const sessionJson = JSON.stringify(sessionValue);
      const registryJson = JSON.stringify(registryValue);
      window.localStorage.setItem(sessionKey, sessionJson);
      window.localStorage.setItem(registryKey, registryJson);
      window.sessionStorage.setItem(sessionKey, sessionJson);
      window.sessionStorage.setItem(registryKey, registryJson);
    },
    {
      sessionKey: E2E_SESSION_STORAGE_KEY,
      registryKey: E2E_SPACE_REGISTRY_KEY,
      sessionValue: session,
      registryValue: registry,
    },
  );
}

function json(route: Route, status: number, body: unknown) {
  return route.fulfill({
    status,
    contentType: 'application/json',
    body: JSON.stringify(body),
  });
}

function createMockState(telemetry?: E2eMockTelemetry) {
  let nextDocumentId = E2E_SOURCE_DOCUMENT.numericId;
  let nextIngestId = 1;
  const agentProfileId = 'e2e-agent-profile-1';
  const documents = new Map<number, { id: number; spaceId: number; title: string; mimeType: string }>();

  documents.set(E2E_SOURCE_DOCUMENT.numericId, {
    id: E2E_SOURCE_DOCUMENT.numericId,
    spaceId: 1,
    title: E2E_SOURCE_DOCUMENT.title,
    mimeType: 'text/markdown',
  });

  return {
    agentProfileId,
    documents,
    nextDocumentId,
    nextIngestId,
    telemetry,
  };
}

type MockState = ReturnType<typeof createMockState>;

function buildRetrievalHits(query: string) {
  const normalized = query.trim().toLowerCase();
  if (!normalized) {
    return [];
  }

  return [
    {
      documentId: E2E_SOURCE_DOCUMENT.id,
      spaceId: '1',
      title: E2E_SOURCE_DOCUMENT.title,
      content: E2E_SOURCE_DOCUMENT.content,
      retrievalMethod: 'vector',
      score: 0.92,
      citation: {
        sourceUri: null,
      },
    },
  ];
}

async function handleKnowledgeRoute(route: Route, state: MockState): Promise<void> {
  const request = route.request();
  const url = new URL(request.url());
  const { pathname } = url;
  const method = request.method().toUpperCase();

  if (method === 'POST' && /\/knowledge\/spaces\/?$/.test(pathname)) {
    const body = request.postDataJSON() as { name?: string; description?: string | null } | null;
    await json(route, 200, {
      id: E2E_SPACE.id,
      uuid: E2E_SPACE.uuid,
      name: body?.name?.trim() || E2E_SPACE.name,
      description: body?.description ?? E2E_SPACE.description,
      driveSpaceId: E2E_SPACE.driveSpaceId,
      status: E2E_SPACE.status,
      okfBundleInitialized: E2E_SPACE.okfBundleInitialized,
      knowledgeMode: E2E_SPACE.knowledgeMode,
    });
    return;
  }

  if (method === 'GET' && /\/knowledge\/spaces\/\d+$/.test(pathname)) {
    await json(route, 200, E2E_SPACE);
    return;
  }

  if (method === 'GET' && /\/knowledge\/spaces\/\d+\/browser$/.test(pathname)) {
    await json(route, 200, {
      spaceId: 1,
      view: 'okf_bundle',
      items: [],
      nextCursor: null,
    });
    return;
  }

  if (method === 'GET' && /\/knowledge\/spaces\/\d+\/context_bindings$/.test(pathname)) {
    await json(route, 200, { items: [] });
    return;
  }

  if (method === 'GET' && /\/knowledge\/spaces\/\d+\/members$/.test(pathname)) {
    await json(route, 200, { members: [] });
    return;
  }

  if (method === 'GET' && pathname.endsWith('/knowledge/agent_profiles')) {
    await json(route, 200, { items: [{ profileId: state.agentProfileId }] });
    return;
  }

  if (method === 'POST' && pathname.endsWith('/knowledge/agent_profiles')) {
    await json(route, 200, {
      profileId: state.agentProfileId,
      name: 'E2E Agent Profile',
      knowledgeMode: 'okf_bundle',
      modelProviderId: 'provider.model.knowledgebase-contract',
      modelId: 'contract',
      agentImplementationId: 'plugin.intelligence.knowledgebase-contract',
      status: 'active',
    });
    return;
  }

  if (method === 'GET' && /\/knowledge\/agent_profiles\/[^/]+$/.test(pathname)) {
    await json(route, 200, {
      profileId: state.agentProfileId,
      name: 'E2E Agent Profile',
      knowledgeMode: 'okf_bundle',
      modelProviderId: 'provider.model.knowledgebase-contract',
      modelId: 'contract',
      agentImplementationId: 'plugin.intelligence.knowledgebase-contract',
      status: 'active',
    });
    return;
  }

  if (method === 'POST' && /\/knowledge\/agent_profiles\/[^/]+\/bindings$/.test(pathname)) {
    await json(route, 200, { profileId: state.agentProfileId, spaceId: '1', enabled: true });
    return;
  }

  if (method === 'POST' && /\/knowledge\/agent_profiles\/[^/]+\/chat$/.test(pathname)) {
    const body = request.postDataJSON() as { message?: string } | null;
    await json(route, 200, {
      answer: `E2E synthesized answer for: ${body?.message ?? 'query'}`,
      sessionId: 'e2e-session-1',
    });
    return;
  }

  if (method === 'POST' && pathname.endsWith('/knowledge/documents')) {
    const body = request.postDataJSON() as { title?: string; spaceId?: number; mimeType?: string } | null;
    state.nextDocumentId += 1;
    const created = {
      id: state.nextDocumentId,
      spaceId: body?.spaceId ?? 1,
      title: body?.title?.trim() || 'New Document',
      mimeType: body?.mimeType ?? 'text/plain',
      visibility: 'private',
      language: 'en',
    };
    state.documents.set(created.id, created);
    state.telemetry?.createdDocumentIds.push(created.id);
    await json(route, 200, created);
    return;
  }

  if (method === 'GET' && /\/knowledge\/documents\/\d+$/.test(pathname)) {
    const documentId = Number(pathname.split('/').pop());
    const document = state.documents.get(documentId);
    if (!document) {
      await json(route, 404, { message: 'document not found' });
      return;
    }
    await json(route, 200, {
      ...document,
      visibility: 'private',
      language: 'en',
    });
    return;
  }

  if (method === 'POST' && pathname.endsWith('/knowledge/ingests')) {
    const body = request.postDataJSON() as { payloadMarkdown?: string } | null;
    if (body?.payloadMarkdown) {
      state.telemetry?.ingestPayloads.push(body.payloadMarkdown);
    }
    const ingestId = state.nextIngestId;
    state.nextIngestId += 1;
    await json(route, 200, {
      id: ingestId,
      state: 'succeeded',
      errorMessage: null,
    });
    return;
  }

  if (method === 'GET' && /\/knowledge\/ingests\/\d+$/.test(pathname)) {
    await json(route, 200, {
      id: Number(pathname.split('/').pop()),
      state: 'succeeded',
      errorMessage: null,
    });
    return;
  }

  if (method === 'POST' && pathname.endsWith('/knowledge/retrievals')) {
    const body = request.postDataJSON() as { query?: string } | null;
    await json(route, 200, {
      retrievalId: '1',
      hits: buildRetrievalHits(body?.query ?? ''),
    });
    return;
  }

  if (method === 'POST' && pathname.endsWith('/knowledge/okf/queries')) {
    const body = request.postDataJSON() as { query?: string } | null;
    await json(route, 200, {
      answerMarkdown: `### E2E retrieval insight\n\nBased on **《${E2E_SOURCE_DOCUMENT.title}》**, ${body?.query ?? 'your question'} aligns with launch readiness verification.\n\n- Cite [1] for source navigation\n- Follow-up: drill backup restore runbook`,
    });
    return;
  }

  if (method === 'GET' && pathname.endsWith('/knowledge/agent_profiles')) {
    await json(route, 200, { items: [] });
    return;
  }

  await json(route, 200, {});
}

export async function mockKnowledgebaseAppApi(
  page: Page,
  telemetry: E2eMockTelemetry = createE2eMockTelemetry(),
): Promise<E2eMockTelemetry> {
  const state = createMockState(telemetry);

  await page.route('**/app/v3/api/knowledge/**', async (route) => {
    await handleKnowledgeRoute(route, state);
  });

  await page.route('**/app/v3/api/iam/**', async (route) => {
    await json(route, 200, {
      id: '42',
      displayName: 'E2E User',
      email: 'e2e@sdkwork.local',
    });
  });

  await page.route('**/app/v3/api/drive/**', async (route) => {
    await json(route, 200, { items: [], nodes: [], nextCursor: null });
  });

  await page.route('**/app/v3/api/**', async (route) => {
    await json(route, 200, {});
  });

  return telemetry;
}

export async function setupKnowledgebaseE2ePage(page: Page, telemetry?: E2eMockTelemetry) {
  await seedKnowledgebaseE2eSession(page);
  return mockKnowledgebaseAppApi(page, telemetry ?? createE2eMockTelemetry());
}
