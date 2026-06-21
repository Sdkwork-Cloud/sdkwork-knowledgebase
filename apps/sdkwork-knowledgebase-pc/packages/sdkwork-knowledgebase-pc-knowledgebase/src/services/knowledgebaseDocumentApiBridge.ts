import type { KnowledgeBrowserNode } from '@sdkwork/knowledgebase-app-sdk';
import * as KnowledgeSpaceSettingsService from './knowledgeSpaceSettingsService';
import {
  applyDriveBrowserNodeUpdates,
  deleteDriveBrowserNode,
} from './knowledgeDriveBrowserService';
import {
  enrichDocumentTreeMetadata,
  listDriveFavoriteNodeIds,
  resolveDriveNodeId,
  writeDriveDocumentOrder,
  writeDriveDocumentTags,
} from './knowledgeDriveDocumentMetadataService';
import {
  readOkfConceptTags,
  updateOkfConceptTags,
} from './knowledgeOkfDocumentMetadataService';
import {
  getKnowledgebaseAppSdkClient,
  getKnowledgebaseTenantId,
  isKnowledgebaseDriveApiAvailable,
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

function resolveBrowserDocumentId(node: KnowledgeBrowserNode, kbId: string): string {
  if (node.conceptId) {
    return `okf:${kbId}:${node.conceptId}`;
  }
  if (node.documentId) {
    return String(node.documentId);
  }
  return node.id;
}

function parseOkfDocumentId(id: string): { spaceId: number; conceptRowId: number } | null {
  const match = /^okf:(\d+):(\d+)$/.exec(id);
  if (!match) {
    return null;
  }
  const spaceId = Number(match[1]);
  const conceptRowId = Number(match[2]);
  if (!Number.isFinite(spaceId) || spaceId <= 0 || !Number.isFinite(conceptRowId) || conceptRowId <= 0) {
    return null;
  }
  return { spaceId, conceptRowId };
}

async function readOkfConceptMarkdown(spaceId: number, conceptRowId: number): Promise<string> {
  const client = requireSdkClient();
  const concept = await client.knowledge.okf.concepts.retrieve(conceptRowId);
  const result = await client.knowledge.retrievals.create({
    query: concept.conceptId,
    bindings: [{ spaceId: String(spaceId), priority: 0, topK: 3 }],
    includeCitations: false,
    includeTrace: false,
    topK: 3,
  });
  const hit =
    result.hits.find((entry) => entry.title === concept.title)
    ?? result.hits.find((entry) => entry.citation?.locator?.includes(concept.conceptId))
    ?? result.hits[0];
  if (hit?.content) {
    return hit.content;
  }
  return `# ${concept.title}\n\n${concept.description}`;
}

function mapNodeType(node: KnowledgeBrowserNode): DocumentMeta['type'] {
  if (node.nodeType === 'folder' || node.nodeType === 'virtual_folder') {
    return 'folder';
  }
  if (node.nodeType === 'okf_concept') {
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
    id: resolveBrowserDocumentId(node, kbId),
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
    avatar: space.avatar,
    isDeployed: space.isDeployed,
    deployedUrl: space.deployedUrl,
    customDomain: space.customDomain,
    siteName: space.siteName,
    siteLogo: space.siteLogo,
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

  await KnowledgeSpaceSettingsService.applyKnowledgeSpaceSettings(created.id, kb);

  const modelSettings = await KnowledgeSpaceSettingsService.loadKnowledgeSpaceModelSettings(created.id);
  const permissionSettings = await KnowledgeSpaceSettingsService.loadKnowledgeSpacePermissionSettings(created.id);

  return {
    id: String(created.id),
    title: created.name,
    icon: entry.icon,
    type: kbType,
    ...modelSettings,
    ...permissionSettings,
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

  if (updates.title && updates.title.trim() !== space.name) {
    await client.knowledge.spaces.update(spaceId, {
      name: updates.title.trim(),
    });
  }

  await KnowledgeSpaceSettingsService.applyKnowledgeSpaceSettings(spaceId, updates);

  const registryPatch: Parameters<typeof updateRegisteredSpace>[2] = {};
  if (updates.type !== undefined) {
    registryPatch.kbType = updates.type;
  }
  if (updates.icon !== undefined) {
    registryPatch.icon = updates.icon;
  }
  if (updates.avatar !== undefined) {
    registryPatch.avatar = updates.avatar;
  }
  if (updates.isDeployed !== undefined) {
    registryPatch.isDeployed = updates.isDeployed;
  }
  if (updates.deployedUrl !== undefined) {
    registryPatch.deployedUrl = updates.deployedUrl;
  }
  if (updates.customDomain !== undefined) {
    registryPatch.customDomain = updates.customDomain;
  }
  if (updates.siteName !== undefined) {
    registryPatch.siteName = updates.siteName;
  }
  if (updates.siteLogo !== undefined) {
    registryPatch.siteLogo = updates.siteLogo;
  }
  if (Object.keys(registryPatch).length > 0) {
    updateRegisteredSpace(tenantId, spaceId, registryPatch);
  }

  const modelSettings = await KnowledgeSpaceSettingsService.loadKnowledgeSpaceModelSettings(spaceId);
  const permissionSettings = await KnowledgeSpaceSettingsService.loadKnowledgeSpacePermissionSettings(spaceId);
  const registry = readRegisteredSpaces(tenantId).find((entry) => entry.spaceId === spaceId);
  const refreshedSpace = await client.knowledge.spaces.retrieve(spaceId);
  return {
    id,
    title: refreshedSpace.name,
    icon: registry?.icon ?? '📘',
    type: registry?.kbType ?? 'personal',
    avatar: registry?.avatar,
    isDeployed: registry?.isDeployed,
    deployedUrl: registry?.deployedUrl,
    customDomain: registry?.customDomain,
    siteName: registry?.siteName,
    siteLogo: registry?.siteLogo,
    ...modelSettings,
    ...permissionSettings,
  };
}

export async function hydrateKnowledgeBase(kb: KnowledgeBase): Promise<KnowledgeBase> {
  return KnowledgeSpaceSettingsService.hydrateKnowledgeBaseFromApi(kb);
}

export async function deleteKnowledgeBase(id: string): Promise<boolean> {
  const tenantId = requireTenantId();
  const spaceId = spaceIdFromKbId(id);
  const client = requireSdkClient();
  await client.knowledge.spaces.delete(spaceId);
  removeRegisteredSpace(tenantId, spaceId);
  return true;
}

export async function getDocuments(kbId: string): Promise<(FolderNode | DocumentMeta)[]> {
  const spaceId = spaceIdFromKbId(kbId);
  const nodes = await listBrowserNodes(spaceId);
  const grouped = groupDocumentsByParent(nodes, kbId);

  let favoriteNodeIds: Set<string> | undefined;
  if (isKnowledgebaseDriveApiAvailable()) {
    try {
      const client = requireSdkClient();
      const space = await client.knowledge.spaces.retrieve(spaceId);
      const driveSpaceId = space.driveSpaceId?.trim();
      if (driveSpaceId) {
        favoriteNodeIds = await listDriveFavoriteNodeIds(driveSpaceId);
      }
    } catch {
      // Pin state is optional when Drive favorites are unavailable.
    }
  }

  await enrichDocumentTreeMetadata(grouped, nodes, kbId, async (conceptRowId) => {
    try {
      return await readOkfConceptTags(conceptRowId);
    } catch {
      return undefined;
    }
  }, favoriteNodeIds);
  return grouped;
}

async function readIndexedDocumentContent(
  spaceId: number,
  documentId: string,
  title: string,
): Promise<string | null> {
  const client = requireSdkClient();
  const result = await client.knowledge.retrievals.create({
    query: title,
    bindings: [{ spaceId: String(spaceId), priority: 0, topK: 8 }],
    includeCitations: false,
    includeTrace: false,
    topK: 8,
  });
  const hit =
    result.hits.find((entry) => entry.documentId === documentId)
    ?? result.hits.find((entry) => entry.title === title)
    ?? result.hits[0];
  const content = hit?.content?.trim();
  return content ? content : null;
}

async function readBrowserNodeContent(
  node: KnowledgeBrowserNode,
  spaceId: number,
): Promise<string | null> {
  if (node.conceptId) {
    return readOkfConceptMarkdown(spaceId, node.conceptId);
  }
  if (node.documentId) {
    const indexed = await readIndexedDocumentContent(
      spaceId,
      String(node.documentId),
      node.name,
    );
    if (indexed) {
      return indexed;
    }
  }
  return null;
}

async function resolveBrowserNodeByDocumentId(
  documentId: string,
): Promise<{ spaceId: number; node: KnowledgeBrowserNode } | null> {
  const tenantId = getKnowledgebaseTenantId();
  if (!tenantId) {
    return null;
  }

  for (const entry of readRegisteredSpaces(tenantId)) {
    try {
      const nodes = await listBrowserNodes(entry.spaceId);
      const node = nodes.find((candidate) => resolveBrowserDocumentId(candidate, String(entry.spaceId)) === documentId)
        ?? nodes.find((candidate) => candidate.id === documentId);
      if (node) {
        return { spaceId: entry.spaceId, node };
      }
    } catch {
      // Skip spaces that fail to list.
    }
  }

  return null;
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

  const okfRef = parseOkfDocumentId(id);
  if (okfRef) {
    try {
      const content = await readOkfConceptMarkdown(okfRef.spaceId, okfRef.conceptRowId);
      writeLocalDocumentContent(tenantId, id, content);
      return content;
    } catch (error) {
      console.warn('[KnowledgebaseDocumentApiBridge] okf concept read failed.', error);
    }
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
      const indexed = await readIndexedDocumentContent(
        document.spaceId,
        String(document.id),
        document.title,
      );
      if (indexed) {
        writeLocalDocumentContent(tenantId, id, indexed);
        return indexed;
      }
      return `# ${document.title}\n\nThis document is indexed in Knowledgebase but no retrieval chunks were found yet.`;
    } catch {
      // Fall through to browser-node resolution.
    }
  }

  const browserMatch = await resolveBrowserNodeByDocumentId(id);
  if (browserMatch) {
    try {
      const content = await readBrowserNodeContent(browserMatch.node, browserMatch.spaceId);
      if (content) {
        writeLocalDocumentContent(tenantId, id, content);
        return content;
      }
    } catch (error) {
      console.warn('[KnowledgebaseDocumentApiBridge] browser node read failed.', error);
    }
  }

  return `# ${id}\n\nDocument content is managed by the Knowledgebase backend.`;
}

export async function saveDocumentContent(id: string, content: string): Promise<boolean> {
  const tenantId = requireTenantId();
  writeLocalDocumentContent(tenantId, id, content);

  const okfRef = parseOkfDocumentId(id);
  if (okfRef) {
    const client = requireSdkClient();
    const concept = await client.knowledge.okf.concepts.retrieve(okfRef.conceptRowId);
    await client.knowledge.okf.concepts.upsert({
      spaceId: okfRef.spaceId,
      conceptId: concept.conceptId,
      markdown: content,
      actor: 'pc-knowledgebase',
      publish: true,
    });
    return true;
  }

  const numericDocumentId = await resolveNumericDocumentId(id);
  if (!numericDocumentId) {
    throw new Error('Document content can only be persisted for indexed Knowledgebase documents.');
  }

  try {
    const client = requireSdkClient();
    const document = await client.knowledge.documents.retrieve(numericDocumentId);
    await client.knowledge.ingests.create({
      spaceId: document.spaceId,
      title: document.title,
      payloadMarkdown: content,
      idempotencyKey: `pc-save-${numericDocumentId}-${Date.now()}`.slice(0, 128),
    });
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    throw new Error(`Failed to persist document content to Knowledgebase: ${detail}`);
  }

  return true;
}

async function resolveNumericDocumentId(id: string): Promise<number | null> {
  const numericDocumentId = Number(id);
  if (Number.isFinite(numericDocumentId) && numericDocumentId > 0) {
    return numericDocumentId;
  }

  const browserMatch = await resolveBrowserNodeByDocumentId(id);
  if (browserMatch?.node.documentId) {
    return browserMatch.node.documentId;
  }

  return null;
}

export async function updateDocument(id: string, updates: Partial<DocumentMeta>): Promise<boolean> {
  if (updates.kbId !== undefined) {
    throw new Error(
      'Cross knowledge base document move is not supported by the Knowledgebase API yet.',
    );
  }

  const browserMatch = await resolveBrowserNodeByDocumentId(id);
  const okfRef = parseOkfDocumentId(id);
  let metadataUpdated = false;

  if (updates.tags !== undefined) {
    if (okfRef) {
      await updateOkfConceptTags(
        okfRef.spaceId,
        okfRef.conceptRowId,
        updates.tags,
        readOkfConceptMarkdown,
      );
      metadataUpdated = true;
    } else if (browserMatch) {
      const driveNodeId = resolveDriveNodeId(browserMatch.node);
      if (!driveNodeId || !isKnowledgebaseDriveApiAvailable()) {
        throw new Error('Drive SDK is required to update document tags.');
      }
      await writeDriveDocumentTags(driveNodeId, updates.tags);
      metadataUpdated = true;
    } else {
      throw new Error('Document tags can only be updated for Drive browser nodes or OKF concepts.');
    }
  }

  if (updates.order !== undefined) {
    if (!browserMatch) {
      throw new Error('Document order can only be updated for Knowledgebase browser nodes.');
    }
    const driveNodeId = resolveDriveNodeId(browserMatch.node);
    if (!driveNodeId || !isKnowledgebaseDriveApiAvailable()) {
      throw new Error('Drive SDK is required to update document order.');
    }
    await writeDriveDocumentOrder(driveNodeId, updates.order);
    metadataUpdated = true;
  }

  const hasDriveMetadataUpdate =
    updates.parentId !== undefined
    || updates.title !== undefined
    || updates.isPinned !== undefined;

  if (hasDriveMetadataUpdate) {
    if (!browserMatch) {
      throw new Error('Document tree metadata can only be updated for Knowledgebase browser nodes.');
    }
    if (!isKnowledgebaseDriveApiAvailable()) {
      throw new Error('Drive SDK is required to update document tree metadata.');
    }
    await applyDriveBrowserNodeUpdates(browserMatch.node, {
      title: updates.title,
      parentId: updates.parentId,
      isPinned: updates.isPinned,
    });
  }

  const numericDocumentId = await resolveNumericDocumentId(id);

  if (numericDocumentId && updates.title && !browserMatch) {
    const client = requireSdkClient();
    const existing = await client.knowledge.documents.retrieve(numericDocumentId);
    await client.knowledge.documents.update(numericDocumentId, {
      spaceId: existing.spaceId,
      title: updates.title ?? existing.title,
      mimeType: updates.type === 'markdown'
        ? 'text/markdown'
        : (existing.mimeType ?? 'text/plain'),
    });
  } else if (numericDocumentId && updates.type) {
    const client = requireSdkClient();
    const existing = await client.knowledge.documents.retrieve(numericDocumentId);
    await client.knowledge.documents.update(numericDocumentId, {
      spaceId: existing.spaceId,
      title: existing.title,
      mimeType: updates.type === 'markdown'
        ? 'text/markdown'
        : (existing.mimeType ?? 'text/plain'),
    });
  }

  if (updates.content !== undefined) {
    await saveDocumentContent(id, updates.content);
  }

  return metadataUpdated || hasDriveMetadataUpdate || numericDocumentId !== null || updates.content !== undefined;
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

    if (docs.length > 0) {
      return { kbs: matchedKbs, docs };
    }
  } catch (error) {
    console.warn('[KnowledgebaseDocumentApiBridge] retrieval search failed.', error);
  }

  const okfDocs = await searchOkfConceptDocs(trimmedQuery, registry);
  return { kbs: matchedKbs, docs: okfDocs };
}

async function searchOkfConceptDocs(
  query: string,
  registry: ReturnType<typeof readRegisteredSpaces>,
): Promise<DocumentMeta[]> {
  const client = requireSdkClient();
  const docs: DocumentMeta[] = [];

  for (const entry of registry.slice(0, 2)) {
    try {
      const result = await client.knowledge.okf.queries.create({
        spaceId: entry.spaceId,
        query,
      });
      const snippet = result.answerMarkdown.trim();
      if (!snippet) {
        continue;
      }
      docs.push({
        id: `okf:${entry.spaceId}:query-${docs.length}`,
        title: `OKF · ${query}`,
        type: 'markdown',
        kbId: String(entry.spaceId),
        updatedAt: new Date().toISOString(),
        author: 'Knowledgebase',
        content: snippet,
      });
    } catch {
      // Skip spaces without initialized OKF bundles.
    }
  }

  return docs;
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
          id: resolveBrowserDocumentId(node, String(entry.spaceId)),
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

  const okfRef = parseOkfDocumentId(id);
  if (okfRef) {
    try {
      const client = requireSdkClient();
      const concept = await client.knowledge.okf.concepts.retrieve(okfRef.conceptRowId);
      trackRecentDocument({
        id,
        title: concept.title,
        type: 'markdown',
        kbId: String(okfRef.spaceId),
        author: 'Knowledgebase',
      });
      return true;
    } catch {
      return false;
    }
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
  const tenantId = requireTenantId();
  const client = requireSdkClient();
  if (!doc.kbId) {
    throw new Error('kbId is required to create a document through the API.');
  }

  if (doc.type === 'folder') {
    throw new Error(
      'Folder creation is managed by the Knowledgebase drive browser tree; refresh the file list after drive import.',
    );
  }

  const created = await client.knowledge.documents.create({
    spaceId: spaceIdFromKbId(doc.kbId),
    title: doc.title?.trim() || 'Untitled',
    mimeType: doc.type === 'markdown' ? 'text/markdown' : 'text/plain',
  });

  const content = doc.content ?? '';
  const createdDoc: DocumentMeta = {
    id: String(created.id),
    title: created.title,
    type: doc.type ?? 'richtext',
    kbId: doc.kbId,
    parentId: doc.parentId ?? null,
    updatedAt: new Date().toISOString(),
    author: doc.author ?? 'Knowledgebase',
    content,
  };

  if (content) {
    writeLocalDocumentContent(tenantId, createdDoc.id, content);
    await client.knowledge.ingests.create({
      spaceId: created.spaceId,
      title: created.title,
      payloadMarkdown: content,
      idempotencyKey: `pc-create-${created.id}`,
    });
  }

  trackRecentDocument(createdDoc);
  return createdDoc;
}

export async function deleteDocument(id: string): Promise<boolean> {
  const tenantId = requireTenantId();
  removeLocalDocumentContent(tenantId, id);
  removeRecentDocument(tenantId, id);

  if (parseOkfDocumentId(id)) {
    const okfRef = parseOkfDocumentId(id)!;
    const client = requireSdkClient();
    await client.knowledge.okf.concepts.delete(okfRef.conceptRowId);
    return true;
  }

  const browserMatch = await resolveBrowserNodeByDocumentId(id);
  const numericDocumentId = await resolveNumericDocumentId(id);
  let deleted = false;

  if (numericDocumentId) {
    const client = requireSdkClient();
    await client.knowledge.documents.delete(numericDocumentId);
    deleted = true;
  }

  if (browserMatch && isKnowledgebaseDriveApiAvailable()) {
    const isFolder =
      browserMatch.node.nodeType === 'folder'
      || browserMatch.node.nodeType === 'virtual_folder';
    const canDeleteDriveNode =
      isFolder
      || browserMatch.node.driveNodeId
      || !numericDocumentId;
    if (canDeleteDriveNode) {
      await deleteDriveBrowserNode(browserMatch.node);
      deleted = true;
    }
  }

  if (!deleted) {
    throw new Error('Only indexed Knowledgebase documents or Drive browser nodes can be deleted through the API.');
  }

  return true;
}
