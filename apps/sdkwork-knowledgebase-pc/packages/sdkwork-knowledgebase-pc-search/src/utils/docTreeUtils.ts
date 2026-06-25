import type { DocumentMeta, FolderNode } from '@sdkwork/sdkwork-knowledgebase-pc-knowledgebase/services/document';

export function findDocInTree(
  nodes: (FolderNode | DocumentMeta)[],
  docId: string,
  parentId: string | null = null
): DocumentMeta | null {
  for (const node of nodes) {
    if (node.id === docId && node.type !== 'folder') {
      return { ...(node as DocumentMeta), parentId: (node as DocumentMeta).parentId ?? parentId };
    }
    if (node.type === 'folder') {
      const folder = node as FolderNode;
      if (folder.children?.length) {
        const found = findDocInTree(folder.children, docId, folder.id);
        if (found) return found;
      }
    }
  }
  return null;
}

export function findKbInCollections(
  kbs: { team?: { id: string; title: string }[]; personal?: { id: string; title: string }[]; public?: { id: string; title: string }[] },
  kbId: string
): { id: string; title: string } | null {
  const all = [...(kbs.team ?? []), ...(kbs.personal ?? []), ...(kbs.public ?? [])];
  return all.find((kb) => kb.id === kbId) ?? null;
}
