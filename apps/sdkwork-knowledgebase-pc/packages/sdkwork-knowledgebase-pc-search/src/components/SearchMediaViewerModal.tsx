import React, { useCallback, useEffect, useRef, useState } from 'react';
import {
  Disc3,
  ExternalLink,
  FolderOpen,
  Headphones,
  Image as ImageIcon,
  ListMusic,
  Maximize2,
  Minimize2,
  ShoppingBag,
  Video,
  X
} from 'lucide-react';
import type { SearchMediaCategory, SearchMediaItem, SearchNavigateToFilePayload } from '../types';
import { supportsMediaViewerMinimize } from './media-viewers/types';
import { hasSyncedTimedText } from '../utils/mediaTimedText';
import {
  getMediaOrientation,
  orientationClass,
  resolveCoverDimensions,
  resolveVideoDimensions
} from '../utils/mediaAspect';
import { openExternalUrl } from './media-viewers/openExternalUrl';
import { SearchMediaViewerPlaylistBody } from './SearchMediaViewerPlaylistBody';
import { SearchMediaViewerStandardBody } from './SearchMediaViewerStandardBody';
import { useSearchMediaPlaylist } from '../hooks/useSearchMediaPlaylist';
import { toNavigateFilePayload } from '../utils/sourceNavigation';

const CATEGORY_META: Record<SearchMediaCategory, { label: string; icon: React.ReactNode }> = {
  image: { label: '图片', icon: <ImageIcon className="w-4 h-4" /> },
  video: { label: '视频', icon: <Video className="w-4 h-4" /> },
  audio: { label: '音频', icon: <Headphones className="w-4 h-4" /> },
  music: { label: '音乐', icon: <Disc3 className="w-4 h-4" /> },
  product: { label: '商品', icon: <ShoppingBag className="w-4 h-4" /> }
};

const THUMB_FALLBACK_ICONS: Record<SearchMediaCategory, React.ReactNode> = {
  image: <ImageIcon className="w-4 h-4" />,
  video: <Video className="w-4 h-4" />,
  audio: <Headphones className="w-4 h-4" />,
  music: <Disc3 className="w-4 h-4" />,
  product: <ShoppingBag className="w-4 h-4" />
};

function isPlaylistCategory(category: SearchMediaCategory): category is 'music' | 'audio' {
  return category === 'music' || category === 'audio';
}

export interface SearchMediaViewerModalProps {
  items: SearchMediaItem[];
  activeIndex: number;
  category: SearchMediaCategory;
  onClose: () => void;
  onIndexChange: (index: number) => void;
  onGoToFile?: (payload: SearchNavigateToFilePayload) => void;
  onOpenWebLink?: (url: string, title?: string) => void;
}

export function SearchMediaViewerModal({
  items,
  activeIndex,
  category,
  onClose,
  onIndexChange,
  onGoToFile,
  onOpenWebLink
}: SearchMediaViewerModalProps) {
  const safeIndex = Math.min(Math.max(activeIndex, 0), Math.max(items.length - 1, 0));
  const item = items[safeIndex];
  const categoryMeta = CATEGORY_META[category];
  const isPlaylistViewer = isPlaylistCategory(category);
  const prevIndexRef = useRef(safeIndex);
  const [slideDirection, setSlideDirection] = useState<'prev' | 'next' | null>(null);
  const [isMinimized, setIsMinimized] = useState(false);
  const canMinimize = supportsMediaViewerMinimize(category);

  const {
    isPlaylistOpen,
    isPlaying,
    playMode,
    shuffleEnabled,
    playlistControls,
    setIsPlaylistOpen,
    setPlayMode,
    setShuffleEnabled,
    handlePreviousTrack,
    handleNextTrack,
    selectPlaylistTrack
  } = useSearchMediaPlaylist({
    items,
    activeIndex: safeIndex,
    enabled: isPlaylistViewer && items.length > 1,
    onIndexChange
  });

  const hasPlaylistUi = isPlaylistViewer && items.length > 1;

  useEffect(() => {
    setIsMinimized(false);
  }, [item?.id, category]);

  useEffect(() => {
    if (isMinimized) return;
    if (safeIndex > prevIndexRef.current) {
      setSlideDirection('next');
    } else if (safeIndex < prevIndexRef.current) {
      setSlideDirection('prev');
    }
    prevIndexRef.current = safeIndex;
  }, [safeIndex, isMinimized]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        onClose();
        return;
      }
      if (isMinimized) return;

      if (hasPlaylistUi) {
        if (event.key === 'ArrowLeft' && event.shiftKey) {
          handlePreviousTrack();
          return;
        }
        if (event.key === 'ArrowRight' && event.shiftKey) {
          handleNextTrack();
          return;
        }
        return;
      }

      if (event.key === 'ArrowLeft' && safeIndex > 0) {
        onIndexChange(safeIndex - 1);
      }
      if (event.key === 'ArrowRight' && safeIndex < items.length - 1) {
        onIndexChange(safeIndex + 1);
      }
    };

    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [handleNextTrack, handlePreviousTrack, hasPlaylistUi, isMinimized, items.length, onClose, onIndexChange, safeIndex]);

  const handleOpenInKb = useCallback(() => {
    if (!item || item.source !== 'kb' || !item.docId || !item.kbId || !item.docType || !onGoToFile) return;
    const payload = toNavigateFilePayload({
      id: item.id,
      title: item.title,
      type: 'doc',
      kbId: item.kbId,
      docId: item.docId,
      docType: item.docType,
      snippet: item.snippet ?? ''
    });
    if (payload) {
      onGoToFile(payload);
      onClose();
    }
  }, [item, onClose, onGoToFile]);

  if (!item) return null;

  const hasLyrics = category === 'music' && hasSyncedTimedText(item.lyrics);
  const hasTranscript = category === 'audio' && hasSyncedTimedText(item.transcript);
  const videoDims = category === 'video' ? resolveVideoDimensions(item) : null;
  const coverDims = category === 'audio' ? resolveCoverDimensions(item) : null;
  const videoOrientation = videoDims ? getMediaOrientation(videoDims.width, videoDims.height) : null;
  const coverOrientation = coverDims ? getMediaOrientation(coverDims.width, coverDims.height) : null;

  const dialogClass = [
    'search-media-viewer-dialog',
    category === 'product' && 'search-media-viewer-dialog--product',
    category === 'image' && 'search-media-viewer-dialog--image',
    category === 'video' && 'search-media-viewer-dialog--video',
    category === 'audio' && 'search-media-viewer-dialog--audio',
    category === 'music' && 'search-media-viewer-dialog--music',
    videoOrientation && orientationClass('search-media-viewer-dialog--video', videoOrientation),
    coverOrientation && orientationClass('search-media-viewer-dialog--audio', coverOrientation),
    hasLyrics && 'search-media-viewer-dialog--with-lyrics',
    hasTranscript && 'search-media-viewer-dialog--with-transcript',
    hasPlaylistUi && isPlaylistOpen && 'search-media-viewer-dialog--with-playlist',
    (category === 'image' || category === 'video' || category === 'music' || (category === 'audio' && hasTranscript)) &&
      'search-media-viewer-dialog--immersive',
    isMinimized && 'search-media-viewer-dialog--minimized'
  ]
    .filter(Boolean)
    .join(' ');

  const showExternal = Boolean(item.url ?? item.previewUrl) && category !== 'product';
  const contentSlideClass = !isMinimized && slideDirection ? `search-media-viewer-content--slide-${slideDirection}` : '';

  return (
    <div
      className={`search-media-viewer-overlay animate-in fade-in duration-200 ${isMinimized ? 'search-media-viewer-overlay--minimized' : ''}`}
    >
      {!isMinimized && <div className="search-media-viewer-backdrop" onClick={onClose} aria-hidden />}

      <div className={`${dialogClass} animate-in zoom-in-95 duration-200`} role="dialog" aria-modal="true" aria-label={`${categoryMeta.label}预览`}>
        <header className={`search-media-viewer-header ${isMinimized ? 'search-media-viewer-header--mini' : ''}`}>
          <div className="search-media-viewer-header-icon">{categoryMeta.icon}</div>
          <div className="search-media-viewer-header-text">
            {!isMinimized && <p className="search-media-viewer-header-label">{categoryMeta.label}</p>}
            <p className="search-media-viewer-header-title" title={item.title}>
              {item.title}
            </p>
          </div>
          {!isMinimized && items.length > 1 && (
            <span className="search-media-viewer-counter">
              {safeIndex + 1} / {items.length}
            </span>
          )}
          {!isMinimized && hasPlaylistUi && (
            <button
              type="button"
              className={`search-media-viewer-icon-btn ${isPlaylistOpen ? 'search-media-viewer-icon-btn--active' : ''}`}
              onClick={() => setIsPlaylistOpen((value) => !value)}
              title={isPlaylistOpen ? '收起播放列表' : '展开播放列表'}
            >
              <ListMusic className="w-4 h-4" />
            </button>
          )}
          {!isMinimized && item.source === 'kb' && onGoToFile && (
            <button type="button" className="search-media-viewer-icon-btn" onClick={handleOpenInKb} title="在知识库中打开">
              <FolderOpen className="w-4 h-4" />
            </button>
          )}
          {!isMinimized && showExternal && (
            <button
              type="button"
              className="search-media-viewer-icon-btn"
              onClick={() => openExternalUrl(item.url ?? item.previewUrl!, item.title, onOpenWebLink)}
              title="在浏览器中打开"
            >
              <ExternalLink className="w-4 h-4" />
            </button>
          )}
          {canMinimize && !isMinimized && (
            <button type="button" className="search-media-viewer-icon-btn" onClick={() => setIsMinimized(true)} title="最小化到右下角">
              <Minimize2 className="w-4 h-4" />
            </button>
          )}
          {canMinimize && isMinimized && (
            <button type="button" className="search-media-viewer-icon-btn" onClick={() => setIsMinimized(false)} title="展开">
              <Maximize2 className="w-4 h-4" />
            </button>
          )}
          <button type="button" className="search-media-viewer-icon-btn search-media-viewer-icon-btn--close" onClick={onClose} title="关闭">
            <X className="w-4 h-4" />
          </button>
        </header>

        {isPlaylistViewer ? (
          <SearchMediaViewerPlaylistBody
            item={item}
            items={items}
            category={category}
            activeIndex={safeIndex}
            isMinimized={isMinimized}
            isPlaylistOpen={isPlaylistOpen}
            isPlaying={isPlaying}
            playMode={playMode}
            shuffleEnabled={shuffleEnabled}
            hasLyrics={hasLyrics}
            hasTranscript={hasTranscript}
            contentSlideClass={contentSlideClass}
            playlistControls={playlistControls}
            onOpenWebLink={onOpenWebLink}
            onPlayModeChange={(mode) => {
              setPlayMode(mode);
              if (mode !== 'shuffle') setShuffleEnabled(false);
            }}
            onShuffleToggle={() => setShuffleEnabled((value) => !value)}
            onPlaylistClose={() => setIsPlaylistOpen(false)}
            onPlaylistSelect={selectPlaylistTrack}
          />
        ) : (
          <SearchMediaViewerStandardBody
            item={item}
            items={items}
            category={category}
            activeIndex={safeIndex}
            isMinimized={isMinimized}
            contentSlideClass={contentSlideClass}
            thumbFallback={THUMB_FALLBACK_ICONS[category]}
            onIndexChange={onIndexChange}
            onOpenWebLink={onOpenWebLink}
          />
        )}
      </div>
    </div>
  );
}
