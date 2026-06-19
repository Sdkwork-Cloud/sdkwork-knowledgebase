import React, { useState } from 'react';
import {
  Image as ImageIcon,
  Video,
  Headphones,
  Disc3,
  ShoppingBag,
  MessageSquareText,
  Play,
  ExternalLink,
  Library,
  Star
} from 'lucide-react';
import type {
  SearchMediaItem,
  SearchMediaTab,
  SearchNavigateToFilePayload,
  SearchNavigateToKbPayload,
  SearchRelatedMedia
} from '../types';
import { getMediaItemsForTab, getMediaTabDefs, hasRelatedMedia } from '../utils/mediaResults';
import { hasSyncedTimedText } from '../utils/mediaTimedText';
import { dispatchOpenSearchMediaViewer } from '../utils/searchMediaViewerBridge';

export interface SearchMediaPanelProps {
  relatedMedia?: SearchRelatedMedia;
  onGoToFile: (payload: SearchNavigateToFilePayload) => void;
  onOpenWebLink?: (url: string, title?: string) => void;
}

function openMediaViewer(items: SearchMediaItem[], item: SearchMediaItem) {
  const activeIndex = items.findIndex((entry) => entry.id === item.id);
  dispatchOpenSearchMediaViewer({
    items,
    activeIndex: activeIndex >= 0 ? activeIndex : 0,
    category: item.category
  });
}

function handleMediaClick(items: SearchMediaItem[], item: SearchMediaItem) {
  openMediaViewer(items, item);
}

const TAB_ICONS: Record<SearchMediaTab, React.ReactNode> = {
  answer: <MessageSquareText className="w-3.5 h-3.5" />,
  image: <ImageIcon className="w-3.5 h-3.5" />,
  video: <Video className="w-3.5 h-3.5" />,
  audio: <Headphones className="w-3.5 h-3.5" />,
  music: <Disc3 className="w-3.5 h-3.5" />,
  product: <ShoppingBag className="w-3.5 h-3.5" />
};

function MediaTimedTextBadge({ item }: { item: SearchMediaItem }) {
  const hasLyrics = hasSyncedTimedText(item.lyrics);
  const hasTranscript = hasSyncedTimedText(item.transcript);
  const isMeeting = hasTranscript && (item.audioKind === 'recording' || item.audioKind === 'speech');

  if (!hasLyrics && !hasTranscript) return null;

  return (
    <span
      className={`search-media-timed-badge ${isMeeting ? 'search-media-timed-badge--meeting' : hasLyrics ? 'search-media-timed-badge--lyrics' : 'search-media-timed-badge--caption'}`}
    >
      {hasLyrics ? '歌词' : isMeeting ? '会议纪要' : '字幕'}
    </span>
  );
}

function MediaSourceBadge({ item }: { item: SearchMediaItem }) {
  return (
    <span
      className={`search-media-source-badge ${item.source === 'kb' ? 'search-media-source-badge--kb' : 'search-media-source-badge--web'}`}
    >
      {item.source === 'kb' ? (
        <>
          <Library className="w-3 h-3" />
          知识库
        </>
      ) : (
        <>
          <ExternalLink className="w-3 h-3" />
          网络
        </>
      )}
    </span>
  );
}

function ImageGrid({ items }: { items: SearchMediaItem[] }) {
  return (
    <div className="search-media-grid search-media-grid--images">
      {items.map((item) => (
        <button
          key={item.id}
          type="button"
          className="search-media-card search-media-card--image"
          onClick={() => handleMediaClick(items, item)}
        >
          <div
            className="search-media-card-thumb search-media-card-thumb--variable"
            style={
              item.imageWidth && item.imageHeight
                ? { aspectRatio: `${item.imageWidth} / ${item.imageHeight}` }
                : undefined
            }
          >
            {item.thumbnailUrl ? (
              <img src={item.thumbnailUrl} alt={item.title} loading="lazy" />
            ) : (
              <div className="search-media-card-placeholder">
                <ImageIcon className="w-8 h-8 opacity-40" />
              </div>
            )}
            <span className="search-media-card-hover">预览</span>
          </div>
          <div className="search-media-card-body">
            <p className="search-media-card-title">{item.title}</p>
            <MediaSourceBadge item={item} />
          </div>
        </button>
      ))}
    </div>
  );
}

function VideoGrid({ items }: { items: SearchMediaItem[] }) {
  return (
    <div className="search-media-grid search-media-grid--videos">
      {items.map((item) => (
        <button
          key={item.id}
          type="button"
          className="search-media-card search-media-card--video"
          onClick={() => handleMediaClick(items, item)}
        >
          <div
            className="search-media-card-thumb search-media-card-thumb--variable"
            style={
              item.videoWidth && item.videoHeight
                ? { aspectRatio: `${item.videoWidth} / ${item.videoHeight}` }
                : undefined
            }
          >
            {item.thumbnailUrl ? (
              <img src={item.thumbnailUrl} alt={item.title} loading="lazy" />
            ) : (
              <div className="search-media-card-placeholder">
                <Video className="w-8 h-8 opacity-40" />
              </div>
            )}
            <span className="search-media-play-badge">
              <Play className="w-4 h-4 fill-current" />
            </span>
            <span className="search-media-card-hover">播放</span>
            {item.duration && <span className="search-media-duration">{item.duration}</span>}
          </div>
          <div className="search-media-card-body">
            <p className="search-media-card-title">{item.title}</p>
            {item.snippet && <p className="search-media-card-snippet">{item.snippet}</p>}
            <MediaSourceBadge item={item} />
          </div>
        </button>
      ))}
    </div>
  );
}

function AudioList({ items }: { items: SearchMediaItem[] }) {
  return (
    <div className="search-media-list search-media-list--audio">
      {items.map((item) => (
        <button
          key={item.id}
          type="button"
          className="search-media-row search-media-row--audio"
          onClick={() => handleMediaClick(items, item)}
        >
          <div className="search-media-row-thumb search-media-row-thumb--variable search-media-row-thumb--playable"
            style={
              item.coverWidth && item.coverHeight
                ? { aspectRatio: `${item.coverWidth} / ${item.coverHeight}` }
                : undefined
            }
          >
            {item.thumbnailUrl ? (
              <img src={item.thumbnailUrl} alt={item.title} loading="lazy" />
            ) : (
              <Headphones className="w-5 h-5 opacity-50" />
            )}
            <span className="search-media-row-play">
              <Play className="w-3.5 h-3.5 fill-current" />
            </span>
          </div>
          <div className="search-media-row-body">
            <p className="search-media-card-title">{item.title}</p>
            {item.snippet && <p className="search-media-card-snippet">{item.snippet}</p>}
            <div className="search-media-row-meta">
              <MediaTimedTextBadge item={item} />
              <MediaSourceBadge item={item} />
            </div>
          </div>
          {item.duration && <span className="search-media-duration">{item.duration}</span>}
        </button>
      ))}
    </div>
  );
}

function MusicList({ items }: { items: SearchMediaItem[] }) {
  return (
    <div className="search-media-list search-media-list--music">
      {items.map((item) => (
        <button
          key={item.id}
          type="button"
          className="search-media-row search-media-row--music"
          onClick={() => handleMediaClick(items, item)}
        >
          <div className="search-media-row-thumb search-media-row-thumb--square search-media-row-thumb--playable">
            {item.thumbnailUrl ? (
              <img src={item.thumbnailUrl} alt={item.title} loading="lazy" />
            ) : (
              <Disc3 className="w-5 h-5 opacity-50" />
            )}
            <span className="search-media-row-play">
              <Play className="w-3.5 h-3.5 fill-current" />
            </span>
          </div>
          <div className="search-media-row-body">
            <p className="search-media-card-title">{item.title}</p>
            <p className="search-media-card-snippet">{item.artist ?? item.snippet}</p>
            <div className="search-media-row-meta">
              <MediaTimedTextBadge item={item} />
              <MediaSourceBadge item={item} />
            </div>
          </div>
          {item.duration && <span className="search-media-duration">{item.duration}</span>}
        </button>
      ))}
    </div>
  );
}

function ProductGrid({ items }: { items: SearchMediaItem[] }) {
  return (
    <div className="search-media-grid search-media-grid--products">
      {items.map((item) => (
        <button
          key={item.id}
          type="button"
          className="search-media-card search-media-card--product"
          onClick={() => handleMediaClick(items, item)}
        >
          <div className="search-media-card-thumb search-media-card-thumb--square">
            {item.thumbnailUrl ? (
              <img src={item.thumbnailUrl} alt={item.title} loading="lazy" />
            ) : (
              <div className="search-media-card-placeholder">
                <ShoppingBag className="w-8 h-8 opacity-40" />
              </div>
            )}
            <span className="search-media-card-hover">详情</span>
          </div>
          <div className="search-media-card-body">
            <p className="search-media-card-title">{item.title}</p>
            <div className="search-media-product-meta">
              {item.price && <span className="search-media-price">{item.price}</span>}
              {item.rating != null && (
                <span className="search-media-rating">
                  <Star className="w-3 h-3 fill-amber-400 text-amber-400" />
                  {item.rating.toFixed(1)}
                </span>
              )}
            </div>
            {item.merchant && <p className="search-media-card-snippet">{item.merchant}</p>}
            <MediaSourceBadge item={item} />
          </div>
        </button>
      ))}
    </div>
  );
}

function MediaTabContent({
  tab,
  relatedMedia
}: {
  tab: SearchMediaTab;
  relatedMedia: SearchRelatedMedia;
}) {
  const items = getMediaItemsForTab(relatedMedia, tab);
  if (items.length === 0) {
    return <p className="search-media-empty">暂无相关内容</p>;
  }

  if (tab === 'image') {
    return <ImageGrid items={items} />;
  }
  if (tab === 'video') {
    return <VideoGrid items={items} />;
  }
  if (tab === 'audio') {
    return <AudioList items={items} />;
  }
  if (tab === 'music') {
    return <MusicList items={items} />;
  }
  if (tab === 'product') {
    return <ProductGrid items={items} />;
  }
  return null;
}

export interface SearchMediaTabsProps extends SearchMediaPanelProps {
  activeTab: SearchMediaTab;
  onTabChange: (tab: SearchMediaTab) => void;
}

export function SearchMediaTabs({
  relatedMedia,
  activeTab,
  onTabChange
}: SearchMediaTabsProps) {
  if (!hasRelatedMedia(relatedMedia)) return null;

  const tabs = getMediaTabDefs(relatedMedia);

  return (
    <div className="search-media-tabs" role="tablist" aria-label="检索结果分类">
      {tabs.map((tab) => (
        <button
          key={tab.id}
          type="button"
          role="tab"
          aria-selected={activeTab === tab.id}
          className={`search-media-tab ${activeTab === tab.id ? 'search-media-tab--active' : ''}`}
          onClick={() => onTabChange(tab.id)}
        >
          {TAB_ICONS[tab.id]}
          <span>{tab.label}</span>
          {tab.id !== 'answer' && tab.count > 0 && (
            <span className="search-media-tab-count">{tab.count}</span>
          )}
        </button>
      ))}
    </div>
  );
}

export function SearchMediaPanel({
  relatedMedia,
  activeTab
}: SearchMediaTabsProps & { activeTab: Exclude<SearchMediaTab, 'answer'> }) {
  if (!relatedMedia) return null;

  return (
    <div className="search-media-panel animate-in fade-in duration-300" role="tabpanel">
      <MediaTabContent tab={activeTab} relatedMedia={relatedMedia} />
    </div>
  );
}

export function useSearchMediaView(relatedMedia?: SearchRelatedMedia) {
  const [activeTab, setActiveTab] = useState<SearchMediaTab>('answer');
  const showMediaPanel = activeTab !== 'answer' && hasRelatedMedia(relatedMedia);
  return { activeTab, setActiveTab, showMediaPanel };
}

export function SearchMessageMediaSwitcher({
  relatedMedia,
  onGoToFile,
  onOpenWebLink,
  activeTab,
  onTabChange
}: SearchMediaTabsProps) {
  if (!hasRelatedMedia(relatedMedia)) return null;

  return (
    <>
      <SearchMediaTabs
        relatedMedia={relatedMedia}
        activeTab={activeTab}
        onTabChange={onTabChange}
        onGoToFile={onGoToFile}
        onOpenWebLink={onOpenWebLink}
      />
      {activeTab !== 'answer' && relatedMedia && (
        <SearchMediaPanel
          relatedMedia={relatedMedia}
          activeTab={activeTab as Exclude<SearchMediaTab, 'answer'>}
          onGoToFile={onGoToFile}
          onOpenWebLink={onOpenWebLink}
          onTabChange={onTabChange}
        />
      )}
    </>
  );
}
