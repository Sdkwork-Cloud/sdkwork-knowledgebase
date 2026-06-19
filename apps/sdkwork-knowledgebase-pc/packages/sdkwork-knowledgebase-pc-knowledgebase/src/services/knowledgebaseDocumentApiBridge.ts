import type { KnowledgeBrowserNode } from '@sdkwork/knowledgebase-app-sdk';
import {
  getKnowledgebaseAppSdkClient,
  getKnowledgebaseTenantId,
  readLocalDocumentContent,
  readRecentDocuments,
  readRegisteredSpaces,
  removeLocalDocumentContent,
  removeRecentDocument,
  removeRegisteredSpace,
  type KnowledgebaseSpaceKbType,
  type RegisteredKnowledgebaseSpace,
  touchRecentDocument,
  updateRegisteredSpace,
  upsertRegisteredSpace,
  writeLocalDocumentContent,
} from 'sdkwork-knowledgebase-pc-core';

import type { DocumentMeta, FolderNode, KnowledgeBase } from './document';

function requireTenantId(): string {
  const tenantId = getKnowledgebaseTenantId();
  if (!tenantId) {
    throw new Error('Knowledgebase tenant context is required for API-backed document operations.');
  }
  return tenantId;
}

function requireSdkClient() {
  const sdk = getKnowledgebaseAppSdkClient();
  if (!sdk) {
    throw new Error('Knowledgebase app SDK is not configured.');
  }
  return sdk.client;
}

function spaceIdFromKbId(kbId: string): number {
  const spaceId = Number(kbId);
  if (!Number.isFinite(spaceId) || spaceId <= 0) {
    throw new Error(`Invalid knowledge space id: ${kbId}`);
  }
  return spaceId;
}

function formatBytes(bytes: number | null | undefined): string | undefined {
  if (!bytes || bytes <= 0) {
    return undefined;
  }
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  if (bytes < 1024 * 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`;
  }
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function mapNodeType(node: KnowledgeBrowserNode): DocumentMeta['type'] {
  if (node.nodeType === 'folder' || node.nodeType === 'virtual_folder') {
    return 'folder';
  }
  if (node.nodeType === 'wiki_page') {
    return 'markdown';
  }

  const mime = node.mimeType ?? '';
  if (mime.startsWith('image/')) {
    return 'image';
  }
  if (mime.startsWith('video/')) {
    return 'video';
  }
  if (mime.startsWith('audio/')) {
    return 'audio';
  }
  if (mime.includes('pdf') || node.name.toLowerCase().endsWith('.pdf')) {
    return 'pdf';
  }
  if (mime.includes('markdown') || node.name.toLowerCase().endsWith('.md')) {
    return 'markdown';
  }
  if (/\.(ts|tsx|js|jsx|html|htm|css|json|xml|py|java|go|rs)$/i.test(node.name)) {
    return 'code';
  }
  return 'file';
}

function mapBrowserNodeToDocument(node: KnowledgeBrowserNode, kbId: string): DocumentMeta | FolderNode {
  const base: DocumentMeta = {
    id: node.id,
    title: node.name,
    type: mapNodeType(node),
    kbId,
    parentId: node.parentId ?? null,
    updatedAt: node.updatedAt,
    author: 'Knowledgebase',
    size: formatBytes(node.sizeBytes ?? undefined),
  };

  if (base.type === 'folder') {
    return {
      ...base,
      type: 'folder',
      children: [],
    };
  }

  return base;
}

function buildKnowledgeBase(
  space: RegisteredKnowledgebaseSpace,
  name: string,
): KnowledgeBase {
  return {
    id: String(space.spaceId),
    title: name,
    icon: space.icon ?? '📘',
    type: space.kbType,
  };
}

async function listBrowserNodes(spaceId: number, parentId?: string | null) {
  const client = requireSdkClient();
  const items: KnowledgeBrowserNode[] = [];
  let cursor: string | null | undefined;

  do {
    const page = await client.knowledge.spaces.browser.list(spaceId, {
      view: 'files',
      parentId: parentId ?? null,
      cursor,
      pageSize: 100,
    });
    items.push(...page.items);
    cursor = page.nextCursor;
  } while (cursor);

  return items;
}

function groupDocumentsByParent(
  nodes: KnowledgeBrowserNode[],
  kbId: string,
): (FolderNode | DocumentMeta)[] {
  const map = new Map<string, FolderNode | DocumentMeta>();
  const roots: (FolderNode | DocumentMeta)[] = [];

  for (const node of nodes) {
    map.set(node.id, mapBrowserNodeToDocument(node, kbId));
  }

  for (const node of nodes) {
    const current = map.get(node.id);
    if (!current) {
      continue;
    }

    if (node.parentId && map.has(node.parentId)) {
      const parent = map.get(node.parentId);
      if (parent && parent.type === 'folder') {
        const folderParent = parent as FolderNode;
        folderParent.children = folderParent.children ?? [];
        folderParent.children.push(current);
      }
    } else {
      roots.push(current);
    }
  }

  return roots;
}

async function ensureDefaultSpace(tenantId: string): Promise<RegisteredKnowledgebaseSpace> {
  const client = requireSdkClient();
  const created = await client.knowledge.spaces.create({
    name: '我的知识库',
    description: 'Default knowledge space',
  });
  const entry: RegisteredKnowledgebaseSpace = {
    spaceId: created.id,
    kbType: 'personal',
    icon: '📓',
    createdAt: new Date().toISOString(),
  };
  upsertRegisteredSpace(tenantId, entry);
  return entry;
}

export async function getKnowledgeBases(): Promise<{
  team: KnowledgeBase[];
  personal: KnowledgeBase[];
  public: KnowledgeBase[];
}> {
  const tenantId = requireTenantId();
  const client = requireSdkClient();
  let registry = readRegisteredSpaces(tenantId);

  if (registry.length === 0) {
    registry = [await ensureDefaultSpace(tenantId)];
  }

  const grouped: Record<KnowledgebaseSpaceKbType, KnowledgeBase[]> = {
    team: [],
    personal: [],
    public: [],
  };

  for (const entry of registry) {
    try {
      const space = await client.knowledge.spaces.retrieve(entry.spaceId);
      grouped[entry.kbType].push(buildKnowledgeBase(entry, space.name));
    } catch {
      removeRegisteredSpace(tenantId, entry.spaceId);
    }
  }

  return grouped;
}

export async function createKnowledgeBase(
  kb: Partial<KnowledgeBase>,
): Promise<KnowledgeBase> {
  const tenantId = requireTenantId();
  const client = requireSdkClient();
  const kbType = kb.type ?? 'personal';

  const created = await client.knowledge.spaces.create({
    name: kb.title?.trim() || 'Untitled',
    description: null,
  });

  const entry: RegisteredKnowledgebaseSpace = {
    spaceId: created.id,
    kbType,
    icon: kb.icon ?? '📁',
    createdAt: new Date().toISOString(),
  };
  upsertRegisteredSpace(tenantId, entry);

  return {
    id: String(created.id),
    title: created.name,
    icon: entry.icon,
    type: kbType,
    provider: kb.provider,
    modelName: kb.modelName,
    temperature: kb.temperature,
    maxTokens: kb.maxTokens,
    systemPrompt: kb.systemPrompt,
    publicPermission: kb.publicPermission,
    guestLinkEnabled: kb.guestLinkEnabled,
  };
}

export async function updateKnowledgeBase(
  id: string,
  updates: Partial<KnowledgeBase>,
): Promise<KnowledgeBase> {
  const tenantId = requireTenantId();
  const spaceId = spaceIdFromKbId(id);
  const client = requireSdkClient();
  const space = await client.knowledge.spaces.retrieve(spaceId);

  if (updates.type || updates.icon) {
    updateRegisteredSpace(tenantId, spaceId, {
      kbType: updates.type,
      icon: updates.icon,
    });
  }

  const registry = readRegisteredSpaces(tenantId).find((entry) => entry.spaceId === spaceId);
  return {
    id,
    title: updates.title ?? space.name,
    icon: updates.icon ?? registry?.icon ?? '📘',
    type: updates.type ?? registry?.kbType ?? 'personal',
    ...updates,
  };
}

export async function deleteKnowledgeBase(id: string): Promise<boolean> {
  const tenantId = requireTenantId();
  removeRegisteredSpace(tenantId, spaceIdFromKbId(id));
  return true;
}

export async function getDocuments(kbId: string): Promise<(FolderNode | DocumentMeta)[]> {
  const nodes = await listBrowserNodes(spaceIdFromKbId(kbId));
  return groupDocumentsByParent(nodes, kbId);
}

export async function getDocumentContent(id: string): Promise<string> {
  const tenantId = requireTenantId();
  const cached = readLocalDocumentContent(tenantId, id);
  if (cached !== undefined) {
    const recent = readRecentDocuments(tenantId).find((entry) => entry.id === id);
    if (recent) {
      touchRecentDocument(tenantId, recent);
    }
    return cached;
  }

  const client = requireSdkClient();
  const numericDocumentId = Number(id);
  if (Number.isFinite(numericDocumentId) && numericDocumentId > 0) {
    try {
      const document = await client.knowledge.documents.retrieve(numericDocumentId);
      trackRecentDocument({
        id: String(document.id),
        title: document.title,
        type: 'markdown',
        kbId: String(document.spaceId),
        author: 'Knowledgebase',
      });
      return `# ${document.title}\n\nThis document is indexed in Knowledgebase. Content is served from the bound drive object (mime: ${document.mimeType ?? 'unknown'}).`;
    } catch {
      // Fall through to browser-node placeholder.
    }
  }

  return `# ${id}\n\nDocument content is managed by the Knowledgebase backend.`;
}

export async function saveDocumentContent(id: string, content: string): Promise<boolean> {
  const tenantId = requireTenantId();
  writeLocalDocumentContent(tenantId, id, content);

  const numericDocumentId = Number(id);
  if (!Number.isFinite(numericDocumentId) || numericDocumentId <= 0) {
    return true;
  }

  try {
    const client = requireSdkClient();
    const document = await client.knowledge.documents.retrieve(numericDocumentId);
    await client.knowledge.ingests.create({
      spaceId: document.spaceId,
      title: document.title,
      payloadMarkdown: content,
      idempotencyKey: `pc-save-${numericDocumentId}`,
    });
  } catch (error) {
    console.warn('[KnowledgebaseDocumentApiBridge] ingest save failed; content kept in local cache.', error);
  }

  return true;
}

export async function updateDocument(id: string, updates: Partial<DocumentMeta>): Promise<boolean> {
  const numericDocumentId = Number(id);
  if (!Number.isFinite(numericDocumentId) || numericDocumentId <= 0) {
    return false;
  }

  const client = requireSdkClient();
  const existing = await client.knowledge.documents.retrieve(numericDocumentId);

  if (updates.title || updates.type) {
    await client.knowledge.documents.update(numericDocumentId, {
      spaceId: existing.spaceId,
      title: updates.title ?? existing.title,
      mimeType: updates.type === 'markdown'
        ? 'text/markdown'
        : (existing.mimeType ?? 'text/plain'),
    });
  }

  if (updates.content !== undefined) {
    await saveDocumentContent(id, updates.content);
  }

  return true;
}

export async function searchAll(query: string): Promise<{
  kbs: KnowledgeBase[];
  docs: DocumentMeta[];
}> {
  const tenantId = requireTenantId();
  const client = requireSdkClient();
  const trimmedQuery = query.trim();
  if (!trimmedQuery) {
    return { kbs: [], docs: [] };
  }

  const grouped = await getKnowledgeBases();
  const allKbs = [...grouped.team, ...grouped.personal, ...grouped.public];
  const lowerQuery = trimmedQuery.toLowerCase();
  const matchedKbs = allKbs.filter((kb) => kb.title.toLowerCase().includes(lowerQuery));

  const registry = readRegisteredSpaces(tenantId);
  if (registry.length === 0) {
    return { kbs: matchedKbs, docs: [] };
  }

  const bindings = registry.map((entry, index) => ({
    spaceId: String(entry.spaceId),
    priority: index,
    topK: 20,
  }));

  try {
    const result = await client.knowledge.retrievals.create({
      tenantId,
      query: trimmedQuery,
      bindings,
      includeCitations: true,
      includeTrace: false,
      topK: 20,
    });

    const docs: DocumentMeta[] = [];
    const seenDocumentIds = new Set<string>();

    for (const hit of result.hits) {
      if (seenDocumentIds.has(hit.documentId)) {
        continue;
      }
      seenDocumentIds.add(hit.documentId);

      docs.push({
        id: hit.documentId,
        title: hit.title,
        type: 'markdown',
        kbId: hit.spaceId,
        updatedAt: new Date().toISOString(),
        author: 'Knowledgebase',
        content: hit.content,
      });
    }

    return { kbs: matchedKbs, docs };
  } catch (error) {
    console.warn('[KnowledgebaseDocumentApiBridge] retrieval search failed.', error);
    return { kbs: matchedKbs, docs: [] };
  }
}

function trackRecentDocument(doc: Pick<DocumentMeta, 'id' | 'title' | 'type' | 'kbId' | 'author'>): void {
  const tenantId = getKnowledgebaseTenantId();
  if (!tenantId || doc.type === 'folder') {
    return;
  }

  touchRecentDocument(tenantId, {
    id: doc.id,
    title: doc.title,
    type: doc.type,
    kbId: doc.kbId,
    author: doc.author,
    updatedAt: new Date().toISOString(),
  });
}

async function collectRecentDocumentsFromSpaces(limit: number): Promise<DocumentMeta[]> {
  const tenantId = requireTenantId();
  const registry = readRegisteredSpaces(tenantId);
  const collected: DocumentMeta[] = [];

  for (const entry of registry) {
    try {
      const nodes = await listBrowserNodes(entry.spaceId);
      for (const node of nodes) {
        if (node.nodeType === 'folder' || node.nodeType === 'virtual_folder') {
          continue;
        }
        collected.push({
          id: node.id,
          title: node.name,
          type: mapNodeType(node),
          kbId: String(entry.spaceId),
          parentId: node.parentId ?? null,
          updatedAt: node.updatedAt,
          author: 'Knowledgebase',
          size: formatBytes(node.sizeBytes ?? undefined),
        });
      }
    } catch {
      // Skip spaces that fail to list.
    }
  }

  return collected
    .sort((left, right) => new Date(right.updatedAt || 0).getTime() - new Date(left.updatedAt || 0).getTime())
    .slice(0, limit);
}

export async function getRecentDocuments(limit: number = 8): Promise<DocumentMeta[]> {
  const tenantId = requireTenantId();
  const cached = readRecentDocuments(tenantId)
    .filter((entry) => entry.type !== 'folder')
    .slice(0, limit)
    .map((entry) => ({
      id: entry.id,
      title: entry.title,
      type: entry.type,
      kbId: entry.kbId,
      updatedAt: entry.updatedAt,
      author: entry.author ?? 'Knowledgebase',
    }));

  if (cached.length >= limit) {
    return cached;
  }

  const remote = await collectRecentDocumentsFromSpaces(limit);
  const merged = new Map<string, DocumentMeta>();
  for (const doc of [...cached, ...remote]) {
    merged.set(doc.id, doc);
  }

  return Array.from(merged.values())
    .sort((left, right) => new Date(right.updatedAt || 0).getTime() - new Date(left.updatedAt || 0).getTime())
    .slice(0, limit);
}

export async function touchDocument(id: string): Promise<boolean> {
  const tenantId = requireTenantId();
  const existing = readRecentDocuments(tenantId).find((entry) => entry.id === id);
  if (existing) {
    touchRecentDocument(tenantId, existing);
    return true;
  }

  const numericDocumentId = Number(id);
  if (!Number.isFinite(numericDocumentId) || numericDocumentId <= 0) {
    return false;
  }

  try {
    const client = requireSdkClient();
    const document = await client.knowledge.documents.retrieve(numericDocumentId);
    trackRecentDocument({
      id: String(document.id),
      title: document.title,
      type: 'markdown',
      kbId: String(document.spaceId),
      author: 'Knowledgebase',
    });
    return true;
  } catch {
    return false;
  }
}

export async function createDocument(doc: Partial<DocumentMeta>): Promise<DocumentMeta> {
  const client = requireSdkClient();
  if (!doc.kbId) {
    throw new Error('kbId is required to create a document through the API.');
  }

  const created = await client.knowledge.documents.create({
    spaceId: spaceIdFromKbId(doc.kbId),
    title: doc.title?.trim() || 'Untitled',
    mimeType: doc.type === 'markdown' ? 'text/markdown' : 'text/plain',
  });

  const createdDoc: DocumentMeta = {
    id: String(created.id),
    title: created.title,
    type: doc.type ?? 'richtext',
    kbId: doc.kbId,
    parentId: doc.parentId ?? null,
    updatedAt: new Date().toISOString(),
    author: doc.author ?? 'Knowledgebase',
    content: doc.content ?? '',
  };
  trackRecentDocument(createdDoc);
  return createdDoc;
}

export async function deleteDocument(id: string): Promise<boolean> {
  const tenantId = requireTenantId();
  removeLocalDocumentContent(tenantId, id);
  removeRecentDocument(tenantId, id);

  const client = requireSdkClient();
  const numericDocumentId = Number(id);
  if (!Number.isFinite(numericDocumentId) || numericDocumentId <= 0) {
    return false;
  }
  await client.knowledge.documents.delete(numericDocumentId);
  return true;
}
