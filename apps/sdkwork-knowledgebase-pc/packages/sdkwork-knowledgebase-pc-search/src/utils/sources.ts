import type { SearchSource } from '../types';

export function getSourceHostLabel(src: SearchSource): string {
  if (!src.url) return src.snippet.slice(0, 72) + (src.snippet.length > 72 ? '…' : '');
  try {
    return new URL(src.url).hostname.replace(/^www\./, '');
  } catch {
    return src.url;
  }
}

export function getSourceTypeLabel(type: SearchSource['type']): string {
  if (type === 'doc') return '知识库文件';
  if (type === 'kb') return '知识库目录';
  return '网络链接';
}

export function getSourceTypePill(type: SearchSource['type']): string {
  if (type === 'doc') return '文件';
  if (type === 'kb') return '目录';
  return '网页';
}

const DOC_TYPE_LABELS: Record<string, string> = {
  richtext: '富文本',
  markdown: 'Markdown',
  code: '代码',
  pdf: 'PDF',
  image: '图片',
  video: '视频',
  audio: '音频',
  music: '音乐',
  file: '文件'
};

export function getDocTypeLabel(docType?: SearchSource['docType']): string {
  if (!docType) return '文档';
  return DOC_TYPE_LABELS[docType] ?? '文档';
}

export function formatSourceUpdatedAt(updatedAt?: string): string | null {
  if (!updatedAt) return null;
  try {
    return new Date(updatedAt).toLocaleDateString();
  } catch {
    return null;
  }
}

export interface GroupedSearchSources {
  docSources: SearchSource[];
  kbSources: SearchSource[];
  webSources: SearchSource[];
  citationIndexById: Map<string, number>;
}

/** Build groups + stable citation numbers matching the unified sources array order */
export function groupSearchSources(sources: SearchSource[]): GroupedSearchSources {
  const citationIndexById = new Map<string, number>();
  sources.forEach((source, index) => {
    citationIndexById.set(source.id, index + 1);
  });

  return {
    docSources: sources.filter((s) => s.type === 'doc'),
    kbSources: sources.filter((s) => s.type === 'kb'),
    webSources: sources.filter((s) => s.type === 'web'),
    citationIndexById
  };
}

export function getCitationIndex(source: SearchSource, indexMap: Map<string, number>): number {
  return indexMap.get(source.id) ?? 0;
}
