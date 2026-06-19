import React, { useEffect, useState } from 'react';
import {
  ChevronDown,
  ChevronRight,
  ExternalLink
} from 'lucide-react';
import { dispatchOpenInAppBrowser } from '@packages/sdkwork-knowledgebase-pc-commons/src';
import type {
  SearchNavigateToFilePayload,
  SearchNavigateToKbPayload,
  SearchSource
} from '../types';
import {
  formatSourceUpdatedAt,
  getCitationIndex,
  getDocTypeLabel,
  getSourceHostLabel,
  groupSearchSources
} from '../utils/sources';
import { toNavigateFilePayload } from '../utils/sourceNavigation';
import { showSearchToast } from '../utils/searchToast';
import { SEARCH_EXPAND_SOURCES_EVENT } from '../utils/markdown';

export interface MessageSourcesPanelProps {
  sources: SearchSource[];
  messageId: string;
  onGoToKb: (payload: SearchNavigateToKbPayload) => void;
  onGoToFile: (payload: SearchNavigateToFilePayload) => void;
  onOpenWebLink?: (url: string, title?: string) => void;
}

interface SourceGroupProps {
  title: string;
  variant: 'kb' | 'web';
  count: number;
  emptyHint?: string;
  children: React.ReactNode;
}

function SourceGroup({ title, variant, count, emptyHint, children }: SourceGroupProps) {
  if (count === 0 && !emptyHint) return null;

  return (
    <section className={`search-sources-group search-sources-group--${variant}`}>
      <div className="search-sources-group-header">
        <span className="search-sources-group-title">{title}</span>
        <span className={`search-sources-group-count search-sources-group-count--${variant}`}>{count}</span>
      </div>
      {count > 0 ? (
        <div className="search-sources-list">{children}</div>
      ) : emptyHint ? (
        <p className="search-sources-empty-hint">{emptyHint}</p>
      ) : null}
    </section>
  );
}

export function MessageSourcesPanel({
  sources,
  messageId,
  onGoToKb,
  onGoToFile,
  onOpenWebLink
}: MessageSourcesPanelProps) {
  const [expanded, setExpanded] = useState(false);

  useEffect(() => {
    const handleExpand = (event: Event) => {
      const detail = (event as CustomEvent<{ messageId: string }>).detail;
      if (detail?.messageId !== messageId) return;
      setExpanded(true);
    };

    window.addEventListener(SEARCH_EXPAND_SOURCES_EVENT, handleExpand);
    return () => window.removeEventListener(SEARCH_EXPAND_SOURCES_EVENT, handleExpand);
  }, [messageId]);

  if (sources.length === 0) return null;

  const { docSources, kbSources, webSources, citationIndexById } = groupSearchSources(sources);
  const localFileCount = docSources.length;
  const localKbCount = kbSources.length;
  const webCount = webSources.length;
  const localTotal = localFileCount + localKbCount;

  const summaryParts: string[] = [];
  if (localTotal > 0) summaryParts.push(`知识库 ${localTotal}`);
  if (webCount > 0) summaryParts.push(`网络 ${webCount}`);
  const summaryText = summaryParts.join(' · ');

  const openWeb = (url: string, title?: string) => {
    if (onOpenWebLink) {
      onOpenWebLink(url, title);
      return;
    }
    dispatchOpenInAppBrowser({ url, title });
  };

  const handleSourceClick = (src: SearchSource) => {
    if (src.type === 'doc') {
      const payload = toNavigateFilePayload(src);
      if (payload) {
        onGoToFile(payload);
        showSearchToast(`正在打开《${payload.title}》`, 'success');
        return;
      }
      showSearchToast('无法定位该文件记录', 'default');
      return;
    }

    if (src.type === 'kb' && src.kbId) {
      onGoToKb({ kbId: src.kbId, kbTitle: src.kbTitle ?? src.title });
      showSearchToast(`正在打开知识库「${src.kbTitle ?? src.title}」`, 'success');
      return;
    }

    if (src.url) {
      openWeb(src.url, src.title);
    }
  };

  const renderMetaLine = (src: SearchSource) => {
    if (src.type === 'doc') {
      const parts = [
        getDocTypeLabel(src.docType),
        src.kbTitle ? src.kbTitle : null,
        src.author ? src.author : null,
        formatSourceUpdatedAt(src.updatedAt)
      ].filter(Boolean);
      return parts.join(' · ');
    }
    if (src.type === 'kb') {
      return src.snippet.slice(0, 56) + (src.snippet.length > 56 ? '…' : '');
    }
    return getSourceHostLabel(src);
  };

  const renderSourceRow = (src: SearchSource) => {
    const isWeb = src.type === 'web';
    const citationNum = getCitationIndex(src, citationIndexById);
    const meta = renderMetaLine(src);

    return (
      <button
        key={src.id}
        type="button"
        id={`citation-card-${messageId}-${citationNum}`}
        onClick={() => handleSourceClick(src)}
        className={`search-source-row search-source-row--${src.type} group`}
      >
        <span className="search-citation-badge static shrink-0">{citationNum}</span>
        <span className="min-w-0 flex-1">
          <span className="search-source-title">{src.title}</span>
          {meta ? <span className="search-source-meta">{meta}</span> : null}
        </span>
        {isWeb ? (
          <ExternalLink className="search-source-action" aria-hidden />
        ) : (
          <ChevronRight className="search-source-action" aria-hidden />
        )}
      </button>
    );
  };

  return (
    <div className="search-sources-footer animate-in fade-in duration-200">
      <button
        type="button"
        className="search-sources-toggle"
        onClick={() => setExpanded((value) => !value)}
        aria-expanded={expanded}
      >
        <span className="search-sources-toggle-label">引用来源</span>
        <span className="search-sources-toggle-meta">{summaryText}</span>
        <ChevronDown className={`search-sources-toggle-chevron ${expanded ? 'search-sources-toggle-chevron--open' : ''}`} />
      </button>

      {expanded && (
        <div className="search-sources-stack">
          {(localTotal > 0 || webCount > 0) && (
            <SourceGroup
              title="知识库"
              variant="kb"
              count={localTotal}
              emptyHint={webCount > 0 ? '未匹配到知识库文件' : undefined}
            >
              {docSources.map((src) => renderSourceRow(src))}
              {kbSources.map((src) => renderSourceRow(src))}
            </SourceGroup>
          )}

          {webCount > 0 && (
            <SourceGroup
              title="网络来源"
              variant="web"
              count={webCount}
            >
              {webSources.map((src) => renderSourceRow(src))}
            </SourceGroup>
          )}
        </div>
      )}
    </div>
  );
}
