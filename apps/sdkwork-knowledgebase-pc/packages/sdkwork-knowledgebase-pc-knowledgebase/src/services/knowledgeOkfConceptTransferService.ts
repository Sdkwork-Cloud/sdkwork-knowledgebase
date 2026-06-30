import type { KnowledgeBrowserNode } from 'sdkwork-knowledgebase-pc-core';
import {
  KnowledgebaseErrorCodes,
  parseKnowledgeSpaceId,
  requireKnowledgebaseAppSdkHttpClient,
  throwKnowledgebaseError,
} from 'sdkwork-knowledgebase-pc-core';

import type { DocumentMeta } from './document';
import {
  listAllKnowledgeBrowserNodes,
  invalidateKnowledgeBrowserNodeCacheForKbIds,
  invalidateKnowledgeBrowserNodeCacheForSpaceIds,
} from './knowledgeBrowserListService';

function requireSdkClient() {
  return requireKnowledgebaseAppSdkHttpClient();
}

function spaceIdFromKbId(kbId: string): number {
  return parseKnowledgeSpaceId(kbId);
}

function buildCopiedConceptId(conceptId: string): string {
  const suffix = `-copy-${Date.now().toString(36)}`;
  const maxLength = 240;
  if (conceptId.length + suffix.length <= maxLength) {
    return `${conceptId}${suffix}`;
  }
  return `${conceptId.slice(0, maxLength - suffix.length)}${suffix}`;
}

function patchOkfMarkdownTitle(markdown: string, title: string): string {
  if (!markdown.startsWith('---')) {
    return `# ${title}\n\n${markdown}`;
  }

  const closingIndex = markdown.indexOf('\n---', 3);
  if (closingIndex < 0) {
    return markdown;
  }

  let frontmatter = markdown.slice(4, closingIndex);
  if (/^title:\s/m.test(frontmatter)) {
    frontmatter = frontmatter.replace(/^title:\s.*$/m, `title: ${title}`);
  } else {
    frontmatter = `title: ${title}\n${frontmatter}`;
  }

  return `---\n${frontmatter}---${markdown.slice(closingIndex + 4)}`;
}

function normalizeBrowserPath(path: string): string {
  return path.replace(/^\/+/, '');
}

function okfConceptLogicalPath(conceptId: string): string {
  return `okf/${conceptId}.md`;
}

function matchOkfConceptNode(
  node: KnowledgeBrowserNode,
  expectedLogicalPath: string,
): node is KnowledgeBrowserNode & { conceptId: number } {
  if (node.nodeType !== 'okf_concept' || !node.conceptId) {
    return false;
  }
  const nodePath = normalizeBrowserPath(node.path);
  return nodePath === expectedLogicalPath || nodePath.endsWith(`/${expectedLogicalPath}`);
}

async function resolveOkfConceptRowIdByConceptId(
  spaceId: number,
  conceptIdString: string,
  logicalPath?: string,
): Promise<number> {
  const expectedLogicalPath = normalizeBrowserPath(
    logicalPath ?? okfConceptLogicalPath(conceptIdString),
  );

  for (let attempt = 0; attempt < 2; attempt += 1) {
    if (attempt > 0) {
      await new Promise((resolve) => setTimeout(resolve, 250));
    }

    const nodes = await listAllKnowledgeBrowserNodes(spaceId);
    const matched = nodes.find((node) => matchOkfConceptNode(node, expectedLogicalPath));
    if (matched?.conceptId) {
      return matched.conceptId;
    }
  }

  const client = requireSdkClient();
  const nodes = await listAllKnowledgeBrowserNodes(spaceId);
  for (const node of nodes) {
    if (node.nodeType !== 'okf_concept' || !node.conceptId) {
      continue;
    }
    const summary = await client.knowledge.okf.concepts.retrieve(node.conceptId);
    if (summary.conceptId === conceptIdString) {
      return node.conceptId;
    }
  }

  throwKnowledgebaseError(KnowledgebaseErrorCodes.DOCUMENT_RESOLVE_FAILED, {
    cause: conceptIdString,
  });
}

export async function copyOkfConcept(
  sourceSpaceId: number,
  sourceConceptRowId: number,
  targetKbId: string,
  readMarkdown: (spaceId: number, conceptRowId: number) => Promise<string>,
  options?: { titleSuffix?: string },
): Promise<DocumentMeta> {
  const client = requireSdkClient();
  const targetSpaceId = spaceIdFromKbId(targetKbId);
  const sourceConcept = await client.knowledge.okf.concepts.retrieve(sourceConceptRowId);
  const titleSuffix = options?.titleSuffix ?? ' (Copy)';
  const targetTitle = `${sourceConcept.title}${titleSuffix}`;
  const targetConceptId = buildCopiedConceptId(sourceConcept.conceptId);
  const markdown = patchOkfMarkdownTitle(
    await readMarkdown(sourceSpaceId, sourceConceptRowId),
    targetTitle,
  );

  const upserted = await client.knowledge.okf.concepts.upsert({
    spaceId: targetSpaceId,
    conceptId: targetConceptId,
    markdown,
    actor: 'pc-knowledgebase',
    publish: true,
  });

  const targetConceptRowId = await resolveOkfConceptRowIdByConceptId(
    targetSpaceId,
    targetConceptId,
    upserted.logicalPath,
  );
  invalidateKnowledgeBrowserNodeCacheForSpaceIds(sourceSpaceId, targetSpaceId);
  return {
    id: `okf:${targetKbId}:${targetConceptRowId}`,
    title: targetTitle,
    type: 'markdown',
    kbId: targetKbId,
    parentId: null,
    updatedAt: new Date().toISOString(),
    author: 'Knowledgebase',
    tags: sourceConcept.tags,
  };
}

export async function moveOkfConcept(
  sourceSpaceId: number,
  sourceConceptRowId: number,
  targetKbId: string,
  readMarkdown: (spaceId: number, conceptRowId: number) => Promise<string>,
): Promise<DocumentMeta> {
  const targetSpaceId = spaceIdFromKbId(targetKbId);
  if (sourceSpaceId === targetSpaceId) {
    throwKnowledgebaseError(KnowledgebaseErrorCodes.TRANSFER_SAME_KB);
  }

  const client = requireSdkClient();
  const sourceConcept = await client.knowledge.okf.concepts.retrieve(sourceConceptRowId);
  const markdown = await readMarkdown(sourceSpaceId, sourceConceptRowId);

  const upserted = await client.knowledge.okf.concepts.upsert({
    spaceId: targetSpaceId,
    conceptId: sourceConcept.conceptId,
    markdown,
    actor: 'pc-knowledgebase',
    publish: true,
  });

  const targetConceptRowId = await resolveOkfConceptRowIdByConceptId(
    targetSpaceId,
    sourceConcept.conceptId,
    upserted.logicalPath,
  );

  await client.knowledge.okf.concepts.delete(sourceConceptRowId);

  invalidateKnowledgeBrowserNodeCacheForSpaceIds(sourceSpaceId, targetSpaceId);
  return {
    id: `okf:${targetKbId}:${targetConceptRowId}`,
    title: sourceConcept.title,
    type: 'markdown',
    kbId: targetKbId,
    parentId: null,
    updatedAt: new Date().toISOString(),
    author: 'Knowledgebase',
    tags: sourceConcept.tags,
  };
}
