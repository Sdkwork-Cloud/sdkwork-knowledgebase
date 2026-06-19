import type { SearchSource } from '../types';

export function getSourceDocId(source: SearchSource): string | undefined {
  if (source.docId) return source.docId;
  if (source.type === 'doc' && source.id.startsWith('doc-')) {
    return source.id.slice(4);
  }
  return undefined;
}

export function toNavigateFilePayload(source: SearchSource) {
  const docId = getSourceDocId(source);
  if (!source.kbId || !docId) return null;
  return {
    kbId: source.kbId,
    docId,
    title: source.title,
    type: source.docType ?? 'richtext',
    kbTitle: source.kbTitle,
    author: source.author,
    updatedAt: source.updatedAt,
    parentId: source.parentId
  };
}
